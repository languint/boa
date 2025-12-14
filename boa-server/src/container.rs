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
use owo_colors::OwoColorize;
use uuid::Uuid;

use crate::logger::Logger;

#[derive(Clone)]
pub struct BoaContainer {
    pub logger: Logger,
    pub container_id: String,
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

        logger.log(
            format!("created new container {}", container_name.bold()),
            "",
        );

        Ok((
            container.id,
            BoaContainer {
                logger,
                container_id: container_name,
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

    pub async fn start(&self, docker: &Docker) -> Result<(), String> {
        docker
            .start_container(&self.container_id, Some(StartContainerOptions::default()))
            .await
            .map_err(|e| format!("failed to start container: {e}"))
    }
}

impl BoaContainer {
    pub async fn run(&self, docker: &Docker, file_path: String) -> Result<i64, String> {
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
                            // → send ProcessOutputPacket::StdOut
                        }
                        Ok(bollard::container::LogOutput::StdErr { message }) => {
                            let text = String::from_utf8_lossy(&message);
                            self.logger.log(format!("stderr: {text}"), "");
                            // → send ProcessOutputPacket::StdErr
                        }
                        _ => {}
                    }
                }
            }
            StartExecResults::Detached => {
                // Not expected unless detach = true
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
