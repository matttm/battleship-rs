# battleship-rs

## Description

A two-player recreation of the classic tabletop game written in Rust.

As of 9/12/25, this reposiitory includes an initial version of a game server, using the websocket protocol. 

## Getting Started

Assuming you have cargo and rust installed, from the root directory, run
```
cargo build
```

Then run
```
❯ cargo test -p battleship_server --bin battleship_server -- --nocapture
```
The `nocapture` flag prevents debug output from being hidden.

## Design Decisions

This section will document key design decisions made for this project.

- initially, I tried to maintain a reference to the lobby inside of the lobby manager, which proves to be very difficult.

## Authors

-   Matt Maloney : matttm

## Contribute

If you want to contribute, just send me a message.
