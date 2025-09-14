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
enum NotificationType {
    Broadcast,
    DirectMessage(String),
    NoMessage,
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
        tx: mpsc::Sender<GameMessage>,
        rx: mpsc::Receiver<GameMessage>,
    ) -> &mut Option<Player> {
        let player_slot = if self.player_a.is_none() {
            &mut self.player_a
        } else {
            &mut self.player_b
        };

        *player_slot = Some(Player::new(tx, rx, self.settings.rows, self.settings.cols));

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
            let (notification_type, server_command) = tokio::select! {
                Some(msg) = self.rx_from_manager.recv() => {
                    dbg!("Received manager msg {:#?}", &msg);
                    match msg {
                        ManagerMessage::NewConnection(details) => {
                            let settings = self.settings;
                            if let Some(player) = self.join(details.tx, details.rx).as_mut() {
                                let player_id = player.id.clone();
                                player.status = PlayerStatus::Initialized;
                                (NotificationType::DirectMessage(player_id.clone()),
                                    battleship_models::ServerCommand::InitializeGame(
                                        player_id,
                                        settings
                                    ))
                            } else {
                                (NotificationType::NoMessage, battleship_models::ServerCommand::Text(String::from("")))
                            }
                        },
                        ManagerMessage::Shutdown => {
                            self.is_running = false;
                            (NotificationType::Broadcast, battleship_models::ServerCommand::Text(String::from("Shutting down")))
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
            self.send_message(notification_type, server_command).await?;
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
    ) -> Result<(NotificationType, ServerCommand), Box<dyn Error>> {
        let player_id = msg.sender;

        // Use a match statement to handle different payload types.
        match msg.payload {
            battleship_models::Payload::ClientCommand(command) => {
                // Use a nested match to handle different commands.
                match command {
                    battleship_models::ClientCommand::PlaceShip(Coordinates { x, y }) => {
                        let player = self.get_mut_player(player_id);

                        let is_placed = player.place_ship(y, x)?;
                        // Use if let to concisely check the player's status.
                        if let PlayerStatus::Selecting(cnt) = &mut player.status {
                            if *cnt == 0 {
                                return Err("No ships remaining to place".into());
                            }

                            if is_placed {
                                // Decrement the count directly and place the ship.
                                *cnt -= 1;
                                Ok((
                                    NotificationType::DirectMessage(player.id.clone()),
                                    battleship_models::ServerCommand::Text(String::from(
                                        "Ship placed",
                                    )),
                                ))
                            } else {
                                Err("No ships remaining to place".into())
                            }
                        } else {
                            Err("Player not in selection mode".into())
                        }
                    }
                    battleship_models::ClientCommand::LaunchMissle(Coordinates { x, y }) => {
                        let player = self.get_opposite_mut_player(player_id);
                        let state = player.strike_cell(y, x)?;
                        Ok((
                            NotificationType::Broadcast,
                            battleship_models::ServerCommand::LaunchMissle(
                                state,
                                Coordinates { x, y },
                            ),
                        ))
                    }
                    battleship_models::ClientCommand::SetProfile(_) => {
                        // TODO: send a uuid as a temp username and then remap with this
                        Ok((
                            NotificationType::Broadcast,
                            battleship_models::ServerCommand::SetProfileConfirmation,
                        ))
                    }
                }
            }
            _ => Err("".into()),
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
                    Some(ServerCommand::SelectionMode(SelectionCriteria {
                        count: cnt,
                    }))
                }
                _ => None,
            },
            GameStatus::SelectionMode => {
                if let (&PlayerStatus::Selecting(0), &PlayerStatus::Selecting(0)) =
                    (&a.status, &b.status)
                {
                    self.status = GameStatus::PlayerTurn(a_name.clone());
                    a.status = PlayerStatus::Deciding(true); // true means missle loadec
                    Some(ServerCommand::PlayerTurn(a_name))
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
                } else if let PlayerStatus::Deciding(false) = bomber.status {
                    // false indicates missle fired
                    self.status = GameStatus::PlayerTurn(bombed_player.name.clone());
                    bombed_player.status = PlayerStatus::Deciding(true);
                    Some(ServerCommand::PlayerTurn(bombed_player.name.clone()))
                } else {
                    None
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

    async fn direct_message(
        &self,
        player: &Player,
        data: ServerCommand,
    ) -> Result<(), Box<dyn Error>> {
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

    async fn send_message(
        &self,
        notification_type: NotificationType,
        data: ServerCommand,
    ) -> Result<(), Box<dyn Error>> {
        match notification_type {
            NotificationType::Broadcast => {
                return self.broadcast(data).await;
            }
            NotificationType::DirectMessage(id) => {
                let player = self.get_player(id)?;
                return self.direct_message(player, data).await;
            }
            NotificationType::NoMessage => {
                return Ok(());
            }
        };
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
    fn get_mut_player(&mut self, id: String) -> &mut Player {
        assert!(self.player_a.is_some() && self.player_b.is_some());
        match (&mut self.player_a, &mut self.player_b) {
            (Some(player_a), Some(player_b)) => {
                if player_a.id == id {
                    player_a
                } else {
                    player_b
                }
            }
            _ => panic!("Players are not initialized."),
        }
    }
    fn get_player(&self, id: String) -> Result<&Player, Box<dyn Error>> {
        if let Some(player) = &self.player_a
            && id == player.id
        {
            Ok(player)
        } else if let Some(player) = &self.player_b
            && id == player.id
        {
            Ok(player)
        } else {
            Err("Player not found".into())
        }
    }
    fn get_opposite_mut_player(&mut self, id: String) -> &mut Player {
        assert!(self.player_a.is_some() && self.player_b.is_some());
        match (&mut self.player_a, &mut self.player_b) {
            (Some(player_a), Some(player_b)) => {
                if player_a.id != id {
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
        if let Err(r) = Lobby::new(String::from("1"), rx_from_man)
            .set_settings(battleship_models::Settings {
                rows: 2,
                cols: 2,
                ship_count: 1,
            })
            .run()
            .await
        {
            dbg!("Error occured in test lobby {:?}", r);
        }
    });
    dbg!("Lobby started");
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
        Payload::ServerCommand(ServerCommand::InitializeGame(_, _))
    ));
    assert!(matches!(
        msg_b.payload,
        Payload::ServerCommand(ServerCommand::InitializeGame(_, _))
    ));
    let a_id = if let Payload::ServerCommand(ServerCommand::InitializeGame(id, _)) = msg_a.payload {
        id
    } else {
        "A".to_string()
    };
    let b_id = if let Payload::ServerCommand(ServerCommand::InitializeGame(id, _)) = msg_b.payload {
        id
    } else {
        "A".to_string()
    };
    // ensure new state is selection mode
    let msg_a = rx_a.recv().await.unwrap();
    dbg!("msg_a: {:#?}", &msg_a);
    let msg_b = rx_b.recv().await.unwrap();
    dbg!("msg_a: {:#?}", &msg_b);
    assert!(matches!(
        msg_a.payload,
        Payload::ServerCommand(ServerCommand::SelectionMode(_))
    ));
    assert!(matches!(
        msg_b.payload,
        Payload::ServerCommand(ServerCommand::SelectionMode(_))
    ));

    let game_msg_a = GameMessage {
        id: 1,
        sender: a_id.clone(),
        payload: Payload::ClientCommand(ClientCommand::PlaceShip(Coordinates { x: 0, y: 0 })),
    };
    tx_a_in.send(game_msg_a).await.unwrap();
    let game_msg_b = GameMessage {
        id: 2,
        sender: b_id.clone(),
        payload: Payload::ClientCommand(ClientCommand::PlaceShip(Coordinates { x: 1, y: 1 })),
    };
    tx_b_in.send(game_msg_b).await.unwrap();

    // Receive responses for ship placement
    let msg_a = rx_a.recv().await.unwrap();
    dbg!("msg_a: {:#?}", &msg_a);
    let msg_b = rx_b.recv().await.unwrap();
    dbg!("msg_a: {:#?}", &msg_b);
    assert!(matches!(
        msg_a.payload,
        Payload::ServerCommand(ServerCommand::Text(_))
    ));
    assert!(matches!(
        msg_b.payload,
        Payload::ServerCommand(ServerCommand::Text(_))
    ));
    let msg_a = rx_a.recv().await.unwrap();
    dbg!("msg_a: {:#?}", &msg_a);
    let msg_b = rx_b.recv().await.unwrap();
    dbg!("msg_a: {:#?}", &msg_b);
    assert!(matches!(
        msg_a.payload,
        Payload::ServerCommand(ServerCommand::PlayerTurn(_))
    ));
    assert!(matches!(
        msg_b.payload,
        Payload::ServerCommand(ServerCommand::PlayerTurn(_))
    ));

    // Simulate missile launch
    let missile_msg_a = GameMessage {
        id: 3,
        sender: a_id.clone(),
        payload: Payload::ClientCommand(ClientCommand::LaunchMissle(Coordinates { x: 1, y: 1 })),
    };
    tx_a_in.send(missile_msg_a).await.unwrap();
    let missile_msg_b = GameMessage {
        id: 4,
        sender: b_id.clone(),
        payload: Payload::ClientCommand(ClientCommand::LaunchMissle(Coordinates { x: 0, y: 0 })),
    };
    tx_b_in.send(missile_msg_b).await.unwrap();

    // Receive responses for missile launch
    let msg_a = rx_a.recv().await.unwrap();
    dbg!("msg_a: {:#?}", &msg_a);
    let msg_b = rx_b.recv().await.unwrap();
    dbg!("msg_a: {:#?}", &msg_b);
    assert!(matches!(
        msg_a.payload,
        Payload::ServerCommand(ServerCommand::LaunchMissle(_, _))
    ));
    assert!(matches!(
        msg_b.payload,
        Payload::ServerCommand(ServerCommand::LaunchMissle(_, _))
    ));
    tx_from_man.send(ManagerMessage::Shutdown).await.unwrap();
    handle.await.unwrap();
    // You can extend this to simulate more of the lifecycle, e.g. selection, turns, game over, etc.
}
