pub struct MatchMaker {
    settings: battleship_models::Settings,
    player_a: Option<Player>,
    player_b: Option<Player>,
}
struct Player {
    socket: tungstenite::WebSocket<TcpStream>,
}
impl Player {
    pub fn new(ws: tungstenite::WebSocket<TcpStream>) -> Self {
        Self { socket: ws }
    }
}
use std::net::TcpStream;

use battleship_models;

impl MatchMaker {
    pub fn new() -> Self {
        MatchMaker {
            settings: battleship_models::Settings { rows: 8, cols: 8 },
            player_a: None,
            player_b: None,
        }
    }
    pub fn join(&mut self, ws: tungstenite::WebSocket<TcpStream>) {
        if let None = self.player_a {
            self.player_a = Some(Player::new(ws));
        } else {
            self.player_b = Some(Player::new(ws));
        }
        ws.send(tungstenite::Message::binary()))
    }
}
