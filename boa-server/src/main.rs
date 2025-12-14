mod routes;

use std::env;

use axum::{Router, routing::get};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), String> {
    let server_port = match env::var("BOA_SERVER_PORT") {
        Ok(port) => port,
        Err(e) => return Err(format!("Failed to read BOA_SERVER_PORT: {e}!")),
    };

    let server_port = match server_port.parse::<u32>() {
        Ok(port) => port,
        Err(e) => {
            return Err(format!(
                "Failed to parse BOA_SERVER_PORT as a valid port: {e}!"
            ));
        }
    };

    let server_url = format!("0.0.0.0:{server_port}");

    let router = Router::new()
        .route("/", get(|| async { "Hello world!" }))
        .route("/ws", get(routes::ws::ws_handler));

    let listener = match TcpListener::bind(&server_url).await {
        Ok(listener) => listener,
        Err(e) => return Err(format!("Failed to bind TCP listener to {server_url}: {e}!")),
    };

    println!("[boa-server] listening on {server_url}");

    axum::serve(listener, router)
        .await
        .map_err(|e| format!("Failed to serve router: {e}!"))?;

    Ok(())
}
