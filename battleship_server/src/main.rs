pub mod lobby;
pub mod lobby_manager;
pub mod manager_message;
pub mod player;

use crate::lobby_manager::LobbyManager;

#[tokio::main]
async fn main() {
    LobbyManager::new().start().await;
}
