use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::task::JoinSet;

use crate::cf::{child_tokio, CFSubCommandsThatRequireSequentialMode};
use crate::environment::Environment;
use crate::options::Options;
use crate::settings::Settings;

pub async fn exec(
    settings: &Settings,
    options: Arc<Options>,
    names: &str,
    command: Arc<Vec<String>>,
    original_cf_home: Arc<PathBuf>,
    mcf_folder: Arc<PathBuf>,
    sequential_mode: &bool,
) -> Result<()> {
    if CFSubCommandsThatRequireSequentialMode::check_if_contains(command.join(""))
        || *sequential_mode
    {
        exec_sequential(
            settings,
            options,
            names,
            command,
            original_cf_home,
            mcf_folder,
        )
        .await
    } else {
        exec_parallel(
            settings,
            options,
            names,
            command,
            original_cf_home,
            mcf_folder,
        )
        .await
    }
}

async fn exec_sequential(
    settings: &Settings,
    options: Arc<Options>,
    names: &str,
    command: Arc<Vec<String>>,
    original_cf_home: Arc<PathBuf>,
    mcf_folder: Arc<PathBuf>,
) -> Result<()> {
    let input_environments = input_environments(names, settings);
    check_if_all_environments_are_known(&input_environments, settings)?;
    for (_env, env_name) in input_environments {
        println!(
            "------------------ NOW ENVIRONMENT {} ------------------",
            env_name
        );
        let options = options.clone();
        let command = command.clone();
        let original_cf_home = original_cf_home.clone();
        let mcf_folder = mcf_folder.clone();
        let child: tokio::process::Child = child_tokio(
            options,
            command,
            &env_name,
            original_cf_home,
            mcf_folder,
            &true,
        )?;
        child.wait_with_output().await?;
    }
    Ok(())
}

async fn exec_parallel(
    settings: &Settings,
    options: Arc<Options>,
    names: &str,
    command: Arc<Vec<String>>,
    original_cf_home: Arc<PathBuf>,
    mcf_folder: Arc<PathBuf>,
) -> Result<()> {
    let input_environments = input_environments(names, settings);
    check_if_all_environments_are_known(&input_environments, settings)?;
    let mut tasks: JoinSet<Result<()>> = JoinSet::new();
    let max_chars = max_environment_name_length(&input_environments)?;
    for (_env, env_name) in input_environments {
        let options = options.clone();
        let command = command.clone();
        let original_cf_home = original_cf_home.clone();
        let mcf_folder = mcf_folder.clone();
        tasks.spawn(async move {
            let whitespace_length = max_chars - env_name.len();
            let whitespace = (0..=whitespace_length).map(|_| " ").collect::<String>();
            let child: tokio::process::Child = child_tokio(
                options,
                command,
                &env_name,
                original_cf_home,
                mcf_folder,
                &false,
            )?;
            let stdout = child.stdout.context("exec: no stdout")?;
            let mut lines = BufReader::new(stdout).lines();
            while let Some(line) = lines.next_line().await? {
                println!("{}{}| {}", &env_name, whitespace, line);
            }
            Ok(())
        });
    }
    while let Some(result) = tasks.join_next().await {
        result??;
    }
    Ok(())
}

fn max_environment_name_length(
    input_environments: &[(Option<Environment>, String)],
) -> Result<usize, anyhow::Error> {
    let result = input_environments
        .iter()
        .map(|(_env, env_name)| env_name.len())
        .max()
        .context("environment name should have length")?;
    Ok(result)
}

