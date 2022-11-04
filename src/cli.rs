use crate::{environment, exec, login, settings::Settings, subcommands::Subcommands};
use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Generator};
use std::{io, path::PathBuf};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(name = "mcf")]
#[clap(bin_name = "mcf")]
struct Mcf {
    #[clap(subcommand)]
    command: Subcommands,

    /// Overwrite mcf config path
    #[clap(long, global = true)]
    override_path: Option<String>,

    /// Overwrite binary name for cloudfoundry cli (for example: "cf8")
    #[clap(long, default_value = "cf", global = true)]
    cf_binary_name: String,
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
        Subcommands::Login { name } => {
            login::to_cf(&settings, mcf.cf_binary_name, override_path.as_ref(), name)
        }
        Subcommands::Exec { names, command } => exec::cf_command(
            &settings,
            mcf.cf_binary_name,
            override_path.as_ref(),
            names,
            command,
        ),
        Subcommands::Completion { shell } => {
            let mut cmd = Mcf::command();
            eprintln!("Generating completion file for {:?}...", shell);
            print_completions(*shell, &mut cmd)
        }
    }
}

fn print_completions<G: Generator>(gen: G, cmd: &mut clap::builder::Command) -> Result<()> {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
    Ok(())
}
