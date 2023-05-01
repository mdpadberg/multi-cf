use crate::{environment, subcommands::Subcommands};
use anyhow::{bail, Context, Result};
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Generator};
use lib::{
    cf::login, exec::exec, options::Options, settings::Settings,
};
use std::{io, path::PathBuf, sync::Arc};

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
    #[clap(long, global = true)]
    cf_binary_name: Option<String>,
}

pub async fn parse() -> Result<()> {
    let mcf: Mcf = Mcf::parse();

    let options = Options::new(mcf.cf_binary_name, mcf.override_path);

    let settings: Settings = Settings::load(&options)?;
    match &mcf.command {
        Subcommands::Environment {
            environment_commands,
        } => environment::match_environment(&settings, &options, environment_commands),
        Subcommands::Login { name } => {
            login(&settings, &options, name, &PathBuf::from(&options.mcf_home)).await
        }
        Subcommands::Exec { names, command, sequential_mode } => {
            exec(
                &settings,
                Arc::new(options.clone()),
                names,
                Arc::new(command.to_vec()),
                Arc::new(
                    dirs::home_dir()
                        .context("Could not find home dir")?
                        .join(".cf"),
                ),
                Arc::new(PathBuf::from(options.mcf_home)),
                sequential_mode
            )
            .await
        }
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
