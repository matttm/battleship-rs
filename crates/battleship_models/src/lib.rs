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
    pub x: u32,
    pub y: u32,
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
pub enum GameStates {
    InitializeGame(Settings),
    SelectionMode(SelectionCriteria),
    PlaceShip(Coordinates),
    LaunchMissle(Coordinates),
    GameOver,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct GameMessage {
    pub id: u32,
    pub sender: String,
    pub payload: GameStates,
}
