use std::net::TcpStream;

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::WebSocketStream;

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
