use boa_core::packets::client::ClientPacket;
use boa_core::packets::client::process::{
    ProcessControlSignal, ProcessControlSignalPacket, ProcessOpenPacket,
};
use boa_core::packets::server::ServerPacket;
use futures_channel::mpsc::{UnboundedSender, unbounded};
use futures_util::{SinkExt, StreamExt};
use gloo_console::console;
use gloo_net::websocket::{Message, futures::WebSocket};

use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

type WsTx = UnboundedSender<Message>;

#[component]
fn App() -> Html {
    let log = use_state(Vec::<String>::new);
    let ws_tx = use_state(|| None::<WsTx>);
    let container_id = use_state(|| None::<String>);

    let open = {
        let ws_tx = ws_tx.clone();

        Callback::from(move |_| {
            let Some(tx) = (*ws_tx).clone() else { return };

            let pkt = ClientPacket::ProcessOpen(ProcessOpenPacket {});

            tx.unbounded_send(Message::Text(serde_json::to_string(&pkt).unwrap()))
                .unwrap();
        })
    };

    let connect = {
        let ws_tx = ws_tx.clone();

        let container_id = container_id.clone();

        Callback::from(move |_: MouseEvent| {
            let container_id_reader = container_id.clone();

            let ws = WebSocket::open("ws://localhost:4040/ws").unwrap();
            let (mut write, mut read) = ws.split();

            let (tx, mut rx) = unbounded::<Message>();
            ws_tx.set(Some(tx.clone()));

            spawn_local(async move {
                while let Some(msg) = rx.next().await {
                    let _ = write.send(msg).await;
                }
            });

            spawn_local(async move {
                while let Some(Ok(Message::Text(txt))) = read.next().await {
                    if let Ok(pkt) = serde_json::from_str::<ServerPacket>(&txt) {
                        match pkt {
                            ServerPacket::ProcessOpenResult(res) => {
                                container_id_reader.set(Some(res.container_id.clone()));
                                console!(format!("Opened container {}", res.container_id));
                            }
                            ServerPacket::ProcessEvent(ev) => {
                                console!(format!("Process event: {:?}", ev));
                            }
                            ServerPacket::ServerError(err) => {
                                console!(format!("Error {:?}: {}", err.err, err.message));
                            }
                            other => {
                                console!(format!("Packet: {:?}", other));
                            }
                        }
                    }
                }
            });
        })
    };

    let upload = {
        let ws_tx = ws_tx.clone();
        let container_id = container_id.clone();

        Callback::from(move |_| {
            let Some(tx) = (*ws_tx).clone() else { return };
            let Some(cid) = (*container_id).clone() else {
                return;
            };

            let code = b"print('hello from container')\n";

            let start = ClientPacket::UploadStart {
                container_id: cid.clone(),
                path: "main.py".into(),
                size: code.len() as u64,
            };

            tx.unbounded_send(Message::Text(serde_json::to_string(&start).unwrap()))
                .unwrap();
            tx.unbounded_send(Message::Bytes(code.to_vec())).unwrap();
            tx.unbounded_send(Message::Text(
                serde_json::to_string(&ClientPacket::UploadFinish { container_id: cid }).unwrap(),
            ))
            .unwrap();
        })
    };

    let start = {
        let ws_tx = ws_tx.clone();
        let container_id = container_id.clone();

        Callback::from(move |_| {
            let Some(tx) = (*ws_tx).clone() else { return };
            let Some(cid) = (*container_id).clone() else {
                return;
            };

            let pkt = ClientPacket::ProcessControlSignal(ProcessControlSignalPacket {
                container_id: cid,
                control_signal: ProcessControlSignal::Start,
            });

            tx.unbounded_send(Message::Text(serde_json::to_string(&pkt).unwrap()))
                .unwrap();
        })
    };

    let exec = {
        let ws_tx = ws_tx.clone();
        let container_id = container_id.clone();

        Callback::from(move |_: MouseEvent| {
            let Some(tx) = (*ws_tx).clone() else { return };
            let Some(cid) = (*container_id).clone() else {
                return;
            };

            let pkt = ClientPacket::ProcessControlSignal(ProcessControlSignalPacket {
                container_id: cid,
                control_signal: ProcessControlSignal::Exec("main.py".into()),
            });

            tx.unbounded_send(Message::Text(serde_json::to_string(&pkt).unwrap()))
                .unwrap();
        })
    };

    html! {
        <div style="font-family: monospace; padding: 1rem;">
            <h2>{ "Boa Python Runner Test UI" }</h2>

            <button onclick={connect}>{ "Connect" }</button>
            <button onclick={open}>{ "Open" }</button>
            <button onclick={upload}>{ "Upload main.py" }</button>
            <button onclick={start}>{ "Start Process" }</button>
            <button onclick={exec}>{ "Exec" }</button>

            <pre style="margin-top: 1rem; background: #111; color: #0f0; padding: 1rem;">
                { for log.iter().map(|l| html! { <div>{l}</div> }) }
            </pre>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
