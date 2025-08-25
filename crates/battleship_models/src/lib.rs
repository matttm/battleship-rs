use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub enum CellStates {
    Empty,
    Boat,
    Miss,
    Hit,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct Coordinates {
    pub x: u32,
    pub y: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    pub rows: usize,
    pub cols: usize,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct SelectionCriteria {
    pub count: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum GameStates {
    InitializeGame(Settings),
    SelectionMode(SelectionCriteria),
    PlaceShip(Coordinates),
    LaunchMissle(Coordinates),
    GameOver,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub id: u32,
    pub sender: String,
    pub payload: GameStates,
}
