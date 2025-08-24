use battleship_models::{self, CellStates};

pub enum PlayerStatus {
    Selecting(u8), // u8 is count remaining to be chosen
    Deciding,
    Idle,
}
pub struct Player {
    name: String,
    // status: PlayerStatus,
    board: Box<[Box<[CellStates]>]>,
}
impl Player {
    pub fn new(name: String, rows: usize, cols: usize) -> Self {
        Self {
            name,
            // status: PlayerStatus::Idle,
            board: Self::initialize_board(rows, cols),
        }
    }
    pub fn strike_cell(
        &mut self,
        row: usize,
        col: usize,
    ) -> Result<CellStates, Box<dyn std::error::Error>> {
        let current = &self.board[row][col];
        match current {
            CellStates::Empty => self.set_cell(row, col, CellStates::Miss),
            CellStates::Boat => self.set_cell(row, col, CellStates::Hit),
            _ => Ok(current.clone()),
        }
    }
    pub fn set_cell(
        &mut self,
        row: usize,
        col: usize,
        state: CellStates,
    ) -> Result<CellStates, Box<dyn std::error::Error>> {
        self.board[row][col] = state.clone();
        Ok(state)
    }
    fn initialize_board(rows: usize, cols: usize) -> Box<[Box<[CellStates]>]> {
        let mut vec_rows = Vec::with_capacity(rows);
        for _ in 0..rows {
            vec_rows.push(vec![CellStates::Empty; cols].into_boxed_slice());
        }
        vec_rows.into_boxed_slice()
    }
}
