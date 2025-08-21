use std::sync::{Arc, Mutex};

use crate::player::Player;
use battleship_models::{self, SelectionCriteria};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;

pub struct Lobby {
    id: String,
    settings: battleship_models::Settings,
    shared_player_a: Option<Arc<Mutex<Player>>>,
    shared_player_b: Option<Arc<Mutex<Player>>>,
}
impl Lobby {
    pub fn new(id: String) -> Self {
        Self {
            id,
            settings: battleship_models::Settings { rows: 8, cols: 8 },
            shared_player_a: None,
            shared_player_b: None,
        }
    }
    pub async fn join(
        &mut self,
        ws: WebSocketStream<TcpStream>,
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

        Self::send_json(shared_player_ws, &self.settings).await?;
        Ok(())
    }
    pub fn is_lobby_full(&self) -> bool {
        self.shared_player_a.is_some() && self.shared_player_b.is_some()
    }
    pub fn get_id(&self) -> String {
        self.id.to_string()
    }
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // send msg to get selections
        Ok(())
    }
    pub async fn broadcast_message(
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
                Self::send_json(player.get_ws_mut(), payload).await?;
            }
        }
        Ok(())
    }
    pub async fn send_json(
        ws: &mut WebSocketStream<TcpStream>,
        payload: &impl Serialize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let msg = serde_json::to_string(payload)?;
        ws.send(tokio_tungstenite::tungstenite::Message::text(msg))
            .await?;
        Ok(())
    }
    pub async fn receive_json<T>(
        ws: &mut WebSocketStream<TcpStream>,
    ) -> Result<T, Box<dyn std::error::Error>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let msg = ws.next().await.ok_or("Stream Closed")??;
        let o = serde_json::from_str::<T>(msg.to_text()?)?;
        Ok(o)
    }
}
