use std::env;

use env_logger::{Builder, Target};
use log::info;
use tungstenite::{Error, Message, Result, connect};

use crate::server::GameServer;

pub mod match_maker;
pub mod server;
/// A WebSocket echo server
fn main() {
    let args: Vec<String> = env::args().collect();
    let url: String;
    let gs = GameServer::new();
    // Initialize env_logger to output to stdout with info level
    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout); // Set the output target to stdout
    builder.filter_level(log::LevelFilter::Trace); // Set the minimum log level to info
    builder.init();
    info!("Initializing BATTLESHIP");
    if args.len() <= 1 {
        // start server
        url = gs.start().to_string();
        info!("Starting server");
    } else {
        url = args[1].to_string();
        info!("Connecting to server");
    }
    let (mut socket, _) = connect(format!("ws://{url}")).expect("Cannot connect to game server");
    match socket.read() {
        Ok(msg) => {
            info!("Received a message");
        }
        Err(_) => {}
    }
    socket.close(None).expect("Error closing socket");
}
