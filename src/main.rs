mod environment;
mod settings;
mod cli;
mod subcommands;
mod login;
mod cf;
#[macro_use]
extern crate log;

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    cli::parse()
}