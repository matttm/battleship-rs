use crate::server::GameServer;

pub mod match_maker;
pub mod server;
/// A WebSocket echo server
fn main() -> () {
    let gs = GameServer::new();
    gs.start();
}
