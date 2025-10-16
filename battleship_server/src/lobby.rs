use std::error::Error;

use crate::{
    manager_message::{ConnectionDetails, ManagerMessage},
    player::{Player, PlayerStatus},
};
use battleship_models::{
    self, Coordinates, GameMessage, GameStatus, SelectionCriteria, ServerCommand,
};
use futures_util::{SinkExt, StreamExt};
use log::info;
use tokio::sync::mpsc;

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
                    info!("Received manager msg");
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
                        info!("Received player_a msg {:?}", &msg.payload);
                        self.handle_player_message(msg).await?
                },
                Some(msg) = Self::try_recv(&mut self.player_b), if self.player_b.is_some() => {
                        info!("Received player_b msg {:?}", &msg.payload);
                        self.handle_player_message(msg).await?
                },
            };
            info!("Handling server command {:?}", server_command);
            if let Err(msg) = self.send_message(notification_type, server_command).await {
                log::error!("{}", msg);
            }
            let next_state: Option<ServerCommand> = self.progress_lobby_state().await;
            if let Some(state) = next_state {
                info!("Progressing lobby produced state {:?}", state);
                self.broadcast(state).await?;
            } else {
                info!("Progressing lobby produced no state emission");
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
                        // TODO: MOVE IN THE IF-statement
                        // Use if let to concisely check the player's status.
                        if let Err(msg) = player.place_ship(y, x) {
                            return Ok((
                                NotificationType::DirectMessage(player.id.clone()),
                                battleship_models::ServerCommand::PlaceBoatError(
                                    player.board[y][x],
                                    Coordinates { x, y },
                                    msg.to_string(),
                                ),
                            ));
                        }
                        Ok((
                            NotificationType::DirectMessage(player.id.clone()),
                            battleship_models::ServerCommand::PlaceBoatConfirmation(Coordinates {
                                x,
                                y,
                            }),
                        ))
                    }
                    battleship_models::ClientCommand::LaunchMissle(Coordinates { x, y }) => {
                        let player_id_clone = player_id.clone();
                        let target_player = self.get_opposite_mut_player(player_id);
                        let state = target_player.strike_cell(y, x)?;

                        // Decrement ships_alive if it's a hit
                        if matches!(state, battleship_models::CellState::Hit) {
                            target_player.ships_alive = target_player.ships_alive.saturating_sub(1);
                        }

                        // Update the shooter's status to indicate they've fired
                        let shooter = self.get_mut_player(player_id_clone);
                        if let PlayerStatus::Deciding(_) = shooter.status {
                            shooter.status = PlayerStatus::Deciding(false);
                        }

                        Ok((
                            NotificationType::Broadcast,
                            battleship_models::ServerCommand::LaunchMissleConfirmation(
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
                    a.status = PlayerStatus::Selecting(cnt.clone());
                    b.status = PlayerStatus::Selecting(cnt.clone());
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
                    // tranditioning from selections to launching
                    self.status = GameStatus::PlayerTurn(a.id.clone());
                    a.status = PlayerStatus::Deciding(true); // true means missle loadec
                    Some(ServerCommand::PlayerTurn(a.id.clone()))
                } else {
                    None
                }
            }
            GameStatus::PlayerTurn(id) => {
                // TODO: add a check to see if player launched or not
                // getting player who wasn launched at
                let (bomber, bombed_player) = if a.id.clone() == *id { (a, b) } else { (b, a) };
                if bombed_player.ships_alive == 0 {
                    self.status = GameStatus::GameOver;
                    Some(ServerCommand::GameOver)
                } else if let PlayerStatus::Deciding(false) = bomber.status {
                    // NOTE: false indicates missle fired
                    self.status = GameStatus::PlayerTurn(bombed_player.name.clone());
                    // bomber done deciding
                    bomber.status = PlayerStatus::Deciding(false);
                    // bombed is deciding
                    bombed_player.status = PlayerStatus::Deciding(true);
                    self.status = GameStatus::PlayerTurn(bombed_player.id.clone());
                    Some(ServerCommand::PlayerTurn(bombed_player.id.clone()))
                } else {
                    // if this case is encountered, deciding must be true
                    info!(
                        "PlayerTurn else encountered (bomber, bombed) : ({:?}, {:?})",
                        bomber.status, bombed_player.status
                    );
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
                info!("Message produced no response");
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
async fn test_simple_lobby_game_lifecycle() {
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
        Payload::ServerCommand(ServerCommand::PlaceBoatConfirmation(_))
    ));
    assert!(matches!(
        msg_b.payload,
        Payload::ServerCommand(ServerCommand::PlaceBoatConfirmation(_))
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

    // Simulate missile launch - Player A first
    let missile_msg_a = GameMessage {
        id: 3,
        sender: a_id.clone(),
        payload: Payload::ClientCommand(ClientCommand::LaunchMissle(Coordinates { x: 1, y: 1 })),
    };
    tx_a_in.send(missile_msg_a).await.unwrap();

    // Receive responses for Player A's missile launch
    let msg_a = rx_a.recv().await.unwrap();
    dbg!("msg_a: {:#?}", &msg_a);
    let msg_b = rx_b.recv().await.unwrap();
    dbg!("msg_a: {:#?}", &msg_b);
    assert!(matches!(
        msg_a.payload,
        Payload::ServerCommand(ServerCommand::LaunchMissleConfirmation(_, _))
    ));
    assert!(matches!(
        msg_b.payload,
        Payload::ServerCommand(ServerCommand::LaunchMissleConfirmation(_, _))
    ));

    // Receive turn change message OR game over (if Player A hits the last ship)
    let msg_a = rx_a.recv().await.unwrap();
    dbg!("msg_a: {:#?}", &msg_a);
    let msg_b = rx_b.recv().await.unwrap();
    dbg!("msg_a: {:#?}", &msg_b);

    // Check if game is over (Player B's ship was destroyed)
    if matches!(
        msg_a.payload,
        Payload::ServerCommand(ServerCommand::GameOver)
    ) {
        assert!(matches!(
            msg_a.payload,
            Payload::ServerCommand(ServerCommand::GameOver)
        ));
        assert!(matches!(
            msg_b.payload,
            Payload::ServerCommand(ServerCommand::GameOver)
        ));
    } else {
        // Game continues, expect turn change
        assert!(matches!(
            msg_a.payload,
            Payload::ServerCommand(ServerCommand::PlayerTurn(_))
        ));
        assert!(matches!(
            msg_b.payload,
            Payload::ServerCommand(ServerCommand::PlayerTurn(_))
        ));

        // Now Player B fires
        let missile_msg_b = GameMessage {
            id: 4,
            sender: b_id.clone(),
            payload: Payload::ClientCommand(ClientCommand::LaunchMissle(Coordinates {
                x: 0,
                y: 0,
            })),
        };
        tx_b_in.send(missile_msg_b).await.unwrap();

        // Receive responses for Player B's missile launch
        let msg_a = rx_a.recv().await.unwrap();
        dbg!("msg_a: {:#?}", &msg_a);
        let msg_b = rx_b.recv().await.unwrap();
        dbg!("msg_a: {:#?}", &msg_b);
        assert!(matches!(
            msg_a.payload,
            Payload::ServerCommand(ServerCommand::LaunchMissleConfirmation(_, _))
        ));
        assert!(matches!(
            msg_b.payload,
            Payload::ServerCommand(ServerCommand::LaunchMissleConfirmation(_, _))
        ));

        // Game should be over now
        let msg_a = rx_a.recv().await.unwrap();
        dbg!("msg_a: {:#?}", &msg_a);
        let msg_b = rx_b.recv().await.unwrap();
        dbg!("msg_a: {:#?}", &msg_b);
        assert!(matches!(
            msg_a.payload,
            Payload::ServerCommand(ServerCommand::GameOver)
        ));
        assert!(matches!(
            msg_b.payload,
            Payload::ServerCommand(ServerCommand::GameOver)
        ));
    }
    tx_from_man.send(ManagerMessage::Shutdown).await.unwrap();
    handle.await.unwrap();
    // You can extend this to simulate more of the lifecycle, e.g. selection, turns, game over, etc.
}

#[tokio::test]
async fn test_complex_lobby_game_with_multiple_ships_and_misses() {
    use crate::lobby::Lobby;
    use battleship_models::*;
    use tokio::sync::mpsc;

    // Setup player channels
    let (tx_a, mut rx_a) = mpsc::channel(10);
    let (tx_b, mut rx_b) = mpsc::channel(10);
    let (tx_a_in, rx_a_in) = mpsc::channel(10);
    let (tx_b_in, rx_b_in) = mpsc::channel(10);

    let (tx_from_man, rx_from_man) = mpsc::channel(10);

    // Create lobby with more ships and larger board
    let ship_count = 3;
    let board_size = 5;

    let handle = tokio::spawn(async move {
        if let Err(r) = Lobby::new(String::from("complex_test"), rx_from_man)
            .set_settings(battleship_models::Settings {
                rows: board_size,
                cols: board_size,
                ship_count,
            })
            .run()
            .await
        {
            dbg!("Error occurred in test lobby {:?}", r);
        }
    });

    dbg!(
        "Complex lobby started with {} ships on {}x{} board",
        ship_count,
        board_size,
        board_size
    );

    // Connect players to lobby
    tx_from_man
        .send(ManagerMessage::NewConnection(ConnectionDetails {
            player_name: "Player_Alpha".to_string(),
            tx: tx_a,
            rx: rx_a_in,
        }))
        .await
        .unwrap();
    tx_from_man
        .send(ManagerMessage::NewConnection(ConnectionDetails {
            player_name: "Player_Beta".to_string(),
            tx: tx_b,
            rx: rx_b_in,
        }))
        .await
        .unwrap();

    // Verify game initialization
    let msg_a = rx_a.recv().await.unwrap();
    let msg_b = rx_b.recv().await.unwrap();
    dbg!("Initialization messages: A={:#?}, B={:#?}", &msg_a, &msg_b);

    assert!(matches!(
        msg_a.payload,
        Payload::ServerCommand(ServerCommand::InitializeGame(_, _))
    ));
    assert!(matches!(
        msg_b.payload,
        Payload::ServerCommand(ServerCommand::InitializeGame(_, _))
    ));

    // Extract player IDs
    let a_id = if let Payload::ServerCommand(ServerCommand::InitializeGame(id, _)) = msg_a.payload {
        id
    } else {
        "Alpha".to_string()
    };
    let b_id = if let Payload::ServerCommand(ServerCommand::InitializeGame(id, _)) = msg_b.payload {
        id
    } else {
        "Beta".to_string()
    };

    // Verify selection mode
    let msg_a = rx_a.recv().await.unwrap();
    let msg_b = rx_b.recv().await.unwrap();
    dbg!("Selection mode messages: A={:#?}, B={:#?}", &msg_a, &msg_b);

    assert!(matches!(
        msg_a.payload,
        Payload::ServerCommand(ServerCommand::SelectionMode(_))
    ));
    assert!(matches!(
        msg_b.payload,
        Payload::ServerCommand(ServerCommand::SelectionMode(_))
    ));

    // Place ships for Player A at (0,0), (1,1), (2,2)
    let ship_positions_a = vec![
        Coordinates { x: 0, y: 0 },
        Coordinates { x: 1, y: 1 },
        Coordinates { x: 2, y: 2 },
    ];

    for (i, pos) in ship_positions_a.iter().enumerate() {
        let ship_msg = GameMessage {
            id: (i + 1) as u32,
            sender: a_id.clone(),
            payload: Payload::ClientCommand(ClientCommand::PlaceShip(*pos)),
        };
        tx_a_in.send(ship_msg).await.unwrap();

        // Receive ship placement confirmation
        let msg = rx_a.recv().await.unwrap();
        dbg!("Player A ship {} placement response: {:#?}", i + 1, &msg);
        assert!(matches!(
            msg.payload,
            Payload::ServerCommand(ServerCommand::PlaceBoatConfirmation(_))
        ));
    }

    // Place ships for Player B at (4,4), (3,3), (0,4)
    let ship_positions_b = vec![
        Coordinates { x: 4, y: 4 },
        Coordinates { x: 3, y: 3 },
        Coordinates { x: 0, y: 4 },
    ];

    for (i, pos) in ship_positions_b.iter().enumerate() {
        let ship_msg = GameMessage {
            id: (i + 4) as u32,
            sender: b_id.clone(),
            payload: Payload::ClientCommand(ClientCommand::PlaceShip(*pos)),
        };
        tx_b_in.send(ship_msg).await.unwrap();

        // Receive ship placement confirmation
        let msg = rx_b.recv().await.unwrap();
        dbg!("Player B ship {} placement response: {:#?}", i + 1, &msg);
        assert!(matches!(
            msg.payload,
            Payload::ServerCommand(ServerCommand::PlaceBoatConfirmation(_))
        ));
    }

    // Verify transition to player turn
    let msg_a = rx_a.recv().await.unwrap();
    let msg_b = rx_b.recv().await.unwrap();
    dbg!("Player turn messages: A={:#?}, B={:#?}", &msg_a, &msg_b);

    assert!(matches!(
        msg_a.payload,
        Payload::ServerCommand(ServerCommand::PlayerTurn(_))
    ));
    assert!(matches!(
        msg_b.payload,
        Payload::ServerCommand(ServerCommand::PlayerTurn(_))
    ));

    // Start missile sequence with deliberate misses before hits
    let mut message_id = 7;

    // Player A fires and misses (targeting empty cells)
    let miss_targets_a = vec![
        Coordinates { x: 1, y: 0 }, // Miss
        Coordinates { x: 2, y: 0 }, // Miss
        Coordinates { x: 4, y: 0 }, // Miss
    ];

    for (i, target) in miss_targets_a.iter().enumerate() {
        let missile_msg = GameMessage {
            id: message_id,
            sender: a_id.clone(),
            payload: Payload::ClientCommand(ClientCommand::LaunchMissle(*target)),
        };
        tx_a_in.send(missile_msg).await.unwrap();
        message_id += 1;

        // Receive missile launch confirmation (should be Miss)
        let msg_a = rx_a.recv().await.unwrap();
        let msg_b = rx_b.recv().await.unwrap();
        dbg!(
            "Player A miss {} responses: A={:#?}, B={:#?}",
            i + 1,
            &msg_a,
            &msg_b
        );

        assert!(matches!(
            msg_a.payload,
            Payload::ServerCommand(ServerCommand::LaunchMissleConfirmation(CellState::Miss, _))
        ));
        assert!(matches!(
            msg_b.payload,
            Payload::ServerCommand(ServerCommand::LaunchMissleConfirmation(CellState::Miss, _))
        ));

        // Receive turn change message
        let msg_a = rx_a.recv().await.unwrap();
        let msg_b = rx_b.recv().await.unwrap();
        dbg!(
            "Turn change after A's miss {}: A={:#?}, B={:#?}",
            i + 1,
            &msg_a,
            &msg_b
        );

        assert!(matches!(
            msg_a.payload,
            Payload::ServerCommand(ServerCommand::PlayerTurn(_))
        ));
        assert!(matches!(
            msg_b.payload,
            Payload::ServerCommand(ServerCommand::PlayerTurn(_))
        ));

        // Player B's turn - fire and miss
        let b_miss_target = match i {
            0 => Coordinates { x: 1, y: 2 },
            1 => Coordinates { x: 3, y: 0 },
            _ => Coordinates { x: 4, y: 1 },
        };

        let missile_msg_b = GameMessage {
            id: message_id,
            sender: b_id.clone(),
            payload: Payload::ClientCommand(ClientCommand::LaunchMissle(b_miss_target)),
        };
        tx_b_in.send(missile_msg_b).await.unwrap();
        message_id += 1;

        // Receive Player B's miss
        let msg_a = rx_a.recv().await.unwrap();
        let msg_b = rx_b.recv().await.unwrap();
        dbg!(
            "Player B miss {} responses: A={:#?}, B={:#?}",
            i + 1,
            &msg_a,
            &msg_b
        );

        assert!(matches!(
            msg_a.payload,
            Payload::ServerCommand(ServerCommand::LaunchMissleConfirmation(CellState::Miss, _))
        ));
        assert!(matches!(
            msg_b.payload,
            Payload::ServerCommand(ServerCommand::LaunchMissleConfirmation(CellState::Miss, _))
        ));

        // Receive turn change message back to A
        let msg_a = rx_a.recv().await.unwrap();
        let msg_b = rx_b.recv().await.unwrap();
        dbg!(
            "Turn change after B's miss {}: A={:#?}, B={:#?}",
            i + 1,
            &msg_a,
            &msg_b
        );

        assert!(matches!(
            msg_a.payload,
            Payload::ServerCommand(ServerCommand::PlayerTurn(_))
        ));
        assert!(matches!(
            msg_b.payload,
            Payload::ServerCommand(ServerCommand::PlayerTurn(_))
        ));
    }

    // Now start hitting ships - Player A hits Player B's ships
    for (i, target) in ship_positions_b.iter().enumerate() {
        let missile_msg = GameMessage {
            id: message_id,
            sender: a_id.clone(),
            payload: Payload::ClientCommand(ClientCommand::LaunchMissle(*target)),
        };
        tx_a_in.send(missile_msg).await.unwrap();
        message_id += 1;

        // Receive hit confirmation
        let msg_a = rx_a.recv().await.unwrap();
        let msg_b = rx_b.recv().await.unwrap();
        dbg!(
            "Player A hit {} responses: A={:#?}, B={:#?}",
            i + 1,
            &msg_a,
            &msg_b
        );

        assert!(matches!(
            msg_a.payload,
            Payload::ServerCommand(ServerCommand::LaunchMissleConfirmation(CellState::Hit, _))
        ));
        assert!(matches!(
            msg_b.payload,
            Payload::ServerCommand(ServerCommand::LaunchMissleConfirmation(CellState::Hit, _))
        ));

        // If this isn't the last ship, expect turn change and Player B's turn
        if i < ship_positions_b.len() - 1 {
            // Expect turn change message
            let msg_a = rx_a.recv().await.unwrap();
            let msg_b = rx_b.recv().await.unwrap();
            dbg!(
                "Turn change after A's hit {}: A={:#?}, B={:#?}",
                i + 1,
                &msg_a,
                &msg_b
            );

            assert!(matches!(
                msg_a.payload,
                Payload::ServerCommand(ServerCommand::PlayerTurn(_))
            ));
            assert!(matches!(
                msg_b.payload,
                Payload::ServerCommand(ServerCommand::PlayerTurn(_))
            ));

            // Player B fires back and hits one of A's ships
            let missile_msg_b = GameMessage {
                id: message_id,
                sender: b_id.clone(),
                payload: Payload::ClientCommand(ClientCommand::LaunchMissle(ship_positions_a[i])),
            };
            tx_b_in.send(missile_msg_b).await.unwrap();
            message_id += 1;

            // Receive Player B's hit
            let msg_a = rx_a.recv().await.unwrap();
            let msg_b = rx_b.recv().await.unwrap();
            dbg!(
                "Player B hit {} responses: A={:#?}, B={:#?}",
                i + 1,
                &msg_a,
                &msg_b
            );

            assert!(matches!(
                msg_a.payload,
                Payload::ServerCommand(ServerCommand::LaunchMissleConfirmation(CellState::Hit, _))
            ));
            assert!(matches!(
                msg_b.payload,
                Payload::ServerCommand(ServerCommand::LaunchMissleConfirmation(CellState::Hit, _))
            ));

            // Expect turn change back to A
            let msg_a = rx_a.recv().await.unwrap();
            let msg_b = rx_b.recv().await.unwrap();
            dbg!(
                "Turn change after B's hit {}: A={:#?}, B={:#?}",
                i + 1,
                &msg_a,
                &msg_b
            );

            assert!(matches!(
                msg_a.payload,
                Payload::ServerCommand(ServerCommand::PlayerTurn(_))
            ));
            assert!(matches!(
                msg_b.payload,
                Payload::ServerCommand(ServerCommand::PlayerTurn(_))
            ));
        }
    }

    // Verify game over after all ships are destroyed
    let msg_a = rx_a.recv().await.unwrap();
    let msg_b = rx_b.recv().await.unwrap();
    dbg!("Game over messages: A={:#?}, B={:#?}", &msg_a, &msg_b);

    assert!(matches!(
        msg_a.payload,
        Payload::ServerCommand(ServerCommand::GameOver)
    ));
    assert!(matches!(
        msg_b.payload,
        Payload::ServerCommand(ServerCommand::GameOver)
    ));

    // Shutdown the lobby
    tx_from_man.send(ManagerMessage::Shutdown).await.unwrap();
    handle.await.unwrap();

    dbg!(
        "Complex test completed successfully - {} ships placed, multiple misses and hits verified",
        ship_count
    );
}
