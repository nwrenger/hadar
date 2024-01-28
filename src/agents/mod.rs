use std::str::FromStr;
use std::string::ToString;

mod astar;
pub use astar::*;
mod random;
pub use random::*;

use crate::game::Game;

use super::env::{GameRequest, MoveResponse};

const MAX_BOARD_SIZE: usize = 19;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Agent {
    AStar(StarAgent),
    Random(RandomAgent),
}

impl Default for Agent {
    fn default() -> Self {
        Self::AStar(StarAgent)
    }
}

impl Agent {
    pub async fn step(&self, request: &GameRequest, latency: u64) -> MoveResponse {
        let game = Game::from_request(request);
        let timeout = request.game.timeout.saturating_sub(latency);

        self.step_internal(timeout, &game).await
    }

    pub async fn step_internal(&self, _timeout: u64, game: &Game) -> MoveResponse {
        if game.grid.width > MAX_BOARD_SIZE || game.grid.height > MAX_BOARD_SIZE {
            return RandomAgent.step(game).await;
        }

        match self {
            Agent::AStar(agent) => agent.step(game).await,
            Agent::Random(agent) => agent.step(game).await,
        }
    }
}

impl FromStr for Agent {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

impl ToString for Agent {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}
