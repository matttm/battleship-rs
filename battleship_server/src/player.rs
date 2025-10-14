use battleship_models::{self, CellState, GameMessage};
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum PlayerStatus {
    Uninitialized,
    Initialized,
    Selecting(usize), // usizes count remaining to be chosen
    Deciding(bool),   // bool is whether player has launched missle yet
    Idle,
}
pub struct Player {
    pub id: String, // uuid
    pub name: String,
    pub status: PlayerStatus,
    pub tx: mpsc::Sender<GameMessage>,
    pub rx: mpsc::Receiver<GameMessage>,
    pub board: Box<[Box<[CellState]>]>,
    pub ships_alive: usize,
}
impl Player {
    pub fn new(
        tx: mpsc::Sender<GameMessage>,
        rx: mpsc::Receiver<GameMessage>,
        rows: usize,
        cols: usize,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::from(""),
            tx,
            rx,
            status: PlayerStatus::Uninitialized,
            board: Self::initialize_board(rows, cols),
            ships_alive: 0,
        }
    }
    pub fn place_ship(&mut self, row: usize, col: usize) -> Result<(), Box<dyn std::error::Error>> {
        let current = &self.board[row][col];
        match current {
            CellState::Empty => self.set_cell(row, col, CellState::Boat),
            CellState::Boat => Err("There is already a ship at this position.".into()),
            _ => Err("Unknown error".into()),
        }
    }
    pub fn strike_cell(
        &mut self,
        row: usize,
        col: usize,
    ) -> Result<CellState, Box<dyn std::error::Error>> {
        let current = &self.board[row][col];
        let state;
        match current {
            CellState::Empty => {
                state = CellState::Miss;
            }
            CellState::Boat => {
                state = CellState::Hit;
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
        state: CellState,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.board[row][col] = state.clone();
        Ok(())
    }
    fn initialize_board(rows: usize, cols: usize) -> Box<[Box<[CellState]>]> {
        let mut vec_rows = Vec::with_capacity(rows);
        for _ in 0..rows {
            vec_rows.push(vec![CellState::Empty; cols].into_boxed_slice());
        }
        vec_rows.into_boxed_slice()
    }
}
