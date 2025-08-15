use std::env;

use tungstenite::{Error, Message, Result, connect};

use crate::server::GameServer;

pub mod match_maker;
pub mod server;
/// A WebSocket echo server
fn main() {
    let args: Vec<String> = env::args().collect();
    let url: String;
    let gs = GameServer::new();
    if args.len() <= 1 {
        // start server
        url = gs.start().to_string();
    } else {
        url = args[1].to_string();
    }
    let (mut socket, _) = connect(format!("ws://{url}")).expect("Cannot connect to game server");
    match socket.read() {
        Ok(msg) => {}
        Err(_) => {}
    }
    socket.close(None).expect("Error closing socket");
}
