use axum::{
    extract::ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use axum_extra::{TypedHeader, headers::UserAgent};

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

                    if let Err(e) = socket
                        .send(Message::Text(Utf8Bytes::from(format!("Echo: {}", t))))
                        .await
                    {
                        eprintln!("[boa-server~/ws]: connection error: {e}!");
                        break;
                    }
                }
                Message::Binary(b) => {
                    println!("[boa-server~/ws]: recieved binary data {b:?}")
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
