use std::{fs::File, path::Path};

use axum::body::Bytes;

use futures_util::stream::StreamExt;

use bollard::{
    Docker, body_full,
    exec::{CreateExecOptions, StartExecResults},
    query_parameters::{
        CreateContainerOptionsBuilder, StartContainerOptions, UploadToContainerOptionsBuilder,
    },
    secret::ContainerCreateBody,
};

use owo_colors::Style;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use crate::{logger::Logger, routes::ws::WsOutbound};

#[derive(Clone)]
pub struct BoaContainer {
    pub logger: Logger,
    pub container_id: String,
    pub started: bool,
    pub executing: bool,
}

impl BoaContainer {
    pub async fn new(
        docker: &Docker,
        container_prefix: String,
    ) -> Result<(String, BoaContainer), String> {
        let container_name = format!("{container_prefix}-{}", Uuid::new_v4());

        let logger = Logger::new(format!("[boa-server#.{container_name}]"));

        logger.log("creating new container", "");

        let container_options = CreateContainerOptionsBuilder::new()
            .name(&container_name)
            .build();

        let container_create = ContainerCreateBody {
            image: Some("python:3.11-slim".to_string()),
            tty: Some(true),
            open_stdin: Some(true),

            cmd: Some(vec![
                "tail".to_string(),
                "-f".to_string(),
                "/dev/null".to_string(),
            ]),

            attach_stdin: Some(true),
            attach_stdout: Some(true),
            attach_stderr: Some(true),

            ..Default::default()
        };

        let container = docker
            .create_container(Some(container_options), container_create)
            .await
            .map_err(|e| format!("failed to create new docker container {container_name}: {e}!"))?;

        logger.log("created new container", "");

        Ok((
            container.id,
            BoaContainer {
                logger,
                container_id: container_name,
                started: false,
                executing: false,
            },
        ))
    }
}

impl BoaContainer {
    pub async fn upload_tar(&self, docker: &Docker, tar: Bytes) -> Result<(), String> {
        docker
            .upload_to_container(
                &self.container_id,
                Some(UploadToContainerOptionsBuilder::new().path("/src/").build()),
                body_full(tar),
            )
            .await
            .map_err(|e| format!("failed to upload tar to container: {e}"))
    }

    pub async fn start(&mut self, docker: &Docker) -> Result<(), String> {
        match docker
            .start_container(&self.container_id, Some(StartContainerOptions::default()))
            .await
        {
            Ok(_) => {
                self.started = true;
                self.logger.log("started container", "");
            }
            Err(e) => return Err(format!("failed to start container: {e}")),
        };

        Ok(())
    }
}

impl BoaContainer {
    pub async fn run(
        &mut self,
        docker: &Docker,
        file_path: String,
        sender: UnboundedSender<WsOutbound>,
    ) -> Result<i64, String> {
        if !self.started {
            return Err("container is not started".to_string());
        }

        self.logger.log("creating exec", "");

        let exec = docker
            .create_exec(
                &self.container_id,
                CreateExecOptions {
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    attach_stdin: Some(false),
                    tty: Some(false),
                    cmd: Some(vec!["python".to_string(), file_path]),
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| format!("failed to create exec: {e}"))?;

        self.logger.log("starting exec", "");

        self.executing = true;

        let mut output = docker
            .start_exec(&exec.id, None)
            .await
            .map_err(|e| format!("failed to start exec: {e}"))?;

        match output {
            StartExecResults::Attached { mut output, .. } => {
                while let Some(msg) = output.next().await {
                    match msg {
                        Ok(bollard::container::LogOutput::StdOut { message }) => {
                            let text = String::from_utf8_lossy(&message);
                            self.logger.log(format!("stdout: {text}"), "");
                            // TODO: send StdOut
                        }
                        Ok(bollard::container::LogOutput::StdErr { message }) => {
                            let text = String::from_utf8_lossy(&message);
                            self.logger.log(format!("stderr: {text}"), "");
                            // TODO: send StdErr
                        }
                        _ => {}
                    }
                }
            }
            StartExecResults::Detached => {
                self.logger.log_style(
                    "why is this running in detached mode?",
                    Style::new().bright_red(),
                    "",
                );
            }
        }

        let inspect = docker
            .inspect_exec(&exec.id)
            .await
            .map_err(|e| format!("failed to inspect exec: {e}"))?;

        let exit_code = inspect.exit_code.unwrap_or(-1);

        self.logger
            .log("exec finished", &format!("exit_code={exit_code}"));

        Ok(exit_code)
    }
}

impl BoaContainer {
    pub async fn upload_file(
        &self,
        docker: &Docker,
        host_path: &Path,
        container_path: &str,
    ) -> Result<(), String> {
        let mut tar_data = Vec::new();

        {
            let mut file = std::fs::File::open(host_path)
                .map_err(|e| format!("open host file failed: {e}"))?;

            let mut builder = tar::Builder::new(&mut tar_data);

            builder
                .append_file(
                    Path::new(container_path)
                        .file_name()
                        .ok_or("invalid container path")?,
                    &mut file,
                )
                .map_err(|e| format!("tar append failed: {e}"))?;

            builder.finish().map_err(|e| e.to_string())?;
        }

        docker
            .upload_to_container(
                &self.container_id,
                Some(
                    UploadToContainerOptionsBuilder::new()
                        .path(
                            Path::new(container_path)
                                .parent()
                                .unwrap_or(Path::new("/"))
                                .to_str()
                                .unwrap(),
                        )
                        .build(),
                ),
                body_full(tar_data.into()),
            )
            .await
            .map_err(|e| format!("docker upload failed: {e}"))
    }
}
