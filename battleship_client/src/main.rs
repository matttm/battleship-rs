use std::env;

use crate::server::GameServer;

pub mod match_maker;
pub mod server;
/// A WebSocket echo server
fn main() -> () {
    let args: Vec<String> = env::args().collect();
    let url: String;
    let gs = GameServer::new();
    if args.len() <= 1 {
        // start server
        url = gs.start();
    } else {
        url = args[1];
    }
}
