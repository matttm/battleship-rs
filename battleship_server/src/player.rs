use battleship_models::{self, CellStates, GameMessage};
use tokio::sync::mpsc;

pub enum PlayerStatus {
    Initialized,
    Selecting(u8), // u8 is count remaining to be chosen
    Deciding,
    Idle,
}
pub struct Player {
    pub name: String,
    status: PlayerStatus,
    pub tx: mpsc::Sender<GameMessage>,
    pub rx: mpsc::Receiver<GameMessage>,
    board: Box<[Box<[CellStates]>]>,
    pub ships_to_place: usize,
    pub ships_alive: usize,
}
impl Player {
    pub fn new(
        name: String,
        tx: mpsc::Sender<GameMessage>,
        rx: mpsc::Receiver<GameMessage>,
        rows: usize,
        cols: usize,
    ) -> Self {
        Self {
            name,
            tx,
            rx,
            status: PlayerStatus::Idle,
            board: Self::initialize_board(rows, cols),
            ships_to_place: 0,
            ships_alive: 0,
        }
    }
    pub fn place_ship(
        &mut self,
        row: usize,
        col: usize,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let current = &self.board[row][col];
        match current {
            CellStates::Empty => {
                if let Ok(_) = self.set_cell(row, col, CellStates::Boat) {
                    Ok(String::from("Boat placed"))
                } else {
                    Err("Error placing boat".into())
                }
            }
            CellStates::Boat => Err("There is already a ship at this position.".into()),
            _ => Err("Unknown error".into()),
        }
    }
    pub fn strike_cell(
        &mut self,
        row: usize,
        col: usize,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let current = &self.board[row][col];
        match current {
            CellStates::Empty => {
                self.set_cell(row, col, CellStates::Miss);
                Ok(String::from("Miss Fire"))
            }
            CellStates::Boat => {
                self.set_cell(row, col, CellStates::Hit);
                Ok(String::from("Critical hit"))
            }
            _ => Err("".into()),
        }
    }
    pub fn set_cell(
        &mut self,
        row: usize,
        col: usize,
        state: CellStates,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.board[row][col] = state.clone();
        Ok(())
    }
    fn initialize_board(rows: usize, cols: usize) -> Box<[Box<[CellStates]>]> {
        let mut vec_rows = Vec::with_capacity(rows);
        for _ in 0..rows {
            vec_rows.push(vec![CellStates::Empty; cols].into_boxed_slice());
        }
        vec_rows.into_boxed_slice()
    }
}
