mod settings;

use colored::*;
use rand::{
    distributions::{Distribution, Standard},
    Rng, random,
};
use std::{
    io::{BufRead, BufReader},
    ops::RangeBounds,
    process::{self, Command, Stdio},
    slice::SliceIndex,
};

use rayon::iter::{ParallelIterator, IntoParallelIterator};
use clap::{Parser, Subcommand};
use dirs::data_dir;
use settings::Settings;

use crate::settings::Environment;

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
        environmentCommands: EnvironmentCommands,
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

#[derive(Debug)]
enum RandomColor {
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
}

impl Distribution<RandomColor> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> RandomColor {
        match rng.gen_range(0..=12) {
            0 => RandomColor::Red,
            1 => RandomColor::Green,
            2 => RandomColor::Yellow,
            3 => RandomColor::Blue,
            4 => RandomColor::Magenta,
            5 => RandomColor::Cyan,
            6 => RandomColor::White,
            7 => RandomColor::BrightRed,
            8 => RandomColor::BrightGreen,
            9 => RandomColor::BrightYellow,
            10 => RandomColor::BrightBlue,
            11 => RandomColor::BrightMagenta,
            _ => RandomColor::BrightWhite,
        }
    }
}

impl RandomColor {
    pub fn to_colored_crate(&self) -> Color {
        match self {
            RandomColor::Red => Color::Red,
            RandomColor::Green => Color::Green,
            RandomColor::Yellow => Color::Yellow,
            RandomColor::Blue => Color::Blue,
            RandomColor::Magenta => Color::Magenta,
            RandomColor::Cyan => Color::Cyan,
            RandomColor::White => Color::White,
            RandomColor::BrightRed => Color::BrightRed,
            RandomColor::BrightGreen => Color::BrightGreen,
            RandomColor::BrightYellow => Color::BrightYellow,
            RandomColor::BrightBlue => Color::BrightBlue,
            RandomColor::BrightMagenta => Color::BrightMagenta,
            RandomColor::BrightCyan => Color::BrightCyan,
            RandomColor::BrightWhite => Color::BrightWhite,
        }
    }
}

fn main() {
    let mut settings = match Settings::new() {
        Some(some) => some,
        None => panic!("could not find or write settings file"),
    };
    let mcf: Mcf = Mcf::parse();

    match &mcf.command {
        Some(Commands::Environment {
                 environmentCommands,
             }) => match environmentCommands {
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
                let mut cf = Command::new("cf");
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
                child.wait();
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

            envs.into_par_iter().for_each(|env| {
                let mut cf_home = data_dir().expect("no data dir");
                cf_home.push("mcf");
                cf_home.push("homes");
                cf_home.push(&env.1);

                let stdout = Command::new("cf")
                    .env("CF_HOME", cf_home)
                    .args(command)
                    .stdout(Stdio::piped())
                    .spawn()
                    .expect("Could not spawn child process.")
                    .stdout
                    .expect("Could not capture standard output.");

                let random_color: RandomColor = rand::random();
                let color = random_color.to_colored_crate();

                BufReader::new(stdout)
                    .lines()
                    .filter_map(|line| line.ok())
                    .for_each(|line| println!("{}: {}", &env.1.color(color), line.color(color)));
            });
        }
        None => panic!("Something went wrong, please contact the developers")
    }
}
