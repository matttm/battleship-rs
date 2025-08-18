use std::net::TcpStream;

use battleship_models;
use serde::{Deserialize, Serialize};

struct Player {
    socket: tungstenite::WebSocket<TcpStream>,
    board: Option<Box<[Box<CellStates>]>>,
}
impl Player {
    pub fn new(ws: tungstenite::WebSocket<TcpStream>, rows: u32, cols: u32) -> Self {
        Self {
            socket: ws,
            board: None,
        }
    }
    pub fn get_ws_mut(&mut self) -> &mut tungstenite::WebSocket<TcpStream> {
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
#[derive(Clone)]
pub enum CellStates {
    Empty,
    Boat,
    Miss,
    Hit,
}
pub struct MatchMaker {
    settings: battleship_models::Settings,
    player_a: Option<Player>,
    player_b: Option<Player>,
}
impl MatchMaker {
    pub fn new() -> Self {
        MatchMaker {
            settings: battleship_models::Settings { rows: 8, cols: 8 },
            player_a: None,
            player_b: None,
        }
    }
    pub fn join(
        &mut self,
        ws: tungstenite::WebSocket<TcpStream>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let player_slot = if self.player_a.is_none() {
            &mut self.player_a
        } else {
            &mut self.player_b
        };

        *player_slot = Some(Player::new(ws, self.settings.rows, self.settings.cols));

        // Get the mutable reference directly from the newly-assigned slot
        let player_ws = player_slot.as_mut().unwrap().get_ws_mut();

        Self::send_json(player_ws, &self.settings)?;
        Ok(())
    }
    pub fn send_json(
        ws: &mut tungstenite::WebSocket<TcpStream>,
        payload: &impl Serialize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let msg = serde_json::to_string(payload).unwrap();
        ws.send(tungstenite::Message::text(msg)).unwrap();
        Ok(())
    }
}
