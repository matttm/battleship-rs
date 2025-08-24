use std::{collections::HashMap, sync::Arc};

use battleship_models::Message;
use futures_util::lock::Mutex;
use log::info;
use tokio::{net::TcpListener, sync::mpsc};

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
        let mut current_lobby: Option<Arc<Mutex<Lobby>>> = None;

        while let Ok((raw_stream, _addr)) = server.accept().await {
            let ws_stream = tokio_tungstenite::accept_async(raw_stream).await.unwrap();
            let (tx_to_lobby, rx_from_client) = mpsc::channel(100);
            let (tx_to_client, rx_from_lobby) = mpsc::channel(100);

            let id = String::from("id"); // TODO: get this thru a msg?
            let mut lobbies_guard = lobbies.lock().await;
            let tx_clone = tx_to_lobby.clone();
            lobbies_guard.entry(id).or_insert_with(|| tx_clone);
            tokio::spawn(async move || {
                Lobby::new(id, rx_from_client, tx_to_client).run();
            });
        }
    }
    // TODO: send an id through  channel to remove the lobby from mp of lobbies?
    pub async fn stop() {}
}
