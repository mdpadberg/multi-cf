use std::path::Path;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use strum::{AsRefStr, EnumIter, IntoEnumIterator};
use tokio::process::Command;

use crate::options::Options;
use crate::settings::Settings;

#[derive(Debug, EnumIter, AsRefStr)]
pub enum CFSubCommandsThatRequireSequentialMode {
    Ssh,
    Delete,
}

impl CFSubCommandsThatRequireSequentialMode {
    pub fn check_if_contains(input: String) -> bool {
        let input_lowercase = input.to_lowercase();
        CFSubCommandsThatRequireSequentialMode::iter()
            .any(|cfsubcommand| input_lowercase.contains(&cfsubcommand.as_ref().to_lowercase()))
    }
}

pub async fn login(
    settings: &Settings,
    options: &Options,
    name: &String,
    mcf_home: &Path,
    sso_passcode: &Option<String>,
    org: &Option<String>,
    space: &Option<String>,
) -> Result<()> {
    if let Some(some) = settings.environments.iter().find(|env| &env.name == name) {
        let cf_binary_name = &options.cf_binary_name;
        let mut cf: Command = cf_command_tokio(cf_binary_name, &some.name, mcf_home);
        cf.arg("login").arg("-a").arg(&some.url);
        if some.skip_ssl_validation {
            cf.arg("--skip-ssl-validation");
        }
        if let Some(some) = sso_passcode {
            cf.args(["--sso-passcode", some]);
        } else if some.sso {
            cf.arg("--sso");
        }
        if let Some(some) = org {
            cf.args(["-o", some]);
        }
        if let Some(some) = space {
            cf.args(["-s", some]);
        }
        let child = cf.spawn().expect("Failure in creating child process");
        child.wait_with_output().await?;
    } else {
        bail!(
            "could not find {:#?} in environment list {:#?}",
            name,
            settings.environments
        );
    }
    Ok(())
}

pub fn child_tokio(
    options: Arc<Options>,
    command: Arc<Vec<String>>,
    env_name: &String,
    original_cf_home: Arc<PathBuf>,
    mcf_folder: Arc<PathBuf>,
    sequential_mode: &bool,
) -> Result<tokio::process::Child> {
    prepare_plugins(env_name, &original_cf_home, &mcf_folder)?;
    let mut tokio_command = cf_command_tokio(&options.cf_binary_name, env_name, &mcf_folder);
    tokio_command.args(command.to_vec());
    if !sequential_mode {
        tokio_command.stdout(Stdio::piped());
    }
    let result = tokio_command.spawn().context("Could not spawn")?;
    Ok(result)
}

pub fn cf_command_tokio(cf_binary_name: &String, name: &String, mcf_folder: &Path) -> Command {
    let mut cf: Command = Command::new(cf_binary_name);
    let cf_home: PathBuf = get_cf_home_from_mcf_environment(name, mcf_folder);
    cf.env("CF_HOME", cf_home);
    cf
}

pub fn check_if_cf_is_installed(cf_binary_name: &String) -> Result<bool> {
    check_if_installed(
        cf_binary_name,
        None,
        vec![
            String::from("cf version"),
            String::from("Cloud Foundry command line tool"),
        ],
    )
}

fn check_if_installed(
    cf_binary_name: &String,
    args: Option<Vec<String>>,
    output_should_contain: Vec<String>,
) -> Result<bool> {
    let mut command = std::process::Command::new(cf_binary_name);
    if let Some(args) = args {
        command.args(args);
    }
    let output = String::from_utf8(
        command
            .stdout(Stdio::piped())
            .spawn()
            .context("mcf: could not spawn to check if cf is installed")?
            .wait_with_output()
            .context("mcf: problem is getting the output to check if cf is installed")?
            .stdout,
    )?;
    Ok(output_should_contain
        .iter()
        .all(|text| output.contains(text)))
}

fn get_cf_home_from_mcf_environment(env_name: &String, mcf_folder: &Path) -> PathBuf {
    let mut cf_home = mcf_folder.to_path_buf();
    cf_home.push("homes");
    cf_home.push(env_name);
    cf_home
}

