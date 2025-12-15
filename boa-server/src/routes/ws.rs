use std::sync::Arc;

use axum::{
    extract::ws::{Message, WebSocketUpgrade},
    response::IntoResponse,
};

use boa_core::packets::{
    client::{ClientPacket, process::ProcessControlSignal},
    server::{
        ServerPacket,
        error::{ServerError, ServerErrorPacket},
        process::{ProcessCloseResultPacket, ProcessEventPacket, ProcessOpenResultPacket},
    },
};

use bollard::query_parameters::RemoveContainerOptions;
use futures_util::{SinkExt, StreamExt};

use tokio::{
    io::AsyncWriteExt,
    sync::mpsc::{self, UnboundedSender},
};

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

pub enum WsOutbound {
    Packet(ServerPacket),
    Pong(Vec<u8>),
}

struct UploadState {
    container_id: String,
    temp_file: tempfile::NamedTempFile,
    container_path: String,
    file_name: String,
    remaining: u64,
}

impl BoaWsRoute {
    pub fn ws_handler(self: Arc<Self>, ws: WebSocketUpgrade) -> impl IntoResponse {
        ws.on_upgrade(move |socket| {
            let this = Arc::clone(&self);
            async move {
                let (mut ws_tx, mut ws_rx) = socket.split();

                let (packet_tx, mut packet_rx) = mpsc::unbounded_channel::<WsOutbound>();

                let writer = tokio::spawn(async move {
                    while let Some(msg) = packet_rx.recv().await {
                        match msg {
                            WsOutbound::Packet(packet) => {
                                let Ok(text) = serde_json::to_string(&packet) else {
                                    continue;
                                };
                                if ws_tx.send(Message::Text(text.into())).await.is_err() {
                                    break;
                                }
                            }
                            WsOutbound::Pong(p) => {
                                if ws_tx.send(Message::Pong(p.into())).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                });

                let mut upload_state: Option<UploadState> = None;

                while let Some(msg) = ws_rx.next().await {
                    let Ok(msg) = msg else { break };

                    match msg {
                        Message::Text(t) => {
                            let packet = match serde_json::from_str::<ClientPacket>(&t) {
                                Ok(p) => p,
                                Err(e) => {
                                    let _ = packet_tx.send(WsOutbound::Packet(
                                        ServerPacket::ServerError(ServerErrorPacket {
                                            err: ServerError::InvalidJson,
                                            message: e.to_string(),
                                        }),
                                    ));
                                    break;
                                }
                            };

                            match packet {
                                ClientPacket::UploadStart {
                                    container_id,
                                    path,
                                    size,
                                } => {
                                    if upload_state.is_some() {
                                        let _ = packet_tx.send(WsOutbound::Packet(
                                            ServerPacket::ServerError(ServerErrorPacket {
                                                err: ServerError::UploadAlreadyInProgress,
                                                message: "upload already in progress".into(),
                                            }),
                                        ));
                                        continue;
                                    }

                                    let temp_file = tempfile::NamedTempFile::new()
                                        .map_err(|e| e.to_string())
                                        .expect("Temp file creation should always succeed");

                                    upload_state = Some(UploadState {
                                        container_id,
                                        temp_file,
                                        container_path: "/src".to_string(),
                                        file_name: path,
                                        remaining: size,
                                    });
                                }

                                ClientPacket::UploadFinish { .. } => {
                                    // 1. We take the state here. upload_state becomes None.
                                    if let Some(state) = upload_state.take() {
                                        let docker = self.server_state.lock().await.docker.clone();

                                        // Retrieve the container instance
                                        let container = {
                                            let state_lock = self.server_state.lock().await;
                                            state_lock.containers.get(&state.container_id).cloned()
                                        };

                                        // DELETE THIS BLOCK START: if let Some(state) = upload_state.take() {
                                        // The state is already available in the 'state' variable from the outer if.

                                        // Ensure we have a container before spawning
                                        if let Some(container) = container {
                                            let docker = docker.clone();
                                            // Extract fields from state to move into the async block
                                            let temp_file = state.temp_file;
                                            let container_path = state.container_path;
                                            let file_name = state.file_name;

                                            // dbg!(&temp_file.path());
                                            // dbg!(&container_path);

                                            tokio::spawn(async move {
                                                if let Err(e) = container
                                                    .upload_file(
                                                        &docker,
                                                        temp_file.path(),
                                                        &container_path,
                                                        &file_name, // Use the extracted file_name
                                                    )
                                                    .await
                                                {
                                                    eprintln!("upload failed: {e}");
                                                }
                                                // temp_file is dropped here, deleting the temp file from host
                                            });
                                        } else {
                                            eprintln!("Container not found for upload");
                                        }

                                        // DELETE THIS BLOCK END: }
                                    }
                                }

                                other => {
                                    if let Err(e) =
                                        this.handle_client_packet(other, packet_tx.clone()).await
                                    {
                                        this.logger.err(e, "~!");
                                        break;
                                    }
                                }
                            }
                        }

                        Message::Binary(bytes) => {
                            if let Some(state) = upload_state.as_mut() {
                                let mut file = tokio::fs::OpenOptions::new()
                                    .write(true)
                                    .open(state.temp_file.path())
                                    .await
                                    .expect("Temporary files should always exist");

                                if let Err(e) = file.write_all(&bytes).await {
                                    let _ = packet_tx.send(WsOutbound::Packet(
                                        ServerPacket::ServerError(ServerErrorPacket {
                                            err: ServerError::UploadFailed,
                                            message: e.to_string(),
                                        }),
                                    ));
                                    upload_state = None;
                                    continue;
                                }

                                state.remaining =
                                    state.remaining.saturating_sub(bytes.len() as u64);
                            } else {
                                let _ = packet_tx.send(WsOutbound::Packet(
                                    ServerPacket::ServerError(ServerErrorPacket {
                                        err: ServerError::UnexpectedBinaryFrame,
                                        message: "binary frame without upload".into(),
                                    }),
                                ));
                            }
                        }

                        Message::Ping(p) => {
                            let _ = packet_tx.send(WsOutbound::Pong(p.to_vec()));
                        }

                        Message::Close(_) => break,
                        _ => {}
                    }
                }

                drop(packet_tx);
                let _ = writer.await;
            }
        })
    }
}

impl BoaWsRoute {
    async fn handle_client_packet(
        &self,
        packet: ClientPacket,
        tx: UnboundedSender<WsOutbound>,
    ) -> Result<(), String> {
        match packet {
            ClientPacket::ProcessOpen(_) => {
                let (container_id, container) = {
                    let state = self.server_state.lock().await;
                    BoaContainer::new(&state.docker, state.container_prefix.clone()).await?
                };

                self.server_state
                    .lock()
                    .await
                    .containers
                    .insert(container_id.clone(), container);

                tx.send(WsOutbound::Packet(ServerPacket::ProcessOpenResult(
                    ProcessOpenResultPacket { container_id },
                )))
                .ok();
            }
            ClientPacket::ProcessControlSignal(pkt) => {
                let (mut container, docker) = {
                    let state = self.server_state.lock().await;
                    (
                        state
                            .containers
                            .get(&pkt.container_id)
                            .cloned()
                            .ok_or("invalid container id")?,
                        state.docker.clone(),
                    )
                };

                match pkt.control_signal {
                    ProcessControlSignal::Start => {
                        tx.send(WsOutbound::Packet(ServerPacket::ProcessEvent(
                            ProcessEventPacket::Started,
                        )))
                        .ok();

                        tokio::spawn(async move {
                            match container.start(&docker).await {
                                Err(e) => {
                                    tx.send(WsOutbound::Packet(ServerPacket::ServerError(
                                        ServerErrorPacket {
                                            err: ServerError::ProcessStartFailed,
                                            message: format!("failed to start: {e}"),
                                        },
                                    )))
                                    .ok();
                                }
                                _ => {
                                    tx.send(WsOutbound::Packet(ServerPacket::ProcessEvent(
                                        ProcessEventPacket::Started,
                                    )))
                                    .ok();
                                }
                            }
                        });

                        // tokio::spawn(async move {
                        //     match container
                        //         .run(&docker, "main.py".to_string(), tx.clone())
                        //         .await
                        //     {
                        //         Ok(code) => {
                        //             tx.send(WsOutbound::Packet(ServerPacket::ProcessEvent(
                        //                 ProcessEventPacket::Finished { exit_code: code },
                        //             )))
                        //             .ok();
                        //         }
                        //         Err(e) => {
                        //             tx.send(WsOutbound::Packet(ServerPacket::ServerError(
                        //                 ServerErrorPacket {
                        //                     err: ServerError::ProcessStartFailed,
                        //                     message: e,
                        //                 },
                        //             )))
                        //             .ok();
                        //         }
                        //     }
                        // });
                    }

                    ProcessControlSignal::Exec(file_path) => {
                        let _ = tx.send(WsOutbound::Packet(ServerPacket::ProcessEvent(
                            ProcessEventPacket::Started,
                        )));

                        let docker = docker.clone();
                        tokio::spawn(async move {
                            match container.run(&docker, file_path, tx.clone()).await {
                                Ok(exit_code) => {
                                    let _ =
                                        tx.send(WsOutbound::Packet(ServerPacket::ProcessEvent(
                                            ProcessEventPacket::Finished { exit_code },
                                        )));
                                }
                                Err(e) => {
                                    let _ = tx.send(WsOutbound::Packet(ServerPacket::ServerError(
                                        ServerErrorPacket {
                                            err: ServerError::ProcessStartFailed,
                                            message: e,
                                        },
                                    )));
                                }
                            }
                        });
                    }

                    ProcessControlSignal::Interrupt => {
                        container
                            .signal(&docker, ProcessControlSignal::Interrupt)
                            .await?;
                    }

                    ProcessControlSignal::Terminate => {
                        container
                            .signal(&docker, ProcessControlSignal::Terminate)
                            .await?;
                    }
                }
            }
            ClientPacket::ProcessClose(pkt) => {
                let docker = self.server_state.lock().await.docker.clone();

                self.server_state
                    .lock()
                    .await
                    .containers
                    .remove(&pkt.container_id);

                let success = docker
                    .remove_container(
                        &pkt.container_id,
                        Some(RemoveContainerOptions {
                            force: true,
                            ..Default::default()
                        }),
                    )
                    .await
                    .is_ok();

                tx.send(WsOutbound::Packet(ServerPacket::ProcessCloseResult(
                    ProcessCloseResultPacket { success },
                )))
                .ok();
            }
            ClientPacket::UploadStart { .. } | ClientPacket::UploadFinish { .. } => unreachable!(),
        }

        Ok(())
    }
}
