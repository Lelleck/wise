# Wise

Wise a middleware layer that sits between a Hell Let Loose software and your application.  
It exposes a WebSocket interface that streams all server events in real time - just connect and start receiving data.  
No need to worry about unstable behaviour the Hell Let Loose - Wise handles it.

### Features
 - **🔄 Real-time Events**  
 Get real-time events from the server including score changes, player movement, kills, etc. 
 - **🛠 Robust and Fast**  
 Built in Rust, Wise by nature is fast and stable handling potential server-side issues.
 - **🔐 Secure API**  
 Unlike the Hell Let Loose server, control access to the API to prevent unwanted access with tokens and encrypt the WebSocket connection with TLS.
 - **🔨 Take Action**  
 Wise accepts commands you send it and can execute them for you on the Hell Let Loose server.

### Planned Features
 - **💾 Save Games** - Save entire games into a common file format to review them later.
 - **🧠 Data Inference** - Extrapolate additional data from context such as whether a player may be in a vehicle.
 - **📋 Transparency and Accountability** - CRCON integration to transparently record actions such as kicks and bans.

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
