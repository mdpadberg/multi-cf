use crate::{
    environment, exec, login, options::Options, settings::Settings, subcommands::Subcommands,
};
use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Generator};
use std::io;

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

pub fn parse() -> Result<()> {
    let mcf: Mcf = Mcf::parse();

    let options = Options::new(mcf.cf_binary_name, mcf.override_path);

    let settings: Settings = Settings::load(&options)?;
    match &mcf.command {
        Subcommands::Environment {
            environment_commands,
        } => environment::match_environment(&settings, &options, environment_commands),
        Subcommands::Login { name } => login::to_cf(&settings, &options, name),
        Subcommands::Exec { names, command } => {
            exec::cf_command(&settings, &options, names, command)
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
