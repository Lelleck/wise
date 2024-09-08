# Wise

Wise is a monitoring tool for Hell Let Loose. 
It hooks directly into the servers RCON connection and continously polls commands related to in-game events,
emitting them in a consistent format.

It polls and extracts information from the following commands:

 - `ShowLog`: Detects when new logs appears and emits them. 
 - `PlayerInfo`: Detects all changes related to a player such as their unit and role.
 - `GameState`: Detects changes in the game mainly the score between teams.

## Setup

1. **Install Rust**

Wise is written in Rust and the project currently does not supply compiled executables. 
As such to run Wise an installation is required, you can download the current version from here: https://www.rust-lang.org/tools/install.

2. **Configuration**

Wise is configured through one primary way, the config file. 
**NOTE: Configuration via environment variables is not currently supported.**

To run the application copy the `config.toml` file as `dev.config.toml`, all files starting in `.dev` are ignored by Git.
Reference the file to see what you need to set.

3. **Running**

As the project currently does not provide binaries users will have to compile it themselves. 
Luckily Cargo the manager of Rust provides a great user experience and compilation plus executing can be triggered via the `cargo run` command. 
Before running make sure to be in the `wise/` directory where the config files should be located. 

Executing `cargo run --release -- dev.config.toml` will build the entire application and execute it in release mode.
Initial compilation times of Rust are quite extensive as it compiles *all* dependencies, this means it will take some time to start.
For quicker compilation times, during development for example, omitting the `--release` flag will yield a *very* significant boost.
