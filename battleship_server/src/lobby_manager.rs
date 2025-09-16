use std::{collections::HashMap, sync::Arc};

use battleship_models::GameMessage;
use futures_util::{SinkExt, StreamExt, lock::Mutex};
use log::info;
use tokio::{
    net::TcpListener,
    sync::{mpsc, oneshot},
};
use tokio_tungstenite::tungstenite;

use crate::{
    lobby::Lobby,
    manager_message::{ConnectionDetails, ManagerMessage},
};

pub struct LobbyManager {
    url: String,
    pub lobbies: HashMap<String, mpsc::Sender<ManagerMessage>>,
}

impl LobbyManager {
    pub fn new() -> Self {
        Self {
            url: "127.0.0.1:9001".to_string(),
            lobbies: HashMap::new(),
        }
    }
    /// A WebSocket echo server
    pub async fn start(self) {
        let url = self.url.to_string();
        let mut lobbies = self.lobbies;
        info!("Binding to {}", &url);
        let server = TcpListener::bind(url).await.unwrap();
        let mut id: Option<String> = None;
        let mut players = 0;

        while let Ok((raw_stream, _addr)) = server.accept().await {
            id = if let None = &id {
                Some(uuid::Uuid::new_v4().to_string())
            } else {
                id
            };
            let id_clone = id.as_ref().unwrap().clone();
            let tx_to_lobby_from_manager = lobbies.entry(id_clone.clone()).or_insert_with(|| {
                let (tx_to_lobby, rx_from_manager) = mpsc::channel(100);
                // task for the lobby and game kgic
                tokio::spawn(async move {
                    if let Err(_) = Lobby::new(id_clone, rx_from_manager).run().await {}
                });
                tx_to_lobby
            });
            let (tx_to_lobby_from_task, rx_from_task) = mpsc::channel(100);
            let (tx_to_task, mut rx_from_lobby) = mpsc::channel::<GameMessage>(100);
            if let Err(_) = tx_to_lobby_from_manager
                .send(ManagerMessage::NewConnection(ConnectionDetails {
                    player_name: String::from("In manager"),
                    tx: tx_to_task,
                    rx: rx_from_task,
                }))
                .await
            {}
            players += 1;
            if players == 2 {
                players = 0;
                id = None;
            }
            // task for handling the websocket
            tokio::spawn(async move {
                let ws_stream = tokio_tungstenite::accept_async(raw_stream).await.unwrap();
                let (mut tx, mut rx) = ws_stream.split();
                loop {
                    tokio::select! {
                        Some(Ok(tungstenite::Message::Text(tung_msg))) = rx.next() => {
                            let s = tung_msg.to_string();
                            if let Ok(msg) = serde_json::from_str::<GameMessage>(&s) {
                                if
                                let Err(_) = tx_to_lobby_from_task.send(msg).await {}
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
