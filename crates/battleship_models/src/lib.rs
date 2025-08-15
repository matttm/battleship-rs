use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Coordinates {
    pub x: u32,
    pub y: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub rows: u32,
    pub cols: u32,
}

#[derive(Serialize, Deserialize)]
pub enum GameStates {
    JoinGame,
    InitializeGame(Settings),
    PlaceShip(Coordinates),
    LaunchMissle(Coordinates),
}
