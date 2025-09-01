use std::error::Error;

use crate::{manager_message::ManagerMessage, player::Player};
use battleship_models::{
    self, Coordinates, GameMessage, Payload, SelectionCriteria, ServerCommand, Settings,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::{net::TcpStream, sync::mpsc};

struct PlayerState {
    pub ships_to_place: usize,
    pub ships_alive: usize,
}

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
    ) -> &Option<Player> {
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

        player_slot
    }
    pub fn is_lobby_full(&self) -> bool {
        self.player_a.is_some() && self.player_b.is_some()
    }
    pub fn get_id(&self) -> String {
        self.id.to_string()
    }
    pub async fn run(mut self) -> Result<(), Box<dyn Error>> {
        let id = self.id.clone();
        loop {
            let server_command = tokio::select! {
                Some(msg) = self.rx_from_manager.recv() => {
                    match msg {
                        ManagerMessage::NewConnection(details) => {
                            let settings = self.settings;
                            if let Some(player) = self.join(details.player_id, details.tx, details.rx) {
                                battleship_models::ServerCommand::InitializeGame(settings)
                            } else {
                                battleship_models::ServerCommand::Text(String::from(""))
                            }
                        },
                    }
                },
                Some(msg) = Self::try_recv(&mut self.player_a), if self.player_a.is_some() => {
                        Self::handle_player_message(&mut self, msg).await?
                },
                Some(msg) = Self::try_recv(&mut self.player_b), if self.player_b.is_some() => {
                        Self::handle_player_message(&mut self, msg).await?
                },
            };
            player
                .tx
                .send(GameMessage {
                    id: 1,
                    sender: id,
                    payload: battleship_models::Payload::ServerCommand(server_command),
                })
                .await?;
            self.progress_state();
        }
    }
    async fn handle_player_message(
        &mut self,
        msg: GameMessage,
    ) -> Result<ServerCommand, Box<dyn Error>> {
        let data = msg.payload;
        if let battleship_models::Payload::ClientCommand(command) = data {
            // TODO: move set cell fns?
            match command {
                battleship_models::ClientCommand::PlaceShip(Coordinates { x, y }) => {
                    let player = self.get_mut_player(msg.sender);
                    let _ = player.place_ship(y, x)?;
                    Ok(battleship_models::ServerCommand::Text(String::from("")))
                }
                battleship_models::ClientCommand::LaunchMissle(Coordinates { x, y }) => {
                    let player = self.get_opposite_mut_player(msg.sender);
                    let _ = player.strike_cell(y, x)?;
                    Ok(battleship_models::ServerCommand::Text(String::from("")))
                }
            }
        } else {
            Ok(battleship_models::ServerCommand::Text(String::from("")))
        }
    }
    async fn progress_state(&mut self) {}
    async fn try_recv(o: &mut Option<Player>) -> Option<GameMessage> {
        if let Some(p) = o {
            p.rx.recv().await
        } else {
            None
        }
    }
    fn get_mut_player(&mut self, name: String) -> &mut Player {
        assert!(self.player_a.is_some() && self.player_b.is_some());
        match (&mut self.player_a, &mut self.player_b) {
            (Some(player_a), Some(player_b)) => {
                if player_a.name == name {
                    player_a
                } else {
                    player_b
                }
            }
            _ => panic!("Players are not initialized."),
        }
    }
    fn get_opposite_mut_player(&mut self, name: String) -> &mut Player {
        assert!(self.player_a.is_some() && self.player_b.is_some());
        match (&mut self.player_a, &mut self.player_b) {
            (Some(player_a), Some(player_b)) => {
                if player_a.name != name {
                    player_a
                } else {
                    player_b
                }
            }
            _ => panic!("Players are not initialized."),
        }
    }
}
