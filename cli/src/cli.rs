use crate::{environment, subcommands::Subcommands};
use anyhow::{bail, Context, Result};
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Generator};
use lib::{
    cf::login, exec::exec_parallel, exec::exec_sequential, options::Options, settings::Settings,
};
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
            login(&settings, &options, name, &PathBuf::from(&options.mcf_home))
        }
        Subcommands::Exec { names, command } => {
            match exec_parallel(
                &settings,
                &options,
                names,
                command,
                &dirs::home_dir()
                    .context("Could not find home dir")?
                    .join(".cf"),
                &PathBuf::from(&options.mcf_home),
            )
            .await
            {
                Ok(_) => Ok(()),
                Err(error) => {
                    println!("🤖🤖🤖 We noticed you are using a command which requires interactive mode 🤖🤖🤖");
                    println!("🤖🤖🤖 Switching to sequential interactive mode 🤖🤖🤖");
                    if error.to_string() == "We need to switch to interactive mode" {
                        exec_sequential(
                            &settings,
                            &options,
                            names,
                            command,
                            &dirs::home_dir()
                                .context("Could not find home dir")?
                                .join(".cf"),
                            &PathBuf::from(&options.mcf_home),
                        )
                        .await
                    } else {
                        bail!(error);
                    }
                }
            }
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
