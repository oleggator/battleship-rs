//! Main game.

use crate::grid::Coordinate;
use crate::grid::Grid;
use crate::player::Player;
use crate::ship::Ship;
use crate::Result;
use std::convert::TryFrom;
use std::thread;
use std::time::Duration;

/// Maximum number of players.
pub const MAX_PLAYERS: usize = 3;

/// Representation of the Battleship game.
///
/// Handles the turns and game logic.
#[derive(Default, Debug)]
pub struct Game {
    /// Players of the game.
    pub players: Vec<Player>,
}

impl Game {
    /// Checks if the players are ready to play.
    pub fn is_ready(&self) -> bool {
        self.players.len() == MAX_PLAYERS
    }

    /// Adds a new player to the game.
    ///
    /// Also see [`Game::is_ready`]
    pub fn add_player(&mut self, player: Player) -> Result<()> {
        if self.players.len() < MAX_PLAYERS {
            self.players.push(player);
            self.players[0].send("Waiting for opponent...\n")?;
        } else {
            self.players.push(player);
            for i in 0..MAX_PLAYERS {
                let message = format!(
                    "Your opponent is {}\n",
                    self.opponent(i).name
                );
                self.players[i].send(&message)?;
            }
        }
        Ok(())
    }

    /// Shows countdown to players for starting the game.
    fn show_countdown(&mut self) -> Result<()> {
        println!("[#] Game is starting.");
        for i in 1..4 {
            let message = format!("Game starts in {}...\n", 4 - i);
            self.players.iter_mut().try_for_each(|p| p.send(&message))?;
            thread::sleep(Duration::from_secs(1));
        }
        Ok(())
    }

    /// Shows the grid of the players.
    ///
    /// Hits/misses are shown on the upper grid.
    /// Lower grid is used for showing the player ships.
    fn show_grid(&mut self, width: u8, height: u8) -> Result<()> {
        for i in 0..MAX_PLAYERS {
            // Show upper grid (hits/misses).
            let ships = self.opponent(i)
                .grid
                .hits
                .iter()
                .map(|coord| Ship {
                    coords: vec![Coordinate {
                        x: coord.x,
                        y: coord.y,
                        is_hit: self.opponent(i)
                            .grid
                            .ships
                            .iter()
                            .any(|ship| ship.coords.contains(coord)),
                    }],
                    ..Default::default()
                })
                .collect();
            let grid_str = Grid {
                width,
                height,
                ships,
                hits: vec![]
            }
            .as_string(false)?;
            self.players[i].send(&grid_str)?;

            // Show lower grid (ships).
            self.players[i].send("\nYour grid:")?;
            let grid_str = self.players[i].grid.as_string(true)?;
            self.players[i].send(&grid_str)?;
        }
        Ok(())
    }

    /// Starts the game.
    ///
    /// Number of players is determined by [`MAX_PLAYERS`] constant.
    /// Game loop continues until one of the players hits all of the ships of the opponent.
    /// Lower and upper grids are shown along with extra messages during the gameplay.
    pub fn start(&mut self, grid_width: u8, grid_height: u8) -> Result<()> {
        self.show_countdown()?;
        'game: loop {
            let mut i = 0;
            while i < MAX_PLAYERS {
                // Check if the player has won.
                if self.players[i].grid.ships.iter().all(|ship| ship.is_sunk()) {
                    let message = format!("{} won.\n", self.opponent(i).name);
                    self.players[i].send(&message)?;
                    self.opponent_mut(i).send("You won!\n")?;
                    self.players.clear();
                    print!("[#] {}", message);
                    break 'game;
                }

                // Show the grid.
                self.show_grid(grid_width, grid_height)?;

                // Handle the player turn.
                {
                    let msg = format!("Your turn to shoot {}: ", self.opponent(i).name);
                    self.players[i].send(&msg)?;
                }
                let message = format!("{}'s turn.\n", self.players[i].name);
                print!("[#] {}", message);
                for j in 0..self.players.len() {
                    if j != i {
                        self.players[j].send(&message)?;
                    }
                }
                
                // Parse the grid coordinate.
                let coordinate_str = self.players[i].read()?;
                let coordinate =
                    if let Ok(coordinate) = Coordinate::try_from(coordinate_str.to_string()) {
                        println!(
                            "[#] {} is firing a shot: {} ({:?})",
                            self.players[i].name, coordinate_str, coordinate
                        );
                        coordinate
                    } else {
                        self.players[i].send("Your missile went to space!\n")?;
                        continue;
                    };

                // Handle hit/miss.
                self.opponent_mut(i).grid.hits.push(coordinate);
                let is_hit = if let Some(coordinate) = self.opponent_mut(i)
                    .grid
                    .ships
                    .iter_mut()
                    .find(|ship| ship.coords.contains(&coordinate))
                    .and_then(|ship| ship.coords.iter_mut().find(|c| *c == &coordinate))
                {
                    coordinate.is_hit = true;
                    self.players[i].send("Hit!\n")?;
                    true
                } else {
                    self.players[i].send("Missed.\n")?;
                    false
                };

                // Inform about the game stats.
                let message = {
                    let opponent = self.opponent(i);
                    format!(
                        "{} has {} ships remaining.\n",
                        opponent.name,
                        opponent
                            .grid
                            .ships
                            .iter()
                            .filter(|ship| !ship.is_sunk())
                            .count()
                    )
                };
                self.players[i].send(&message)?;
                let message = format!("{} is firing at {}\n", self.players[i].name, coordinate);
                self.opponent_mut(i).send(&message)?;

                if !is_hit {
                    i += 1;
                }
            }
        }
        Ok(())
    }

    fn opponent_index(&self, i: usize) -> usize {
        (i + 1) % MAX_PLAYERS
    }

    fn opponent(&self, i: usize) -> &Player {
        let player_index = self.opponent_index(i);
        &self.players[player_index]
    }

    fn opponent_mut(&mut self, i: usize) -> &mut Player {
        let player_index = self.opponent_index(i);
        &mut self.players[player_index]
    }
}
