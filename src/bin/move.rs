use log::info;

use hadar::agents::*;
use hadar::env::GameRequest;
use hadar::game::*;
use hadar::logging;

use clap::Parser;

#[derive(Parser)]
#[clap(version, author, about = "Simulate a move for an agent.")]
struct Opts {
    /// Default configuration.
    #[clap(long, default_value_t)]
    config: Agent,
    /// JSON Game request.
    #[clap(value_parser = parse_request)]
    request: GameRequest,
    /// Time in ms that is subtracted from the game timeouts.
    #[clap(long, default_value_t = 200)]
    latency: usize,
}

fn parse_request(s: &str) -> Result<GameRequest, serde_json::Error> {
    serde_json::from_str(s)
}

#[tokio::main]
async fn main() {
    logging();

    let Opts {
        config,
        request,
        latency,
    } = Opts::parse();

    let game = Game::from_request(&request);
    info!("{config:?}");
    info!("{game:?}");

    let step = config.step(&request, latency as _).await;

    info!("Step: {step:?}");
}