fn check_if_all_environments_are_known(
    input_environments: &[(Option<Environment>, String)],
    settings: &Settings,
) -> Result<()> {
    for env in input_environments.iter() {
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

fn input_environments(names: &str, settings: &Settings) -> Vec<(Option<Environment>, String)> {
    names
        .split(',')
        .map(|s| s.to_string())
        .map(|env| (settings.get_environment_by_name(&env), env))
        .collect::<Vec<(Option<Environment>, String)>>()
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use gag::BufferRedirect;
    use tempfile::tempdir;

    use super::*;

    #[tokio::test]
    async fn test_exec_could_not_find_env_in_list() {
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
            Arc::new(Options {
                cf_binary_name: String::from("echo"),
                mcf_home: tempdir.to_str().unwrap().to_string(),
            }),
            &String::from("p01,p02"),
            Arc::new(vec![String::from("")]),
            Arc::new(PathBuf::from("")),
            Arc::new(PathBuf::from("")),
            &false,
        )
        .await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "could not find \"p02\" in environment list [\n    Environment {\n        name: \"p01\",\n        url: \"url\",\n        sso: false,\n        skip_ssl_validation: false,\n    },\n]"
        );
    }

    #[tokio::test]
    async fn test_exec_environment_should_have_length() {
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
            Arc::new(Options {
                cf_binary_name: String::from("echo"),
                mcf_home: tempdir.to_str().unwrap().to_string(),
            }),
            &String::from("p01"),
            Arc::new(vec![String::from("hello")]),
            Arc::new(tempdir.join(".cf")),
            Arc::new(tempdir.join("test-exec-environment-should-have-length")),
            &false,
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_exec() {
        test_if_run_in_sequential_mode_when_boolean_is_true().await;
        test_if_run_in_sequential_mode_when_boolean_is_false_but_command_is_in_enum_list().await;
        test_if_run_in_parallel_mode().await;
    }

    async fn test_if_run_in_sequential_mode_when_boolean_is_true() {
        let mut buf = BufferRedirect::stdout().unwrap();
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
            Arc::new(Options {
                cf_binary_name: String::from("echo"),
                mcf_home: tempdir.to_str().unwrap().to_string(),
            }),
            &String::from("p01"),
            Arc::new(vec![String::from("hello")]),
            Arc::new(tempdir.join(".cf")),
            Arc::new(tempdir.join("test-exec-environment-should-have-length")),
            &true,
        )
        .await;
        assert!(result.is_ok());
        let mut output = String::new();
        buf.read_to_string(&mut output).unwrap();
        drop(buf);
        assert!(!output.contains("p01 | \n"));
        assert!(output.contains("------------------ NOW ENVIRONMENT p01 ------------------\n"));
    }

    async fn test_if_run_in_sequential_mode_when_boolean_is_false_but_command_is_in_enum_list() {
        let mut buf = BufferRedirect::stdout().unwrap();
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
            Arc::new(Options {
                cf_binary_name: String::from("echo"),
                mcf_home: tempdir.to_str().unwrap().to_string(),
            }),
            &String::from("p01"),
            Arc::new(vec![String::from("Delete")]),
            Arc::new(tempdir.join(".cf")),
            Arc::new(tempdir.join("test-exec-environment-should-have-length")),
            &false,
        )
        .await;
        assert!(result.is_ok());
        let mut output = String::new();
        buf.read_to_string(&mut output).unwrap();
        drop(buf);
        assert!(!output.contains("p01 | \n"));
        assert!(output.contains("------------------ NOW ENVIRONMENT p01 ------------------\n"));
    }

    async fn test_if_run_in_parallel_mode() {
        let mut buf = BufferRedirect::stdout().unwrap();
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
            Arc::new(Options {
                cf_binary_name: String::from("echo"),
                mcf_home: tempdir.to_str().unwrap().to_string(),
            }),
            &String::from("p01"),
            Arc::new(vec![String::from("Hello")]),
            Arc::new(tempdir.join(".cf")),
            Arc::new(tempdir.join("test-exec-environment-should-have-length")),
            &false,
        )
        .await;
        assert!(result.is_ok());
        let mut output = String::new();
        buf.read_to_string(&mut output).unwrap();
        drop(buf);
        assert!(!output.contains("------------------ NOW ENVIRONMENT p01 ------------------\n"));
        assert!(output.contains("p01 | Hello\n"));
    }
}
