use std::{collections::HashMap, sync::Arc};

use battleship_models::Message;
use futures_util::{SinkExt, StreamExt, lock::Mutex};
use log::info;
use tokio::{net::TcpListener, sync::mpsc};
use tokio_tungstenite::tungstenite;

use crate::lobby::Lobby;

pub struct LobbyManager {
    url: String,
    lobbies: Arc<Mutex<HashMap<String, mpsc::Sender<Message>>>>,
}

impl LobbyManager {
    fn _new() -> Self {
        Self {
            url: "127.0.0.1:9001".to_string(),
            lobbies: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    /// A WebSocket echo server
    pub async fn start() {
        let s = Self::_new();
        let url = s.url.to_string();
        let lobbies = Arc::clone(&s.lobbies);
        info!("Binding to {}", &url);
        let server = TcpListener::bind(url).await.unwrap();

        while let Ok((raw_stream, _addr)) = server.accept().await {
            let (tx_to_client, mut rx_from_lobby) = mpsc::channel(100);

            let id = String::from("id"); // TODO: get this thru a msg?
            let id_clone = id.clone();
            let mut lobbies_guard = lobbies.lock().await;
            let tx_to_lobby = lobbies_guard.entry(id).or_insert_with(|| {
                let (tx_to_lobby, rx_from_client) = mpsc::channel(100);
                // task for the lobby and game kgic
                tokio::spawn(async move {
                    Lobby::new(id_clone, rx_from_client, tx_to_client).run();
                });
                tx_to_lobby
            });
            // task for handkking the websocker
            let tx_clone = tx_to_lobby.clone();
            tokio::spawn(async move {
                let ws_stream = tokio_tungstenite::accept_async(raw_stream).await.unwrap();
                let (mut tx, mut rx) = ws_stream.split();
                loop {
                    tokio::select! {
                        Some(Ok(tungstenite::Message::Text(tung_msg))) = rx.next() => {
                            let s = tung_msg.to_string();
                            if let Ok(msg) = serde_json::from_str::<Message>(&s) {
                                if let Err(_) = tx_clone.send(msg).await {}
                            } else {}
                        },
                        Some(bs_msg) = rx_from_lobby.recv() => {
                            if let Ok(json) = serde_json::to_string(&bs_msg) {
                                let tung_msg = tungstenite::protocol::Message::text(json);
                                if let Err(_) = tx.send(tung_msg).await {}
                            } else {}
                        },
                        else => break
                    }
                }
            });
        }
    }
    // TODO: send an id through  channel to remove the lobby from mp of lobbies?
    pub async fn stop() {}
}
