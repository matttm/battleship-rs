use std::{
    net::TcpStream,
    sync::{Arc, Mutex},
};

use crate::player::Player;
use battleship_models::{self, GameStates, SelectionCriteria};
use serde::{Deserialize, Serialize};

pub struct MatchMaker {
    settings: battleship_models::Settings,
    shared_player_a: Option<Arc<Mutex<Player>>>,
    shared_player_b: Option<Arc<Mutex<Player>>>,
}
impl MatchMaker {
    pub fn new() -> Self {
        MatchMaker {
            settings: battleship_models::Settings { rows: 8, cols: 8 },
            shared_player_a: None,
            shared_player_b: None,
        }
    }
    pub fn join(
        &mut self,
        ws: tungstenite::WebSocket<TcpStream>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let shared_player_slot = if self.shared_player_a.is_none() {
            &mut self.shared_player_a
        } else {
            &mut self.shared_player_b
        };

        *shared_player_slot = Some(Arc::new(Mutex::new(Player::new(
            ws,
            self.settings.rows,
            self.settings.cols,
        ))));

        // Get the mutable reference directly from the newly-assigned slot
        let mut player = shared_player_slot.as_mut().unwrap().lock().unwrap();
        let shared_player_ws = player.get_ws_mut();

        Self::send_json(shared_player_ws, &self.settings)?;
        Ok(())
    }
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // send msg to get selections
        self.broadcast_message(&GameStates::SelectionMode(SelectionCriteria {
            count: 4usize,
        }))?;
        self.receive_selections()?;
        Ok(())
    }
    pub fn receive_selections(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut recipients = [&mut self.shared_player_a, &mut self.shared_player_b];
        for opt in recipients.iter_mut() {
            // Now, `shared_player_option` is `&mut &mut Option<Player>`.
            // The `if let` statement correctly and safely extracts the mutable reference.
            if let Some(shared_player) = opt {
                // `player` is now a `&mut Player`.
                let mut player = shared_player.lock().unwrap();
                let selection: GameStates = Self::receive_json(player.get_ws_mut())?;
            }
        }
        Ok(())
    }
    pub fn broadcast_message(
        &mut self,
        payload: &impl Serialize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut recipients = [&mut self.shared_player_a, &mut self.shared_player_b];
        for opt in recipients.iter_mut() {
            // Now, `shared_player_option` is `&mut &mut Option<Player>`.
            // The `if let` statement correctly and safely extracts the mutable reference.
            if let Some(shared_player) = opt {
                // `player` is now a `&mut Player`.
                let mut player = shared_player.lock().unwrap();
                Self::send_json(player.get_ws_mut(), payload)?;
            }
        }
        Ok(())
    }
    pub fn send_json(
        ws: &mut tungstenite::WebSocket<TcpStream>,
        payload: &impl Serialize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let msg = serde_json::to_string(payload)?;
        ws.send(tungstenite::Message::text(msg))?;
        Ok(())
    }
    pub fn receive_json<T>(
        ws: &mut tungstenite::WebSocket<TcpStream>,
    ) -> Result<T, Box<dyn std::error::Error>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let msg = ws.read()?;
        let o = serde_json::from_str::<T>(msg.to_text()?)?;
        Ok(o)
    }
}
