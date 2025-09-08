use std::error::Error;

use crate::{
    manager_message::{ConnectionDetails, ManagerMessage},
    player::{Player, PlayerStatus},
};
use battleship_models::{self, Coordinates, GameMessage, SelectionCriteria, ServerCommand};
use futures_util::{SinkExt, StreamExt};
use log::debug;
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
    is_running: bool,
    rx_from_manager: mpsc::Receiver<ManagerMessage>,
    player_a: Option<Player>,
    player_b: Option<Player>,
}
impl Lobby {
    pub fn new(id: String, rx_from_manager: mpsc::Receiver<ManagerMessage>) -> Self {
        Self {
            id,
            settings: battleship_models::Settings {
                rows: 8,
                cols: 8,
                ship_count: 4,
            },
            status: GameStatus::Uninitialized,
            is_running: false,
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
        self.is_running = true;
        let id = self.id.clone();
        while self.is_running {
            let server_command = tokio::select! {
                Some(msg) = self.rx_from_manager.recv() => {
                    dbg!("Received manager msg {:#?}", &msg);
                    match msg {
                        ManagerMessage::NewConnection(details) => {
                            let settings = self.settings;
                            if let Some(player) = self.join(details.player_name, details.tx, details.rx).as_mut() {
                                player.status = PlayerStatus::Initialized;
                                battleship_models::ServerCommand::InitializeGame(settings)
                            } else {
                                battleship_models::ServerCommand::Text(String::from(""))
                            }
                        },
                        ManagerMessage::Shutdown => {
                            self.is_running = false;
                            battleship_models::ServerCommand::Text(String::from("Shutting down"))
                        }
                    }
                },
                Some(msg) = Self::try_recv(&mut self.player_a), if self.player_a.is_some() => {
                        dbg!("Received player msg {:#?}", &msg);
                        self.handle_player_message(msg).await?
                },
                Some(msg) = Self::try_recv(&mut self.player_b), if self.player_b.is_some() => {
                        dbg!("Received player msg {:#?}", &msg);
                        self.handle_player_message(msg).await?
                },
            };
            self.direct_message(player, server_command).await?;
            let next_state: Option<ServerCommand> = self.progress_lobby_state().await;
            if let Some(state) = next_state {
                self.broadcast(state).await?;
            }
        }
        Ok(())
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
                    let cnt = self.settings.ship_count;
                    self.status = GameStatus::SelectionMode;
                    a.status = PlayerStatus::Selecting(cnt);
                    b.status = PlayerStatus::Selecting(cnt);
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

    async fn direct_message(&self, player: &Player, data: ServerCommand) -> Result<(), Box<dyn Error>> {
    player
        .tx
        .send(GameMessage {
            id: 1,
            sender: self.id.clone(),
            payload: battleship_models::Payload::ServerCommand(data.clone()),
        })
        .await?;
        Ok(())
    }
    async fn try_recv(o: &mut Option<Player>) -> Option<GameMessage> {
        if let Some(p) = o {
            p.rx.recv().await
        } else {
            None
        }
    }
    pub fn set_settings(mut self, settings: battleship_models::Settings) -> Self {
        self.settings = settings;
        self
    }
    fn get_lobby_status(&self) -> &GameStatus {
        &self.status
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

#[tokio::test]
async fn test_lobby_game_lifecycle() {
    use crate::lobby::Lobby;
    use battleship_models::*;
    use tokio::sync::mpsc;

    // Setup player channels
    let (tx_a, mut rx_a) = mpsc::channel(10);
    let (tx_b, mut rx_b) = mpsc::channel(10);
    let (tx_a_in, rx_a_in) = mpsc::channel(10);
    let (tx_b_in, rx_b_in) = mpsc::channel(10);

    let (tx_from_man, rx_from_man) = mpsc::channel(10);
    // Create lobby
    let handle = tokio::spawn(async move {
        if let Err(_) = Lobby::new(String::from("1"), rx_from_man)
            .set_settings(battleship_models::Settings {
                rows: 2,
                cols: 2,
                ship_count: 1,
            })
            .run()
            .await
        {}
    });
    tx_from_man
        .send(ManagerMessage::NewConnection(ConnectionDetails {
            player_name: "A".to_string(),
            tx: tx_a,
            rx: rx_a_in,
        }))
        .await
        .unwrap();
    tx_from_man
        .send(ManagerMessage::NewConnection(ConnectionDetails {
            player_name: "B".to_string(),
            tx: tx_b,
            rx: rx_b_in,
        }))
        .await
        .unwrap();
    // Simulate game start by sending messages from players
    let msg_a = rx_a.recv().await.unwrap();
    dbg!("msg_a: {:#?}", &msg_a);
    let msg_b = rx_b.recv().await.unwrap();
    dbg!("msg_a: {:#?}", &msg_b);
    assert!(matches!(
        msg_a.payload,
        Payload::ServerCommand(ServerCommand::InitializeGame(_))
    ));
    assert!(matches!(
        msg_b.payload,
        Payload::ServerCommand(ServerCommand::InitializeGame(_))
    ));
    let game_msg_a = GameMessage {
        id: 1,
        sender: "A".to_string(),
        payload: Payload::ClientCommand(ClientCommand::PlaceShip(Coordinates { x: 0, y: 0 })),
    };
    tx_a_in.send(game_msg_a).await.unwrap();
    let game_msg_b = GameMessage {
        id: 2,
        sender: "B".to_string(),
        payload: Payload::ClientCommand(ClientCommand::PlaceShip(Coordinates { x: 1, y: 1 })),
    };
    tx_b_in.send(game_msg_b).await.unwrap();

    // Receive responses for ship placement
    let msg_a = rx_a.recv().await.unwrap();
    dbg!("msg_a: {:#?}", &msg_a);
    let msg_b = rx_b.recv().await.unwrap();
    dbg!("msg_a: {:#?}", &msg_b);
    // assert!(matches!(
    //     msg_a.payload,
    //     Payload::ServerCommand(ServerCommand::Text(_))
    // ));
    // assert!(matches!(
    //     msg_b.payload,
    //     Payload::ServerCommand(ServerCommand::Text(_))
    // ));
    //
    // // Simulate missile launch
    // let missile_msg_a = GameMessage {
    //     id: 3,
    //     sender: "A".to_string(),
    //     payload: Payload::ClientCommand(ClientCommand::LaunchMissle(Coordinates { x: 1, y: 1 })),
    // };
    // tx_a_in.send(missile_msg_a).await.unwrap();
    // let missile_msg_b = GameMessage {
    //     id: 4,
    //     sender: "B".to_string(),
    //     payload: Payload::ClientCommand(ClientCommand::LaunchMissle(Coordinates { x: 0, y: 0 })),
    // };
    // tx_b_in.send(missile_msg_b).await.unwrap();
    //
    // // Receive responses for missile launch
    // let msg_a = rx_a.recv().await.unwrap();
    // let msg_b = rx_b.recv().await.unwrap();
    // assert!(matches!(
    //     msg_a.payload,
    //     Payload::ServerCommand(ServerCommand::Text(_))
    // ));
    // assert!(matches!(
    //     msg_b.payload,
    //     Payload::ServerCommand(ServerCommand::Text(_))
    // ));
    // tx_from_man.send(ManagerMessage::Shutdown).await.unwrap();
    handle.await.unwrap();
    // You can extend this to simulate more of the lifecycle, e.g. selection, turns, game over, etc.
}
