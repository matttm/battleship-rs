pub mod lobby_manager;
pub mod lobby;
pub mod player;

use crate::{lobby_manager::LobbyManager};

#[tokio::main]
async fn main() {
    let server = LobbyManager::new();
    LobbyManager::start(server.url, server.lobbies).await;
}
