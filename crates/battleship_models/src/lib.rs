use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug)]
pub enum CellStates {
    Empty,
    Boat,
    Miss,
    Hit,
}
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Coordinates {
    pub x: usize,
    pub y: usize,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Settings {
    pub rows: usize,
    pub cols: usize,
    pub ship_count: usize,
}
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct SelectionCriteria {
    pub count: usize,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum ClientCommand {
    PlaceShip(Coordinates),
    LaunchMissle(Coordinates),
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerCommand {
    InitializeGame(Settings),
    SelectionMode(SelectionCriteria),
    PlayerTurn(String), // the string is whose turn it is
    Text(String),
    GameOver,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Payload {
    ClientCommand(ClientCommand),
    ServerCommand(ServerCommand),
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameMessage {
    pub id: u32,
    pub sender: String,
    pub payload: Payload,
}
