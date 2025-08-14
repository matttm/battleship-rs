pub struct Coordinates {
    pub x: u32,
    pub y: u32,
}

pub struct Settings {
    pub rows: u32,
    pub cols: u32,
}

pub enum GameStates {
    JoinGame,
    InitializeGame(Settings),
    PlaceShip(Coordinates),
    LaunchMissle(Coordinates),
}
