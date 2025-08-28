use crate::{manager_message::ManagerMessage, player::Player};
use battleship_models::{self, GameMessage, GameStates, SelectionCriteria};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::{net::TcpStream, sync::mpsc};

pub struct Lobby {
    // TODO: give the lobby and lobby manager a channel to communicate
    id: String,
    settings: battleship_models::Settings,
    rx_from_manager: mpsc::Receiver<ManagerMessage>,
    player_a: Option<Player>,
    player_b: Option<Player>,
}
impl Lobby {
    pub fn new(id: String, rx_from_manager: mpsc::Receiver<ManagerMessage>) -> Self {
        Self {
            id,
            settings: battleship_models::Settings { rows: 8, cols: 8 },
            rx_from_manager,
            player_a: None,
            player_b: None,
        }
    }
    pub fn join(
        &mut self,
        id: String,
        tx: mpsc::Sender<GameMessage>,
        rx: mpsc::Receiver<GameMessage>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let player_slot = if self.player_a.is_none() {
            &mut self.player_a
        } else {
            &mut self.player_b
        };

        *player_slot = Some(Player::new(
            id,
            tx,
            rx,
            self.settings.rows,
            self.settings.cols,
        ));

        Ok(())
    }
    pub fn is_lobby_full(&self) -> bool {
        self.player_a.is_some() && self.player_b.is_some()
    }
    pub fn get_id(&self) -> String {
        self.id.to_string()
    }
    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                Some(msg) = self.rx_from_manager.recv() => {
                    match msg {
                        ManagerMessage::NewConnection(details) => {
                            if let Err(_) = self.join(details.player_id, details.tx, details.rx) {}
                        },
                    }
                },
                Some(msg) = Self::try_recv(&mut self.player_a), if self.player_a.is_some() => {
                        Self::handle_player_message(msg).await;
                },
                Some(msg) = Self::try_recv(&mut self.player_b), if self.player_b.is_some() => {
                        Self::handle_player_message(msg).await;
                },
            }
        }
    }
    async fn handle_player_message(msg: GameMessage) {
        let data = msg.payload;
        match data {}
    }
    async fn try_recv(o: &mut Option<Player>) -> Option<GameMessage> {
        if let Some(p) = o {
            p.rx.recv().await
        } else {
            None
        }
    }
}
