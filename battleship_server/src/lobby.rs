use std::error::Error;

use crate::{
    manager_message::ManagerMessage,
    player::{Player, PlayerStatus},
};
use battleship_models::{self, Coordinates, GameMessage, SelectionCriteria, ServerCommand};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;

enum GameStatus {
    Uninitialized,
    SelectionMode,
    PlayerTurn(String), // whose turn it is
    GameOver,
}

pub struct Lobby {
    // TODO: give the lobby and lobby manager a channel to communicate
    id: String,
    settings: battleship_models::Settings,
    status: GameStatus,
    rx_from_manager: mpsc::Receiver<ManagerMessage>,
    player_a: Option<Player>,
    player_b: Option<Player>,
}
impl Lobby {
    pub fn new(id: String, rx_from_manager: mpsc::Receiver<ManagerMessage>) -> Self {
        Self {
            id,
            settings: battleship_models::Settings { rows: 8, cols: 8 },
            status: GameStatus::Uninitialized,
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
    ) -> &mut Option<Player> {
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
                            if let Some(player) = self.join(details.player_id, details.tx, details.rx).as_mut() {
                                player.status = PlayerStatus::Initialized;
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
            self.broadcast(server_command).await?;
            let next_state: Option<ServerCommand> = self.progress_lobby_state().await;
            if let Some(state) = next_state {
                self.broadcast(state).await?;
            }
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
    async fn progress_lobby_state(&mut self) -> Option<ServerCommand> {
        let a = self.player_a.as_mut()?;
        let b = self.player_b.as_mut()?;
        let a_name = a.name.clone();
        let b_name = b.name.clone();
        match &self.status {
            GameStatus::Uninitialized => match (&mut self.player_a, &mut self.player_b) {
                (Some(a), Some(b)) => {
                    self.status = GameStatus::SelectionMode;
                    a.status = PlayerStatus::Selecting(4);
                    b.status = PlayerStatus::Selecting(4);
                    Some(ServerCommand::SelectionMode(SelectionCriteria { count: 4 }))
                }
                _ => None,
            },
            GameStatus::SelectionMode => {
                if let (PlayerStatus::Selecting(x), PlayerStatus::Selecting(y)) =
                    (&a.status, &b.status)
                {
                    if *x == 0 && *y == 0 {
                        self.status = GameStatus::PlayerTurn(a_name.clone());
                        Some(ServerCommand::PlayerTurn(a_name))
                    } else {
                        Some(ServerCommand::Text(format!(
                            "Selection(s) remaining -- {x} - {y}"
                        )))
                    }
                } else {
                    None
                }
            }
            GameStatus::PlayerTurn(name) => {
                // TODO: add a check to see if player launched or not
                // getting player who wasn launched at
                let (bomber, bombed_player) = if a_name == *name { (a, b) } else { (b, a) };
                if bombed_player.ships_alive == 0 {
                    self.status = GameStatus::GameOver;
                    Some(ServerCommand::GameOver)
                } else {
                    self.status = GameStatus::PlayerTurn(bombed_player.name.clone());
                    Some(ServerCommand::PlayerTurn(bombed_player.name.clone()))
                }
            }
            GameStatus::GameOver => None,
        }
    }
    async fn broadcast(&self, data: ServerCommand) -> Result<(), Box<dyn Error>> {
        let players = vec![&self.player_a, &self.player_b];
        let mut iter = players.iter();
        while let Some(Some(player)) = iter.next() {
            // TODO: replace with tokio::broaccast
            player
                .tx
                .send(GameMessage {
                    id: 1,
                    sender: self.id.clone(),
                    payload: battleship_models::Payload::ServerCommand(data.clone()),
                })
                .await?;
        }
        Ok(())
    }
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
