use std::sync::Arc;

use axum::{
    extract::ws::{CloseFrame, Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use boa_core::packets::{
    client::ClientPacket,
    server::{
        ServerPacket,
        error::{ServerError, ServerErrorPacket},
        process::{ProcessCloseResultPacket, ProcessOpenResultPacket},
    },
};
use bollard::query_parameters::{RemoveContainerOptions, StopContainerOptions};
use owo_colors::{OwoColorize, Style};

use crate::{container::BoaContainer, logger::Logger, state::ShareableServerState};

pub struct BoaWsRoute {
    logger: Arc<Logger>,
    server_state: ShareableServerState,
}

impl BoaWsRoute {
    pub fn new(server_state: ShareableServerState) -> BoaWsRoute {
        BoaWsRoute {
            logger: Arc::new(Logger::new("boa-server~/ws".to_string())),
            server_state,
        }
    }
}

impl BoaWsRoute {
    pub async fn ws_handler(self: Arc<Self>, ws: WebSocketUpgrade) -> impl IntoResponse {
        self.logger.log("new connection opened", "");

        ws.on_upgrade(move |socket| {
            let this = Arc::clone(&self);
            async move { this.handle_socket(socket).await }
        })
    }

    async fn handle_socket(&self, mut socket: WebSocket) {
        while let Some(msg) = socket.recv().await {
            if let Ok(msg) = msg {
                match msg {
                    Message::Text(t) => {
                        self.logger.log(format!("recieved text data {t:?}"), "");

                        match serde_json::from_str::<ClientPacket>(&t) {
                            Ok(json) => {
                                self.logger
                                    .log(format!("recieved client packet {json:?}!"), "");
                                match self.handle_client_packet(json, &mut socket).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        self.logger.err(
                                            format!("failed to handle client packet: {e}!"),
                                            "",
                                        );
                                        break;
                                    }
                                };
                            }
                            Err(e) => {
                                self.logger.err(format!("recieved invalid json {e}!"), "~!");
                                match socket
                                    .send(Message::Close(Some(CloseFrame {
                                        code: 1007,
                                        reason: Utf8Bytes::from(e.to_string()),
                                    })))
                                    .await
                                {
                                    Ok(_) => {
                                        self.logger.log_style(
                                            "sent close frame with code 1007",
                                            Style::new().bright_yellow(),
                                            "",
                                        );

                                        break;
                                    }
                                    Err(e) => {
                                        self.logger
                                            .err(format!("failed to send close frame: {e}!"), "~!");
                                        break;
                                    }
                                };
                            }
                        }
                    }
                    Message::Binary(b) => {
                        self.logger.log(format!("recieved binary data {b:?}"), "");
                    }
                    Message::Ping(p) => {
                        self.logger.log(format!("recieved ping {p:?}"), "");
                        if socket.send(Message::Pong(p)).await.is_err() {
                            self.logger.err(format!("failed to send pong!"), "~!");
                            break;
                        }
                    }
                    Message::Pong(_) => {
                        self.logger.log("recieved pong, ignoring", "");
                    }
                    Message::Close(_) => {
                        self.logger.log("recieved close, ignoring", "");
                    }
                }
            } else {
                self.logger.err(
                    format!("failed to read msg: {}!", unsafe {
                        msg.unwrap_err_unchecked()
                    },),
                    "~!",
                );
            }
        }

        self.logger.log("connection closed", "");
    }
}

impl BoaWsRoute {
    async fn handle_client_packet(
        &self,
        packet: ClientPacket,
        socket: &mut WebSocket,
    ) -> Result<(), String> {
        match packet {
            ClientPacket::ControlSignal(packet) => {}

            ClientPacket::Open(packet) => {
                let mut state = self.server_state.lock().await;

                let (container_id, container) =
                    BoaContainer::new(&state.docker, state.container_prefix.clone()).await?;

                state.containers.insert(container_id.clone(), container);

                let packet =
                    ServerPacket::ProcessOpenResult(ProcessOpenResultPacket { container_id });

                let serialized_packet = serde_json::to_string::<ServerPacket>(&packet)
                    .map_err(|e| format!("failed to serialize packet: {e}"))?;

                self.logger.log(
                    format!("sending packet: {}", serialized_packet.to_string()),
                    "",
                );

                socket
                    .send(Message::Text(Utf8Bytes::from(serialized_packet)))
                    .await
                    .map_err(|e| format!("failed to send open result packet: {e}"))?;
            }
            ClientPacket::Close(client_packet) => {
                let mut state = self.server_state.lock().await;

                if state.containers.contains_key(&client_packet.container_id) {
                    state.containers.remove(&client_packet.container_id);

                    self.logger.log(
                        format!(
                            "recieved packet to stop close {}",
                            client_packet.container_id.bold()
                        ),
                        "",
                    );

                    let success = state
                        .docker
                        .remove_container(
                            &client_packet.container_id,
                            Some(RemoveContainerOptions::default()),
                        )
                        .await
                        .is_ok();

                    let packet = serde_json::to_string(&ServerPacket::ProcessCloseResult(
                        ProcessCloseResultPacket { success },
                    ))
                    .map_err(|e| format!("failed to serialize packet: {e}"))?;

                    socket
                        .send(Message::Text(Utf8Bytes::from(packet)))
                        .await
                        .map_err(|e| format!("failed to send error packet: {e}"))?;
                } else {
                    let err_packet =
                        serde_json::to_string(&ServerPacket::ServerError(ServerErrorPacket {
                            err: ServerError::InvalidContainerId,
                            message: format!(
                                "container with id {} was not found",
                                client_packet.container_id
                            ),
                        }))
                        .map_err(|e| format!("failed to serialize packet: {e}"))?;

                    self.logger.log_style(
                        format!(
                            "container with id {} was not found",
                            client_packet.container_id.bold()
                        ),
                        Style::new().bright_yellow(),
                        "~?",
                    );

                    socket
                        .send(Message::Text(Utf8Bytes::from(err_packet)))
                        .await
                        .map_err(|e| format!("failed to send error packet: {e}"))?;
                }
            }
        }
        Ok(())
    }
}
