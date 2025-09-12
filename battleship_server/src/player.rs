use battleship_models::{self, CellStates, GameMessage};
use tokio::sync::mpsc;

pub enum PlayerStatus {
    Uninitialized,
    Initialized,
    Selecting(usize), // usizes count remaining to be chosen
    Deciding(bool),   // bool is whether player has launched missle yet
    Idle,
}
pub struct Player {
    pub name: String,
    pub status: PlayerStatus,
    pub tx: mpsc::Sender<GameMessage>,
    pub rx: mpsc::Receiver<GameMessage>,
    board: Box<[Box<[CellStates]>]>,
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
            status: PlayerStatus::Uninitialized,
            board: Self::initialize_board(rows, cols),
            ships_alive: 0,
        }
    }
    pub fn place_ship(
        &mut self,
        row: usize,
        col: usize,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let current = &self.board[row][col];
        match current {
            CellStates::Empty => self.set_cell(row, col, CellStates::Boat),
            CellStates::Boat => Err("There is already a ship at this position.".into()),
            _ => Err("Unknown error".into()),
        }
    }
    pub fn strike_cell(
        &mut self,
        row: usize,
        col: usize,
    ) -> Result<CellStates, Box<dyn std::error::Error>> {
        let current = &self.board[row][col];
        let state;
        match current {
            CellStates::Empty => {
                state = CellStates::Miss;
            }
            CellStates::Boat => {
                state = CellStates::Hit;
            }
            _ => return Err("".into()),
        };
        self.set_cell(row, col, state)?;
        Ok(state)
    }
    pub fn set_cell(
        &mut self,
        row: usize,
        col: usize,
        state: CellStates,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        self.board[row][col] = state.clone();
        Ok(true)
    }
    fn initialize_board(rows: usize, cols: usize) -> Box<[Box<[CellStates]>]> {
        let mut vec_rows = Vec::with_capacity(rows);
        for _ in 0..rows {
            vec_rows.push(vec![CellStates::Empty; cols].into_boxed_slice());
        }
        vec_rows.into_boxed_slice()
    }
}
