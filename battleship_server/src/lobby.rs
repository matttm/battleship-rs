use crate::{player::Player, server_messages::ServerMessage};
use battleship_models::{self, Message, SelectionCriteria};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::{net::TcpStream, sync::mpsc};

pub struct Lobby {
    // TODO: give the lobby and lobby manager a channel to communicate
    id: String,
    settings: battleship_models::Settings,
    rx_from_manager: mpsc::Receiver<ServerMessage>,
    player_a: Option<Player>,
    player_b: Option<Player>,
}
impl Lobby {
    pub fn new(id: String, rx_from_manager: mpsc::Receiver<ServerMessage>) -> Self {
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
        tx: mpsc::Sender<Message>,
        rx: mpsc::Receiver<Message>,
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
                        ServerMessage::NewConnection(details) => {
                            self.join(details.player_id, details.tx, details.rx);
                        },
                    }
                },
                Some(msg) = Self::try_recv(&mut self.player_a), if self.player_a.is_some() => {},
            }
        }
    }
    async fn try_recv(o: &mut Option<Player>) -> Option<Message> {
        if let Some(p) = o {
            p.rx.recv().await
        } else {
            None
        }
    }
}
