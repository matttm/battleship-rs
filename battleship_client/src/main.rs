use std::env;

use env_logger::{Builder, Target};
use futures_util::{SinkExt, StreamExt};
use log::info;
use tokio_tungstenite::tungstenite::Message;

pub mod app;
pub mod event;
pub mod ui;
pub mod widgets;

use app::App;

/// A WebSocket echo server
#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    // let args: Vec<String> = env::args().collect();
    // let url: String;
    // // Initialize env_logger to output to stdout with info level
    // let mut builder = Builder::from_default_env();
    // builder.target(Target::Stdout); // Set the output target to stdout
    // builder.filter_level(log::LevelFilter::Info); // Set the minimum log level to info
    // builder.init();
    // info!("Initializing BATTLESHIP");
    // url = args[1].to_string();
    // info!("Connecting to server");
    // let (mut socket, _) = tokio_tungstenite::connect_async(format!("ws://{url}"))
    //     .await
    //     .expect("Cannot connect to game server");
    // let (tx, rx) = socket.split();
    // info!("Connection established");
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal).await;
    ratatui::restore();
    result
}
