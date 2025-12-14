use bollard::{
    Docker, query_parameters::CreateContainerOptionsBuilder, secret::ContainerCreateBody,
};
use uuid::Uuid;

use crate::logger::Logger;

pub struct BoaContainer {
    logger: Logger,
    container_id: String,
}

impl BoaContainer {
    pub async fn new(docker: &Docker, container_prefix: String) -> Result<BoaContainer, String> {
        let container_name = format!("{container_prefix}-{}", Uuid::new_v4());

        let logger = Logger::new(format!("[boa-server#.{container_name}]"));

        let container_options = CreateContainerOptionsBuilder::new()
            .name(&container_prefix)
            .build();

        let container_create = ContainerCreateBody {
            image: Some("python:3-11-alpine".to_string()),
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

        Ok(BoaContainer {
            logger,
            container_id: container.id,
        })
    }
}
