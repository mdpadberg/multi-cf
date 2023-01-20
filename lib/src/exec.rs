use crate::cf::stdout;
use crate::environment::Environment;
use crate::options::Options;
use crate::settings::Settings;
use anyhow::{bail, Result, Context};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::path::PathBuf;
use std::process::ChildStdout;
use std::io::{BufRead, BufReader};

pub fn exec(
    settings: &Settings,
    options: &Options,
    names: &String,
    command: &Vec<String>,
    original_cf_home: &PathBuf,
    mcf_folder: &PathBuf,
) -> Result<()> {
    let cf_binary_name = &options.cf_binary_name;
    let input_enviroments: Vec<(Option<Environment>, String)> = names
        .split(',')
        .map(|s| s.to_string())
        .map(|env| (settings.get_environment_by_name(&env), env))
        .collect();

    for env in input_enviroments.iter() {
        if env.0.is_none() {
            bail!(
                "could not find {:#?} in environment list {:#?}",
                env.1,
                settings.environments
            );
        }
    }

    let max_chars = input_enviroments
        .iter()
        .map(|(_env, env_name)| env_name.len())
        .max()
        .context("environment name should have length")?;

    input_enviroments
        .into_par_iter()
        .try_for_each(|(_env, env_name)| -> Result<()> {
            let stdout: ChildStdout =
                stdout(cf_binary_name, command, &env_name, original_cf_home, mcf_folder)?;
            let whitespace_length = max_chars - env_name.len();
            let whitespace = (0..=whitespace_length).map(|_| " ").collect::<String>();

            BufReader::new(stdout)
                .lines()
                .filter_map(|line| line.ok())
                .for_each(|line| println!("{}{}| {}", &env_name, whitespace, line));
            Ok(())
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_exec_could_not_find_env_in_list() {
        let tempdir: PathBuf = tempdir().unwrap().into_path();
        let result = exec(
            &Settings {
                environments: vec![Environment {
                    name: "p01".to_string(),
                    url: "url".to_string(),
                    sso: false,
                    skip_ssl_validation: false,
                }],
            },
            &Options {
                cf_binary_name: String::from("echo"),
                mcf_home: tempdir.to_str().unwrap().to_string(),
            },
            &String::from("p01,p02"),
            &vec![String::from("")],
            &PathBuf::from(""),
            &PathBuf::from(""),
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(), 
            "could not find \"p02\" in environment list [\n    Environment {\n        name: \"p01\",\n        url: \"url\",\n        sso: false,\n        skip_ssl_validation: false,\n    },\n]"
        );
    }

    #[test]
    fn test_exec_environment_should_have_length() {
        let tempdir: PathBuf = tempdir().unwrap().into_path();
        let _ = std::fs::create_dir_all(tempdir.join(".cf").join("plugins"));
        let result = exec(
            &Settings {
                environments: vec![Environment {
                    name: "p01".to_string(),
                    url: "url".to_string(),
                    sso: false,
                    skip_ssl_validation: false,
                }],
            },
            &Options {
                cf_binary_name: String::from("echo"),
                mcf_home: tempdir.to_str().unwrap().to_string(),
            },
            &String::from("p01"),
            &vec![String::from("hello")],
            &tempdir.join(".cf"),
            &tempdir.join("test-exec-environment-should-have-length"),
        );
        assert!(result.is_ok());
    }
}
