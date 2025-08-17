use log::{info, trace};
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
        trace!("Spawning thread for server");
        spawn(move || {
            GameServer::_start(x);
        });
        &self.url
    }
    /// A WebSocket echo server
    fn _start(url: String) {
        info!("Binding to {}", &url);
        let server = TcpListener::bind(url).unwrap();
        // NOTE: should i keep this maker stored in struct
        loop {
            let mut maker = match_maker::MatchMaker::new();
            let ws_a = GameServer::accept_ws(&server);
            let ws_b = GameServer::accept_ws(&server);
            maker.join(ws_a);
            maker.join(ws_b);
        }
    }
    pub fn stop() {}
    fn accept_ws(server: &TcpListener) -> WebSocket<TcpStream> {
        let (stream, _) = server.accept().unwrap();
        let ws = accept(stream).unwrap();
        trace!("Accepting websocket connection");
        ws
    }
}
