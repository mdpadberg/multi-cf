use crate::cf::{child, child_tokio};
use crate::environment::Environment;
use crate::options::Options;
use crate::settings::Settings;
use anyhow::{bail, Context, Result};
use futures::future;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, ChildStderr, ChildStdout};

pub async fn exec_parallel(
    settings: &Settings,
    options: &Options,
    names: &String,
    command: &Vec<String>,
    original_cf_home: &PathBuf,
    mcf_folder: &PathBuf,
) -> Result<()> {
    let input_enviroments = input_enviroments(names, settings);
    check_if_all_enviroments_are_known(&input_enviroments, settings)?;
    let max_chars = max_enviroment_name_length(&input_enviroments)?;
    let tasks: Vec<tokio::task::JoinHandle<Result<(), anyhow::Error>>> = input_enviroments
        .into_iter()
        .map(|(_env, env_name)| {
            let child: Child = child(
                &options.cf_binary_name,
                command,
                &env_name,
                original_cf_home,
                mcf_folder,
            )?;
            let whitespace_length = max_chars - env_name.len();
            let whitespace = (0..=whitespace_length).map(|_| " ").collect::<String>();
            let stdout = child.stdout.context("context")?;
            let stderr = child.stderr.context("context")?;
            Ok(vec![
                tokio::spawn(async move { print_stdout(env_name, whitespace, stdout).await }),
                tokio::spawn(async move {
                    bail_if_stderr_contains_interactive_mode_error(stderr).await
                }),
            ])
        })
        .collect::<Result<Vec<Vec<tokio::task::JoinHandle<Result<(), anyhow::Error>>>>>>()?
        .into_iter()
        .flatten()
        .collect();

    let we_need_to_switch = future::join_all(tasks)
        .await
        .into_iter()
        .filter(|result| result.is_ok())
        .map(|result| result.unwrap())
        .filter(|result| result.is_err())
        .any(|result_with_error| match result_with_error {
            Ok(_) => false,
            Err(err) => {
                if err.to_string() == "We need to switch to interactive mode" {
                    true
                } else {
                    false
                }
            }
        });

    if we_need_to_switch {
        bail!("We need to switch to interactive mode")
    }
    Ok(())
}

pub async fn exec_sequential(
    settings: &Settings,
    options: &Options,
    names: &String,
    command: &Vec<String>,
    original_cf_home: &PathBuf,
    mcf_folder: &PathBuf,
) -> Result<()> {
    let input_enviroments = input_enviroments(names, settings);
    check_if_all_enviroments_are_known(&input_enviroments, settings)?;
    for (_env, env_name) in input_enviroments {
        println!("------------------ NOW ENVIROMENT {} ------------------", env_name);
        let child: tokio::process::Child = child_tokio(
            &options.cf_binary_name,
            command,
            &env_name,
            original_cf_home,
            mcf_folder,
        )?;
        child.wait_with_output().await?;
    }
    Ok(())
}

fn max_enviroment_name_length(input_enviroments: &Vec<(Option<Environment>, String)>) -> Result<usize, anyhow::Error> {
    Ok(input_enviroments
        .iter()
        .map(|(_env, env_name)| env_name.len())
        .max()
        .context("environment name should have length")?)
}

fn check_if_all_enviroments_are_known(input_enviroments: &Vec<(Option<Environment>, String)>, settings: &Settings) -> Result<()> {
    for env in input_enviroments.iter() {
        if env.0.is_none() {
            bail!(
                "could not find {:#?} in environment list {:#?}",
                env.1,
                settings.environments
            );
        }
    }
    Ok(())
}

fn input_enviroments(names: &String, settings: &Settings) -> Vec<(Option<Environment>, String)> {
    names
        .split(',')
        .map(|s| s.to_string())
        .map(|env| (settings.get_environment_by_name(&env), env))
        .collect::<Vec<(Option<Environment>, String)>>()
}

async fn print_stdout(env_name: String, whitespace: String, stdout: ChildStdout) -> Result<()> {
    BufReader::new(stdout)
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| println!("{}{}| {}", env_name, whitespace, line));
    Ok(())
}

async fn bail_if_stderr_contains_interactive_mode_error(stderr: ChildStderr) -> Result<()> {
    for line in BufReader::new(stderr).lines() {
        if line.is_ok() && line?.contains("inappropriate ioctl for device") {
            bail!("We need to switch to interactive mode")
        }
    }
    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use tempfile::tempdir;

//     #[test]
//     fn test_exec_could_not_find_env_in_list() {
//         let tempdir: PathBuf = tempdir().unwrap().into_path();
//         let result = exec(
//             &Settings {
//                 environments: vec![Environment {
//                     name: "p01".to_string(),
//                     url: "url".to_string(),
//                     sso: false,
//                     skip_ssl_validation: false,
//                 }],
//             },
//             &Options {
//                 cf_binary_name: String::from("echo"),
//                 mcf_home: tempdir.to_str().unwrap().to_string(),
//             },
//             &String::from("p01,p02"),
//             &vec![String::from("")],
//             &PathBuf::from(""),
//             &PathBuf::from(""),
//         );
//         assert!(result.is_err());
//         assert_eq!(
//             result.unwrap_err().to_string(),
//             "could not find \"p02\" in environment list [\n    Environment {\n        name: \"p01\",\n        url: \"url\",\n        sso: false,\n        skip_ssl_validation: false,\n    },\n]"
//         );
//     }

//     #[test]
//     fn test_exec_environment_should_have_length() {
//         let tempdir: PathBuf = tempdir().unwrap().into_path();
//         let _ = std::fs::create_dir_all(tempdir.join(".cf").join("plugins"));
//         let result = exec(
//             &Settings {
//                 environments: vec![Environment {
//                     name: "p01".to_string(),
//                     url: "url".to_string(),
//                     sso: false,
//                     skip_ssl_validation: false,
//                 }],
//             },
//             &Options {
//                 cf_binary_name: String::from("echo"),
//                 mcf_home: tempdir.to_str().unwrap().to_string(),
//             },
//             &String::from("p01"),
//             &vec![String::from("hello")],
//             &tempdir.join(".cf"),
//             &tempdir.join("test-exec-environment-should-have-length"),
//         );
//         assert!(result.is_ok());
//     }
// }
