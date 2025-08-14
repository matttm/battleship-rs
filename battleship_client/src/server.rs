use std::net::TcpListener;
use std::thread::spawn;
use tungstenite::accept;

use crate::match_maker::MatchMaker;

pub struct GameServer {
    url: String,
    match_maker: MatchMaker,
}

impl GameServer {
    pub fn new() -> Self {
        return Self {
            url: "127.0.0.1:9001".to_string(),
            match_maker: MatchMaker::new(),
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
        for stream in server.incoming() {
            spawn(move || {
                let mut websocket = accept(stream.unwrap()).unwrap();
                loop {
                    let msg = websocket.read().unwrap();

                    // We do not want to send back ping/pong messages.
                    if msg.is_binary() || msg.is_text() {
                        websocket.send(msg).unwrap();
                    }
                }
            });
        }
    }
    pub fn stop() {}
}
