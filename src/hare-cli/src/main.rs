mod cli;
mod control_plane;
mod logging;

use clap::Parser;

#[tokio::main]
async fn main() {
    cli::Cli::parse().run().await
}
