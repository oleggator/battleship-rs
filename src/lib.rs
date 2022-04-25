//! Battleship game implemented in Rust

#![warn(missing_docs, clippy::unwrap_used)]

pub mod game;
pub mod grid;
pub mod player;
pub mod ship;

use crate::game::Game;
use crate::grid::Grid;
use crate::player::Player;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

/// Type alias for the standard [`Result`] type.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Runs the game.
pub fn run() -> Result<()> {
    let (grid_width, grid_height) = (10, 10);
    let listener = TcpListener::bind("0.0.0.0:1234")?;
    log::info!("Server is listening on port :1234");
    let game = Arc::new(Mutex::new(Game::default()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                log::debug!("New connection: {}", stream.peer_addr()?);
                let mut player = Player::new(stream);
                if game.lock().expect("failed to retrieve game").is_ready() {
                    player.send_message("Lobby is full.")?;
                    continue;
                }
                let game = Arc::clone(&game);
                thread::spawn(move || {
                    let start_game = move || -> Result<()> {
                        player.greet()?;
                        let mut game = game.lock().expect("failed to retrieve game");
                        game.add_player(player)?;
                        if game.is_ready() {
                            for player in game.players.iter_mut() {
                                player.grid = Grid::new_random(grid_width, grid_height);
                            }
                            game.play(grid_width, grid_height)?;
                        }
                        Ok(())
                    };
                    start_game().expect("failed to run game")
                });
            }
            Err(e) => {
                log::error!("Connection failed: {}", e);
            }
        }
    }
    Ok(())
}
