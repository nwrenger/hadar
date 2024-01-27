# hadar

This crate contains some simple battlesnake agents. The structure of this project is forked from [this crate](https://github.com/wrenger/snork/).

## Usage

### Running the Server

First, the rust toolchain has to be installed (https://www.rust-lang.org/learn/get-started).

Starting the server:

```bash
cargo run --release --bin server -- [-h] [--host <ip:port>] [--config <json>]
```

> There are additional options for `--runtime` and visual representation of the snake (`--head`, `--tail`, `--color`).
> Run `cargo run --release --bin server -- -h` to see all the commandline options.

`config` defines the agent to be used (`AStar`, `Random`) and configures the agent's heuristic.
The default config for the `AStar` agent is, for example:

```json
{
  "AStar": null
}
```

> If a config parameter (like f.e. `health`) is excluded the default value is used.

### Simulating Configs

This tool can be used to simulate different configurations.
These configurations specify the agent and its hyperparameters.
If no parameters are provided, the default values for the agent are used.
The number of simulated games can be specified with `--game-count`.
Use `-h` for more information about other arguments to define the board size and game rules.

The example below simulates the `AStar` and `Random` agents for 10 games:

```bash
cargo run --release --bin simulate -- '{"AStar":{ }}' '{"Random":{ }}' --game-count 10
```

The last line of the standard output contains the number of wins of the first snake and the total number of games played:

```
Result: 3/10
```

### Testing moves

The `move` program outputs the chosen move for a given game state and agent configuration.
This can be useful for debugging situational bugs.
The game input can be downloaded from the [battlesnake](https://play.battlesnake.com) with this [Firefox extension](https://addons.mozilla.org/firefox/addon/battlesnake-downloader/).

```bash
cargo run --release --bin move -- [--config <json>] [--runtime] <json>
```

### Running tests & benchmarks

There are multiple tests for the different modules that can be run, as shown below.
For more information on unit testing in Rust, see https://doc.rust-lang.org/book/ch11-01-writing-tests.html.

```bash
cargo test -- [--nocapture] [testname]
```

Besides the functional tests, there are several performance benchmarks.
They are executed with the release config with compiler and linker optimizations.
The criterion benchmark runner tracks the execution times from previous runs and reports any improvements or degradations.

```bash
cargo bench -- [testname]
```