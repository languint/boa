use axum::{
    extract::ws::{CloseFrame, Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use axum_extra::{TypedHeader, headers::UserAgent};
use boa_core::packets::client::ClientPacket;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<UserAgent>>,
) -> impl IntoResponse {
    if let Some(TypedHeader(user_agent)) = user_agent {
        println!("`{}` connected", user_agent.as_str());
    }

    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            match msg {
                Message::Text(t) => {
                    println!("[boa-server~/ws]: recieved text data {t:?}");

                    match serde_json::from_str::<ClientPacket>(&t) {
                        Ok(json) => {
                            println!("[boa-server~/ws]: recieved client packet {json:?}");
                        }
                        Err(e) => {
                            eprintln!("[boa-server~/ws]: recieved invalid json: {e}!");
                            match socket
                                .send(Message::Close(Some(CloseFrame {
                                    code: 1007,
                                    reason: Utf8Bytes::from(e.to_string()),
                                })))
                                .await
                            {
                                Ok(_) => {
                                    println!("[boa-server~/ws]: sent close frame with code 400");
                                    break;
                                }
                                Err(e) => {
                                    eprintln!("[boa-server~/ws]: failed to send close frame: {e}!");
                                    break;
                                }
                            };
                        }
                    }
                }
                Message::Binary(b) => {
                    println!("[boa-server~/ws]: recieved binary data {b:?}");
                }
                Message::Ping(p) => {
                    println!("[boa-server~/ws]: recieved ping request {p:?}");
                    if socket.send(Message::Pong(p)).await.is_err() {
                        break;
                    }
                }
                Message::Pong(_) => {
                    println!("[boa-server~/ws]: recieved pong, ignoring");
                }
                Message::Close(_) => {
                    println!("[boa-server~/ws]: recieved close, ignoring");
                }
            }
        } else {
            eprintln!("[boa-server~/ws]: failed to read msg: {}!", unsafe {
                msg.unwrap_err_unchecked()
            });
        }
    }

    println!("[boa-server~/ws]: connection closed");
}
