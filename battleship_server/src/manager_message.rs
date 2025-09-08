use battleship_models::GameMessage;
use tokio::sync::mpsc;

pub struct ConnectionDetails {
    pub player_id: String,
    pub tx: mpsc::Sender<GameMessage>,
    pub rx: mpsc::Receiver<GameMessage>,
}

pub enum ManagerMessage {
    NewConnection(ConnectionDetails),
    Shutdown,
}
