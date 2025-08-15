use std::net::TcpListener;
use std::thread::spawn;
use tungstenite::accept;

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
        spawn(move || {
            GameServer::_start(x);
        });
        &self.url
    }
    /// A WebSocket echo server
    fn _start(url: String) {
        let server = TcpListener::bind(url).unwrap();
        // NOTE: should i keep this maker stored in struct
        let maker = match_maker::MatchMaker::new();
        let mut waiting = 0u8;
        for stream in server.incoming() {
            spawn(move || {
                let mut websocket = accept(stream.unwrap()).unwrap();
                waiting += 1u8;
                maker.join(websocket);
                if waiting == 2u8 {
                    maker = match_maker::MatchMaker::new();
                    waiting = 0;
                }
            });
        }
    }
    pub fn stop() {}
}