fn prepare_plugins(name: &String, original_cf_home: &Path, mcf_folder: &Path) -> Result<()> {
    let source = original_cf_home.join("plugins");
    if !source.exists() {
        bail!("source does not exist, source={:#?}", source);
    }
    let cf_dir = get_cf_home_from_mcf_environment(name, mcf_folder).join(".cf");
    let destination = cf_dir.join("plugins");
    if let Ok(metadata) = std::fs::symlink_metadata(&destination) {
        if metadata.is_dir() {
            std::fs::remove_dir(&destination)?;
            create_symlink(source, destination)?;
        } else if metadata.is_file() {
            std::fs::remove_file(&destination)?;
            create_symlink(source, destination)?;
        }
    } else {
        std::fs::create_dir_all(&cf_dir)?;
        create_symlink(source, destination)?;
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn create_symlink<P: AsRef<Path>, Q: AsRef<Path>>(source: P, destination: Q) -> Result<()> {
    std::os::windows::fs::symlink_dir(source, destination).context("Symlink creation failed")
}

#[cfg(not(target_os = "windows"))]
fn create_symlink<P: AsRef<Path>, Q: AsRef<Path>>(source: P, destination: Q) -> Result<()> {
    std::os::unix::fs::symlink(source, destination).context("Symlink creation failed")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::Environment;
    use std::ffi::OsStr;
    use tempfile::tempdir;

    #[test]
    fn test_cf_command() {
        let tempdir: PathBuf = tempdir().unwrap().into_path();
        let result = cf_command_tokio(
            &String::from("echo"),
            &String::from("envname"),
            &tempdir.join("mcf-lib-test"),
        );
        assert_eq!(result.as_std().get_program().to_str().unwrap(), "echo");
        assert_eq!(
            result
                .as_std()
                .get_envs()
                .map(|(key, _)| key)
                .collect::<Vec<&OsStr>>(),
            vec![OsStr::new("CF_HOME")]
        );
        assert_eq!(
            result
                .as_std()
                .get_envs()
                .map(|(_, value)| value)
                .filter(|value| value.is_some())
                .filter(|value| value.unwrap().to_str().unwrap()
                    == tempdir
                        .join("mcf-lib-test")
                        .join("homes")
                        .join("envname")
                        .to_str()
                        .unwrap())
                .collect::<Vec<Option<&OsStr>>>()
                .len(),
            1
        );
    }

    #[test]
    fn test_get_mcf_home() {
        let tempdir: PathBuf = tempdir().unwrap().into_path();
        let result: PathBuf = get_cf_home_from_mcf_environment(
            &String::from("envname"),
            &tempdir.join("mcf-lib-test"),
        );
        let expected: PathBuf = [
            &tempdir.join("mcf-lib-test").to_str().unwrap(),
            "homes",
            &String::from("envname"),
        ]
        .iter()
        .collect::<PathBuf>();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_prepare_plugins_if_happy_case() {
        let tempdir: PathBuf = tempdir().unwrap().into_path();
        let source = &tempdir.join(".cf").join("plugins");
        let _ = std::fs::create_dir_all(source);
        let _ = std::fs::File::create(source.join("test-file"));
        let result = prepare_plugins(
            &String::from("envname"),
            &tempdir.join(".cf"),
            &tempdir.join("mcf-lib-home"),
        );
        assert!(result.is_ok());
        assert!(tempdir
            .join("mcf-lib-home")
            .join("homes")
            .join("envname")
            .join(".cf")
            .join("plugins")
            .is_symlink());
        assert_eq!(
            std::fs::read_link(
                tempdir
                    .join("mcf-lib-home")
                    .join("homes")
                    .join("envname")
                    .join(".cf")
                    .join("plugins")
            )
            .unwrap(),
            tempdir.join(".cf").join("plugins")
        );
        assert!(
            std::fs::read_dir(
                tempdir
                    .join("mcf-lib-home")
                    .join("homes")
                    .join("envname")
                    .join(".cf")
                    .join("plugins")
            )
            .unwrap()
            .into_iter()
            .map(|path| String::from(path.unwrap().file_name().to_str().unwrap()))
            .filter(|file| file == &"test-file")
            .count()
                == 1
        );
    }

    #[test]
    fn test_prepare_plugins_if_source_does_not_exist() {
        let tempdir: PathBuf = tempdir().unwrap().into_path();
        let result = prepare_plugins(
            &String::from("envname"),
            &tempdir.join(".cf"),
            &tempdir.join("mcf-lib-home"),
        );
        assert!(result.is_err());
        assert!(result
            .err()
            .unwrap()
            .to_string()
            .contains(&"source does not exist, source="));
    }

    #[test]
    fn test_prepare_plugins_if_folder_exists() {
        let tempdir: PathBuf = tempdir().unwrap().into_path();
        let source = &tempdir.join(".cf").join("plugins");
        let _ = std::fs::create_dir_all(source);
        let _ = std::fs::create_dir_all(
            get_cf_home_from_mcf_environment(
                &String::from("envname"),
                &tempdir.join("mcf-lib-home"),
            )
            .join(".cf")
            .join("plugins"),
        );
        let result = prepare_plugins(
            &String::from("envname"),
            &tempdir.join(".cf"),
            &tempdir.join("mcf-lib-home"),
        );
        assert!(result.is_ok());
        assert!(tempdir
            .join("mcf-lib-home")
            .join("homes")
            .join("envname")
            .join(".cf")
            .join("plugins")
            .is_symlink());
    }

    #[test]
    fn test_prepare_plugins_if_file_exists() {
        let tempdir: PathBuf = tempdir().unwrap().into_path();
        let source = &tempdir.join(".cf").join("plugins");
        let _ = std::fs::create_dir_all(source);
        let folder = get_cf_home_from_mcf_environment(
            &String::from("envname"),
            &tempdir.join("mcf-lib-home"),
        )
        .join(".cf");
        let _ = std::fs::create_dir_all(&folder);
        let _ = std::fs::File::create(&folder.join("plugins"));
        let result = prepare_plugins(
            &String::from("envname"),
            &tempdir.join(".cf"),
            &tempdir.join("mcf-lib-home"),
        );
        assert!(result.is_ok());
        assert!(tempdir
            .join("mcf-lib-home")
            .join("homes")
            .join("envname")
            .join(".cf")
            .join("plugins")
            .is_symlink());
    }

    #[tokio::test]
    async fn test_login_could_not_find_environment_in_list() {
        let tempdir: PathBuf = tempdir().unwrap().into_path();
        let result = login(
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
            &String::from("p02"),
            &PathBuf::from(""),
            &None,
            &None,
            &None,
        )
        .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "could not find \"p02\" in environment list [\n    Environment {\n        name: \"p01\",\n        url: \"url\",\n        sso: false,\n        skip_ssl_validation: false,\n    },\n]");
    }

    #[tokio::test]
    async fn test_login_happy_case() {
        let tempdir: PathBuf = tempdir().unwrap().into_path();
        let result = login(
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
            &PathBuf::from(""),
            &None,
            &None,
            &None,
        )
        .await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_if_installed() {
        let output_one = check_if_installed(
            &String::from("echo"),
            Some(vec![String::from("hello")]),
            vec![
                String::from("cf version"),
                String::from("Cloud Foundry command line tool"),
            ],
        );
        let output_two = check_if_installed(
            &String::from("echo"),
            Some(vec![
                String::from("cf"),
                String::from("version"),
                String::from("8.6.1+b5a352a.2023-02-27,"),
                String::from("Cloud"),
                String::from("Foundry"),
                String::from("command"),
                String::from("line"),
                String::from("tool"),
            ]),
            vec![
                String::from("cf version"),
                String::from("Cloud Foundry command line tool"),
            ],
        );
        assert!(output_one.is_ok());
        assert_eq!(output_one.unwrap(), false);
        assert!(output_two.is_ok());
        assert_eq!(output_two.unwrap(), true);
    }
}
