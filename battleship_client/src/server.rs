use log::{error, info};
use std::net::{TcpListener, TcpStream};
use std::thread::spawn;
use tungstenite::{WebSocket, accept};

use crate::match_maker::{self, MatchMaker};

pub struct GameServer {
    url: String,
}

impl GameServer {
    pub fn new() -> Self {
        return Self {
            url: "127.0.0.1:9001".to_string(),
        };
    }
    pub fn start(&self) -> &String {
        let x = self.url.to_string();
        info!("Spawning thread for server");
        spawn(move || {
            GameServer::_start(x);
        });
        &self.url
    }
    /// A WebSocket echo server
    fn _start(url: String) {
        info!("Binding to {}", &url);
        let server = TcpListener::bind(url).unwrap();
        // NOTE: maybe remove this loop if only doing one lobby
        loop {
            let mut maker = match_maker::MatchMaker::new();
            let ws_a = GameServer::accept_ws(&server);
            // join method sends settings
            if let Err(_) = maker.join(ws_a) {
                error!("Local player cannot join server");
            }
            info!("Waiting for second player");
            let ws_b = GameServer::accept_ws(&server);
            if let Err(_) = maker.join(ws_b) {
                error!("Second player cannot join server");
            }
            // TODO: if i dont remove loop and expect more lobbies, then put run on own thread
            maker.run();
        }
    }
    pub fn stop() {}
    fn accept_ws(server: &TcpListener) -> WebSocket<TcpStream> {
        let (stream, _) = server.accept().unwrap();
        let ws = accept(stream).unwrap();
        info!("Accepting websocket connection");
        ws
    }
}
