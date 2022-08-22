mod settings;

use std::process::{self, Command};

use settings::Settings;
use clap::{Parser, Subcommand};

use crate::settings::Environment;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(name = "cfe")]
#[clap(bin_name = "cfe")]
struct Cfe {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add, Remove, List environment (example cf-dev)
    Environment {
        #[clap(subcommand)]
        environmentCommands: EnvironmentCommands
    },
    /// Login to one of the cloud foundry environments
    Login {
        /// Name of the environment (example cf-dev)
        name: String
    }
}

#[derive(Subcommand, Debug)]
enum EnvironmentCommands {
    /// Add an environment to the environment list
    Add {
        name: String,
        url: String,
        #[clap(long)]
        sso: bool,
        #[clap(long)]
        skip_ssl_validation: bool,
    },
    #[clap(visible_alias = "rm")]
    /// Remove an environment to the environment list
    Remove {
        name: String,
    },
    /// List all the environment you stored
    List
}

fn main() {
    let mut settings = match Settings::new() {
        Some(some) => some,
        None => panic!("could not find or write settings file"),
    };
    let cfe = Cfe::parse();

    match &cfe.command {
        Some(Commands::Environment { environmentCommands }) => {
            match environmentCommands {
                EnvironmentCommands::Add { name, url, sso, skip_ssl_validation } => {
                    settings.add(Environment {
                        name: name.clone(),
                        url: url.clone(),
                        sso: sso.clone(),
                        skip_ssl_validation: skip_ssl_validation.clone(),
                });
                    settings.save();
                },
                EnvironmentCommands::Remove { name } => {
                    settings.remove(name);
                    settings.save();
                },
                EnvironmentCommands::List => {
                    println!("{:#?}", settings)
                },
            }
        }
        Some(Commands::Login { name }) => {
            let environment = settings.get_by_environment_by_name(name);
            if let Some(some) = environment {
                let mut cf = Command::new("cf");
                if some.skip_ssl_validation {
                    cf.arg("--skip-ssl-validation");
                }
                if some.sso {
                    cf.arg("--sso");
                }
            } else {
                println!("could not find {:#?} in environment list {:#?}", name, settings.get_environments());
                process::exit(1);
            }
            
        }
        Some(some) => {
            println!("some {:#?}", some);
        }
        None => {

        }
    }
}