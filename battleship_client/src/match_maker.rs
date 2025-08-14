pub struct MatchMaker {
    settings: battleship_models::Settings,
}

use battleship_models;

impl MatchMaker {
    pub fn new() -> Self {
        return MatchMaker {
            settings: battleship_models::Settings { rows: 8, cols: 8 },
        };
    }
}
