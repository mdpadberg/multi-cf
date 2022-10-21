use crate::{environment, login, settings::Settings, subcommands::Subcommands, exec};
use anyhow::Result;
use clap::{Parser, CommandFactory};
use clap_complete::{Generator, generate};
use std::{path::PathBuf, io};

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
        Subcommands::Exec { names, command } => exec::cf_command(&settings, override_path.as_ref(), names, command),
        Subcommands::Completion  { shell } => {
            let mut cmd = Mcf::command();
            eprintln!("Generating completion file for {:?}...", shell);
            print_completions(*shell, &mut cmd)
        }
    }
}

fn print_completions<G: Generator>(gen: G, cmd: &mut clap::builder::Command) -> Result<()>  {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
    Ok(())
}
