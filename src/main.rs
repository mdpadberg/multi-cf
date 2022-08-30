use std::{
    io,
    io::{BufRead, BufReader},
    process::{self, Stdio},
};

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Generator, Shell};
use colored::Color::{
    Blue, BrightBlue, BrightCyan, BrightGreen, BrightMagenta, BrightRed, BrightWhite, BrightYellow,
    Cyan, Green, Magenta, Red, White, Yellow,
};
use colored::*;
use dirs::data_dir;
use rayon::iter::{IntoParallelIterator, ParallelIterator, IndexedParallelIterator};

use settings::Settings;

use crate::settings::Environment;

mod settings;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(name = "mcf")]
#[clap(bin_name = "mcf")]
struct Mcf {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add, Remove, List environment (example cf-dev)
    #[clap(visible_alias = "env")]
    Environment {
        #[clap(subcommand)]
        environment_commands: EnvironmentCommands,
    },
    /// Login to one of the cloud foundry environments
    #[clap(visible_alias = "l")]
    Login {
        /// Name of the environment (example "cf-dev")
        name: String,
    },
    /// Execute command on cloud foundry environment
    #[clap(visible_alias = "e", trailing_var_arg = true)]
    Exec {
        /// Names of the environments (example "cf-dev,cf-prod")
        names: String,
        /// Command you want to execute (example "logs your-application --recent")
        command: Vec<String>,
    },
    /// Generate shell autocompletion files
    Completion {
        #[clap(arg_enum, value_parser)]
        shell: Shell,
    },
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
    Remove { name: String },
    /// List all the environment you stored
    List,
}

const COLORS: &'static [Color] = &[
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
];

fn main() {
    let mut settings = match Settings::new() {
        Some(some) => some,
        None => panic!("could not find or write settings file"),
    };
    let mcf: Mcf = Mcf::parse();

    match &mcf.command {
        Some(Commands::Environment {
            environment_commands,
        }) => match environment_commands {
            EnvironmentCommands::Add {
                name,
                url,
                sso,
                skip_ssl_validation,
            } => {
                settings.add(Environment {
                    name: name.clone(),
                    url: url.clone(),
                    sso: sso.clone(),
                    skip_ssl_validation: skip_ssl_validation.clone(),
                });
                settings.save();
            }
            EnvironmentCommands::Remove { name } => {
                settings.remove(name);
                settings.save();
            }
            EnvironmentCommands::List => {
                println!("{:#?}", settings)
            }
        },
        Some(Commands::Login { name }) => {
            let environment = settings.get_by_environment_by_name(name);
            if let Some(some) = environment {
                let mut cf = process::Command::new("cf");
                let mut cf_home = data_dir().expect("no data dir");
                cf_home.push("mcf");
                cf_home.push("homes");
                cf_home.push(some.name);
                cf.env("CF_HOME", cf_home);
                cf.arg("login").arg("-a").arg(some.url);

                if some.skip_ssl_validation {
                    cf.arg("--skip-ssl-validation");
                }
                if some.sso {
                    cf.arg("--sso");
                }
                let mut child = cf.spawn().expect("Failure in creating child process");
                let _ = child.wait();
            } else {
                println!(
                    "could not find {:#?} in environment list {:#?}",
                    name,
                    settings.get_environments()
                );
                process::exit(1);
            }
        }
        Some(Commands::Exec { names, command }) => {
            let envs: Vec<(Option<Environment>, String)> = names
                .split(",")
                .map(|s| s.to_string())
                .map(|env| (settings.get_by_environment_by_name(&env), env))
                .collect();

            for env in envs.iter() {
                if let None = env.0 {
                    println!(
                        "could not find {:#?} in environment list {:#?}",
                        env.1,
                        settings.get_environments()
                    );
                    process::exit(1);
                }
            }

            let colors: Vec<&Color> = COLORS.iter().cycle().take(envs.len()).collect();
            envs.into_par_iter().zip(colors).for_each(|(env, color)| {
                let mut cf_home = data_dir().expect("no data dir");
                cf_home.push("mcf");
                cf_home.push("homes");
                cf_home.push(&env.1);

                let stdout = process::Command::new("cf")
                    .env("CF_HOME", cf_home)
                    .args(command)
                    .stdout(Stdio::piped())
                    .spawn()
                    .expect("Could not spawn child process.")
                    .stdout
                    .expect("Could not capture standard output.");

                BufReader::new(stdout)
                    .lines()
                    .filter_map(|line| line.ok())
                    .for_each(|line| println!("{}: {}", &env.1.color(*color), line.color(*color)));
            });
        }
        Some(Commands::Completion { shell }) => {
            let mut cmd = Mcf::command();
            eprintln!("Generating completion file for {:?}...", shell);
            print_completions(*shell, &mut cmd);
        }
        None => panic!("Something went wrong, please contact the developers"),
    }
}

fn print_completions<G: Generator>(gen: G, cmd: &mut clap::builder::Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}
