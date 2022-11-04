use crate::{cf, environment::Environment, settings::Settings};
use anyhow::Result;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::{
    io::{BufRead, BufReader},
    path::PathBuf,
    process,
};

pub fn cf_command(
    settings: &Settings,
    cf_binary_name: String,
    override_path: Option<&PathBuf>,
    names: &String,
    command: &Vec<String>,
) -> Result<()> {
    let input_enviroments: Vec<(Option<Environment>, String)> = names
        .split(',')
        .map(|s| s.to_string())
        .map(|env| (settings.get_environment_by_name(&env), env))
        .collect();

    for env in input_enviroments.iter() {
        if env.0.is_none() {
            println!(
                "could not find {:#?} in environment list {:#?}",
                env.1, settings.environments
            );
            process::exit(1);
        }
    }

    let max_chars = input_enviroments
        .iter()
        .map(|(_env, env_name)| env_name.len())
        .max()
        .expect("environment name should have length");

    input_enviroments
        .into_par_iter()
        .for_each(|(_env, env_name)| {
            let stdout = cf::exec(&cf_binary_name, override_path, &env_name, command);
            let whitespace_length = max_chars - env_name.len();
            let whitespace = (0..=whitespace_length).map(|_| " ").collect::<String>();

            BufReader::new(stdout)
                .lines()
                .filter_map(|line| line.ok())
                .for_each(|line| println!("{}{}| {}", &env_name, whitespace, line));
        });

    Ok(())
}
