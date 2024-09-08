# Wise

Wise is a monitoring tool for Hell Let Loose. 
It hooks directly into the servers RCON connection and continously polls commands related to in-game events,
emitting them in a consistent format.

It polls and extracts information from the following commands:

 - `ShowLog`: Detects when new logs appears and emits them. **(WIP: only few events are currently supported)**
 - `PlayerInfo`: Detects all changes related to a player such as their unit and role.
 - `GameState`: Detects changes in the game mainly the score between teams. **(WIP: not currently implemented)**

## Setup

1. **Install Rust**

Wise is written in Rust and the project currently does not supply compiled executables. 
As such to run Wise an installation is required, you can download the current version from here: https://www.rust-lang.org/tools/install.

2. **Configuration**

Wise is configured through two primary ways, the CLI config and file config. 
Currently the CLI configuration is exclusively used to point to the config file which holds the entire configuration.
**NOTE: Configuration via environment variables is not currently supported.**

To run the application copy your RCON credentials into the `wise-config.toml` file. 
As this file also acts as an example file it is tracked by Git and when updating the codebase may result in a conflict.
This may be changed in the future, for now just keep it in mind.

If you want to develop and later commit to this projects its recommended to consolidate your configuration in a `dev.toml`.
All files ending in `dev.toml` are ignored and not tracked by Git.

3. **Running**

As the project currently does not provide binaries users will have to compile it themselves. 
Luckily Cargo the manager of Rust provides a great user experience and compilation plus executing can be triggered via the `cargo run` command. 
Before running make sure to be in the `wise/` directory where the config files should be located. 

Executing `cargo run --release -- wise-config.toml` will build the entire application and execute it in release mode.
Initial compilation times of Rust are quite extensive as it compiles *all* dependencies, this means it will take some time to start.
For quicker compilation times, during development for example, omitting the `--release` flag will yield a *very* significant boost.
