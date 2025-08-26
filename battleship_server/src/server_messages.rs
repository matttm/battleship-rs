use battleship_models::Message;
use tokio::sync::mpsc;

pub struct ConnectionDetails {
    pub player_id: String,
    pub tx: mpsc::Sender<Message>,
    pub rx: mpsc::Receiver<Message>,
}

pub enum ServerMessage {
    NewConnection(ConnectionDetails),
}
