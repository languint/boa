mod container;
mod logger;
mod routes;
mod state;

use std::{env, process::exit, sync::Arc};

use axum::{Router, routing::get};
use bollard::Docker;
use owo_colors::Style;
use tokio::{net::TcpListener, sync::Mutex};

use crate::{logger::Logger, state::ServerState};

#[tokio::main]
async fn main() {
    let logger = Logger::new("boa-server".to_string());

    let server_port = match env::var("BOA_SERVER_PORT") {
        Ok(port) => port,
        Err(e) => {
            logger.err(format!("failed to read BOA_SERVER_PORT: {e}!"), "~!");
            exit(1);
        }
    };

    let server_port = match server_port.parse::<u32>() {
        Ok(port) => port,
        Err(_) => {
            logger.err("failed to parse BOA_SERVER_PORT!", "~!");
            exit(1);
        }
    };

    let container_prefix = match env::var("BOA_CONTAINER_PREFIX") {
        Ok(container_name) => container_name,
        Err(e) => {
            logger.err(format!("failed to read BOA_CONTAINER_PREFIX: {e}!"), "");
            exit(1);
        }
    };

    let docker = match Docker::connect_with_local_defaults() {
        Ok(docker) => docker,
        Err(e) => {
            logger.err(format!("failed to connect to docker daemon: {e}!"), "");
            exit(1);
        }
    };

    let server_state = Arc::new(Mutex::new(ServerState::new(
        docker,
        server_port,
        container_prefix,
    )));

    let server_url = format!("0.0.0.0:{server_port}");

    let router = Router::new()
        .route("/", get(|| async { "Hello world!" }))
        .route(
            "/ws",
            get(async |ws| {
                let route = Arc::new(routes::ws::BoaWsRoute::new(server_state));

                route.ws_handler(ws)
            }),
        );

    let listener = match TcpListener::bind(&server_url).await {
        Ok(listener) => listener,
        Err(e) => {
            logger.err(
                format!("failed to bind TCP listener to {server_url}: {e}!"),
                "",
            );
            exit(1);
        }
    };

    logger.log_style(
        format!("server listening on {server_url}"),
        Style::new().bright_green(),
        "",
    );

    if let Err(e) = axum::serve(listener, router).await {
        logger.err(format!("failed to serve router: {e}!"), "");
        exit(1);
    }

    logger.log("all done, exiting", "");
    exit(0);
}
