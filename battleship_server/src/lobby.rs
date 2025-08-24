use crate::{internal, player::Player};
use battleship_models::{self, Message, SelectionCriteria};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::{net::TcpStream, sync::mpsc};

pub struct Lobby {
    // TODO: give the lobby and lobby manager a channel to communicate
    id: String,
    settings: battleship_models::Settings,
    rx_from_client: mpsc::Receiver<Message>,
    tx_to_client: mpsc::Sender<Message>,
    player_a: Option<Player>,
    player_b: Option<Player>,
}
impl Lobby {
    pub fn new(
        id: String,
        rx_from_client: mpsc::Receiver<Message>,
        tx_to_client: mpsc::Sender<Message>,
    ) -> Self {
        Self {
            id,
            settings: battleship_models::Settings { rows: 8, cols: 8 },
            manager_rx,
            lobby_rx,
            player_a: None,
            player_b: None,
        }
    }
    pub async fn join(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let player_slot = if self.player_a.is_none() {
            &mut self.player_a
        } else {
            &mut self.player_b
        };

        *player_slot = Some(Player::new(
            String::from(""),
            self.settings.rows,
            self.settings.cols,
        ));

        Ok(())
    }
    pub fn is_lobby_full(&self) -> bool {
        self.player_a.is_some() && self.player_b.is_some()
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
        let mut recipients = [&mut self.player_a, &mut self.player_b];
        for opt in recipients.iter_mut() {
            // Now, `shared_player_option` is `&mut &mut Option<Player>`.
            // The `if let` statement correctly and safely extracts the mutable reference.
            if let Some(player) = opt {
                // `player` is now a `&mut Player`.
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
