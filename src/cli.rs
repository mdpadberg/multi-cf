use crate::{environment, login, settings::Settings, subcommands::Subcommands};
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(name = "mcf")]
#[clap(bin_name = "mcf")]
struct Mcf {
    #[clap(subcommand)]
    command: Subcommands,

    #[clap(long, global = true)]
    override_path: Option<String>,
}

pub fn parse() -> Result<()> {
    let mcf: Mcf = Mcf::parse();
    let override_path: Option<PathBuf> = mcf.override_path.map(|string| PathBuf::from(string));
    let settings: Settings = Settings::load(override_path.as_ref())?;
    match &mcf.command {
        Subcommands::Environment {
            environment_commands,
        } => {
            environment::match_environment(&settings, override_path.as_ref(), environment_commands)
        }
        Subcommands::Login { name } => login::to_cf(&settings, override_path.as_ref(), name),
        Subcommands::Exec { names, command } => todo!(),
        Subcommands::Completion { shell } => todo!(),
    }
}
