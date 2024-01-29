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

impl StarAgent {
    pub async fn step(&self, game: &Game) -> MoveResponse {
        let my = &game.snakes[0];

        let mut food = Vec::new();
        for y in 0..game.grid.height as i16 {
            for x in 0..game.grid.width as i16 {
                if game.grid[v2(x, y)].t == CellT::Food {
                    food.push(v2(x, y));
                }
            }
        }

        if let Some(target) = food
            .iter()
            .min_by(|&&a, &&b| {
                let distance_a = ((a.x - my.head().x).pow(2) + (a.y - my.head().y).pow(2)) as f64;
                let distance_b = ((b.x - my.head().x).pow(2) + (b.y - my.head().y).pow(2)) as f64;
                distance_a
                    .partial_cmp(&distance_b)
                    .unwrap_or(Ordering::Equal)
            })
            .copied()
        {
            if let Some(path) = game.grid.a_star(my.head(), target, &[0.0, 0.0, 0.0, 0.0]) {
                if path.len() >= 2 {
                    return MoveResponse::new(move_check(
                        game,
                        Direction::from(path[1] - path[0]),
                        &mut None,
                    ));
                }
            }
        }
        MoveResponse::new(random(game, &mut None))
    }
}

thread_local! {
    static RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_entropy())
}

fn random(game: &Game, nots: &mut Option<Vec<Direction>>) -> Direction {
    let mut moves = game.valid_moves(0).collect::<Vec<Direction>>();
    if let Some(nots) = nots {
        for not in nots {
            moves.retain(|dir| *dir != *not);
        }
    }
    if moves.is_empty() {
        return *game
            .valid_moves(0)
            .collect::<Vec<Direction>>()
            .first()
            .unwrap_or(&Direction::Up);
    }
    move_check(
        game,
        *RNG.with_borrow_mut(|rng| moves.iter().choose(rng).unwrap_or(&Direction::Up)),
        nots,
    )
}

fn move_check(game: &Game, r#move: Direction, nots: &mut Option<Vec<Direction>>) -> Direction {
    let my = &game.snakes[0];
    let future_pos = my.head().apply(r#move);
    for snake in &game.snakes[1..] {
        if snake.body.len() >= my.body.len()
            && Direction::all()
                .iter()
                .any(|dir| snake.head().apply(*dir) == future_pos)
        {
            match nots {
                Some(nots) => nots.push(r#move),
                None => *nots = Some(vec![r#move]),
            }
            return random(game, nots);
        }
    }

    r#move
}
