mod cf;
mod cli;
mod environment;
mod exec;
mod login;
mod settings;
mod subcommands;

#[macro_use]
extern crate log;

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    cli::parse()
}
