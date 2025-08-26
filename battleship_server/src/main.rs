pub mod lobby;
pub mod lobby_manager;
pub mod player;
pub mod server_messages;

use crate::lobby_manager::LobbyManager;

#[tokio::main]
async fn main() {
    LobbyManager::start().await;
}
