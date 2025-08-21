use tokio::net::TcpStream;

use battleship_models::{self, CellStates};
use tokio_tungstenite::WebSocketStream;

pub enum PlayerStatus {
    Selecting(u8), // u8 is count remaining to be chosen
    Deciding,
    Idle,
}
pub struct Player {
    socket: WebSocketStream<TcpStream>,
    // status: PlayerStatus,
    board: Box<[Box<[CellStates]>]>,
}
impl Player {
    pub fn new(ws: WebSocketStream<TcpStream>, rows: usize, cols: usize) -> Self {
        Self {
            socket: ws,
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
    pub fn get_ws_mut(&mut self) -> &mut WebSocketStream<TcpStream> {
        &mut self.socket
    }
    fn initialize_board(rows: usize, cols: usize) -> Box<[Box<[CellStates]>]> {
        let mut vec_rows = vec![];
        for _ in 0..rows {
            vec_rows.push(vec![CellStates::Empty; cols].into_boxed_slice());
        }
        vec_rows.into_boxed_slice()
    }
}
