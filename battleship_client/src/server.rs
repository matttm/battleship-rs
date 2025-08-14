use std::net::TcpListener;
use std::thread::spawn;
use tungstenite::accept;

use crate::match_maker;

pub struct GameServer {
    match_maker: match_maker::MatchMaker,
}

impl GameServer {
    pub fn new() -> Self {
        return Self {};
    }
    /// A WebSocket echo server
    pub fn start(&self) {
        let server = TcpListener::bind("127.0.0.1:9001").unwrap();
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
