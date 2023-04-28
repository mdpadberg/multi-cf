mod cli;
mod environment;
mod subcommands;
extern crate log;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    cli::parse().await
}
