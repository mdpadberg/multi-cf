mod cli;
mod environment;
mod subcommands;
extern crate log;

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    cli::parse()
}
