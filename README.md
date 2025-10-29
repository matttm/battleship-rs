# battleship-rs
<img width="800" alt="image" src="https://github.com/user-attachments/assets/55c195cb-3292-49ea-8d3b-f45c2b57cf10" />

## Description

A two-player recreation of the classic tabletop game written in Rust. There are two seperate binaries--one for the client and one for the backend.

## Getting Started

Assuming you have cargo and rust installed, from the root directory, run
```
❯ cargo build
```

Then to run the tests, run
```
❯ cargo test -p battleship_server --bin battleship_server -- --nocapture
```
The `nocapture` flag prevents debug output from being hidden.

To play, you'll need three terminals--one for backend (a), two for client (b) and (c).

In (a), run
```
> cd battleship-rs/battleship_server
```
then start with
```

RUST_LOG=info cargo run
```
Similarly, for (b) and (c), run
```
> cd battleship-rs/battleship_client
```
and then start the game with the same.

Use WASD keys to move and enter for the action button. Currently, you must select four positions for your fleet's positions. Once, both players do this, begin firing!

## Design Decisions

This section will document key design decisions made for this project.

- Initially, I tried to maintain a reference to the lobby inside of the lobby manager, which proves to be very difficult because of Rust's ownership rules. I then changed this design, so the LobbyManager holds a channel to every Lobby instead of a reference.

## Development Note

Enable logging with:
```
RUST_LOG=info cargo run
```
The logs are currently output to:
```
❯ ~/Library/Application\ Support/com.matttm.battleship_client/battleship_client.log
```

## Authors

-   Matt Maloney : matttm

## Contribute

If you want to contribute, just send me a message.
