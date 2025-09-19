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
    info!("Initializing BATTLESHIP");
    let url = "".to_string();
    info!("Connecting to server");
    info!("Connection established");
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().await.run(terminal).await;
    ratatui::restore();
    result
}
