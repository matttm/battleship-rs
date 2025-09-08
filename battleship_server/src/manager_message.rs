use battleship_models::GameMessage;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct ConnectionDetails {
    pub player_name: String,
    pub tx: mpsc::Sender<GameMessage>,
    pub rx: mpsc::Receiver<GameMessage>,
}
#[derive(Debug)]
pub enum ManagerMessage {
    NewConnection(ConnectionDetails),
    Shutdown,
}
