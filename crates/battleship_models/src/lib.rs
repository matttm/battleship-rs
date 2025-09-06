use serde::{Deserialize, Serialize};

#[derive(Clone, Copy)]
pub enum CellStates {
    Empty,
    Boat,
    Miss,
    Hit,
}
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Coordinates {
    pub x: usize,
    pub y: usize,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Settings {
    pub rows: usize,
    pub cols: usize,
}
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct SelectionCriteria {
    pub count: usize,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum ClientCommand {
    PlaceShip(Coordinates),
    LaunchMissle(Coordinates),
}
#[derive(Serialize, Deserialize, Clone)]
pub enum ServerCommand {
    InitializeGame(Settings),
    SelectionMode(SelectionCriteria),
    PlayerTurn(String), // the string is whose turn it is
    Text(String),
    GameOver,
}
#[derive(Serialize, Deserialize, Clone)]
pub enum Payload {
    ClientCommand(ClientCommand),
    ServerCommand(ServerCommand),
}
#[derive(Serialize, Deserialize, Clone)]
pub struct GameMessage {
    pub id: u32,
    pub sender: String,
    pub payload: Payload,
}
