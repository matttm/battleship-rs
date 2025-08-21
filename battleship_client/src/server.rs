use futures_util::lock::Mutex;
use log::{error, info};
use std::{collections::HashMap, sync::Arc, thread::spawn};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::WebSocketStream;

use crate::match_maker::{self, MatchMaker};

pub struct GameServer {
    url: String,
    lobbies: Arc<Mutex<HashMap<String, Arc<Mutex<MatchMaker>>>>>,
}

impl GameServer {
    pub fn new() -> Self {
        Self {
            url: "127.0.0.1:9001".to_string(),
            lobbies: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    pub fn start(&self) -> &String {
        let x = self.url.to_string();
        info!("Spawning thread for server");
        tokio::spawn(GameServer::_start(x, self.lobbies.clone()));
        &self.url
    }
    /// A WebSocket echo server
    async fn _start(url: String, lobbies: Arc<Mutex<HashMap<String, Arc<Mutex<MatchMaker>>>>>) {
        info!("Binding to {}", &url);
        let server = TcpListener::bind(url).await.unwrap();
        let mut current_lobby: Option<Arc<Mutex<MatchMaker>>> = None;

        while let Ok((raw_stream, _addr)) = server.accept().await {
            let ws_stream = tokio_tungstenite::accept_async(raw_stream).await.unwrap();

            let lobby_to_use = match current_lobby {
                Some(ref lobby) => {
                    let is_full = {
                        let lobby_guard = lobby.lock().await;
                        lobby_guard.is_lobby_full()
                    };
                    if is_full {
                        let new_lobby = Arc::new(Mutex::new(MatchMaker::new(String::from(""))));
                        current_lobby = Some(Arc::clone(&new_lobby));
                        new_lobby
                    } else {
                        Arc::clone(lobby)
                    }
                }
                None => {
                    let new_lobby = Arc::new(Mutex::new(MatchMaker::new(String::from(""))));
                    current_lobby = Some(Arc::clone(&new_lobby));
                    new_lobby
                }
            };

            // Lock the lobby_to_use to call the join method.
            let mut lobby_guard = lobby_to_use.lock().await;
            if let Err(e) = lobby_guard.join(ws_stream).await {
                eprintln!("Error joining lobby: {:?}", e);
            }

            // Lock the lobbies map and insert the lobby.
            let mut map = lobbies.lock().await;
            map.insert(lobby_guard.get_id().to_string(), Arc::clone(&lobby_to_use));
        }
    }
    // TODO: send an id through  channel to remove the lobby from mp of lobbies?
    pub async fn stop() {}
}
