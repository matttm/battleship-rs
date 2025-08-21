use std::env;

use battleship_models::Settings;
use env_logger::{Builder, Target};
use log::info;

pub mod match_maker;
pub mod player;
pub mod server;
/// A WebSocket echo server
#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let url: String;
    // Initialize env_logger to output to stdout with info level
    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout); // Set the output target to stdout
    builder.filter_level(log::LevelFilter::Info); // Set the minimum log level to info
    builder.init();
    info!("Initializing BATTLESHIP");
    url = args[1].to_string();
    info!("Connecting to server");
    let (mut socket, _) = connect(format!("ws://{url}")).expect("Cannot connect to game server");
    info!("Connection established");
    match socket.read() {
        Ok(Message::Text(msg)) => {
            let json = msg.to_string();
        }
        Ok(_) => {}
        Err(_) => {}
    }
    socket.close(None).expect("Error closing socket");
}
