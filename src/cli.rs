use crate::{environment, settings::Settings, subcommands::Subcommands, login};
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
        } => match environment_commands {
            crate::environment::EnvironmentCommands::Add {
                name,
                url,
                sso,
                skip_ssl_validation,
            } => environment::add(
                &settings,
                override_path.as_ref(),
                name,
                url,
                sso,
                skip_ssl_validation,
            ),
            crate::environment::EnvironmentCommands::Remove { name } => {
                environment::remove(&settings, override_path.as_ref(), name)
            }
            crate::environment::EnvironmentCommands::List => environment::list(&settings),
        },
        Subcommands::Login { name } => login::to_cf(&settings, override_path.as_ref(), name),
        Subcommands::Exec { names, command } => todo!(),
        Subcommands::Completion { shell } => todo!(),
    }
}
