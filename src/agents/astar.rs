use std::cell::RefCell;
use std::cmp::Ordering;

use rand::rngs::SmallRng;
use rand::seq::IteratorRandom;
use rand::SeedableRng;

use crate::env::*;
use crate::game::Game;
use crate::grid::CellT;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StarAgent;

enum Mode {
    Food,
    Kill(usize),
}

impl StarAgent {
    pub async fn step(&self, game: &Game) -> MoveResponse {
        let my = &game.snakes[0];
        let mut mode = Mode::Food;
        for i in 1..game.snakes.len() {
            if game.snakes[i].alive()
                && my.body.len() > 10
                && my.body.len() > game.snakes[i].body.len() + 3
                && (game.snakes[i].head().x - my.head().x).abs() <= 1
                && (game.snakes[i].head().y - my.head().y).abs() <= 1
            {
                mode = Mode::Kill(i);
            }
        }

        let target = match mode {
            Mode::Food => {
                let mut food = Vec::new();
                for y in 0..game.grid.height as i16 {
                    for x in 0..game.grid.width as i16 {
                        if game.grid[v2(x, y)].t == CellT::Food {
                            food.push(v2(x, y));
                        }
                    }
                }

                food.iter()
                    .min_by(|&&a, &&b| {
                        let distance_a =
                            ((a.x - my.head().x).pow(2) + (a.y - my.head().y).pow(2)) as f64;
                        let distance_b =
                            ((b.x - my.head().x).pow(2) + (b.y - my.head().y).pow(2)) as f64;
                        distance_a
                            .partial_cmp(&distance_b)
                            .unwrap_or(Ordering::Equal)
                    })
                    .copied()
            }
            Mode::Kill(i) => Some(game.snakes[i].head()),
        };

        match target {
            Some(target) => {
                let path = game.grid.a_star(my.head(), target, &[0.0, 0.0, 0.0, 0.0]);
                match path {
                    Some(path) => {
                        if path.len() >= 2 {
                            MoveResponse::new(Direction::from(path[1] - path[0]))
                        } else {
                            MoveResponse::new(random(game))
                        }
                    }
                    None => MoveResponse::new(random(game)),
                }
            }
            None => MoveResponse::new(random(game)),
        }
    }
}

thread_local! {
    static RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_entropy())
}

fn random(game: &Game) -> Direction {
    let moves = game.valid_moves(0);
    RNG.with_borrow_mut(|rng| moves.choose(rng).unwrap_or(Direction::Up))
}
