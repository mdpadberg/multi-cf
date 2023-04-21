use std::path::Path;
use std::path::PathBuf;
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

pub fn login(
    settings: &Settings,
    options: &Options,
    name: &String,
    mcf_home: &PathBuf,
) -> Result<()> {
    if let Some(some) = settings.environments.iter().find(|env| &env.name == name) {
        let cf_binary_name = &options.cf_binary_name;
        let mut cf: Command = cf_command_tokio(cf_binary_name, &some.name, mcf_home);
        cf.arg("login").arg("-a").arg(&some.url);
        if some.skip_ssl_validation {
            cf.arg("--skip-ssl-validation");
        }
        if some.sso {
            cf.arg("--sso");
        }
        let mut child = cf.spawn().expect("Failure in creating child process");
        let _ = child.wait();
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
) -> Result<tokio::process::Child> {
    prepare_plugins(env_name, &original_cf_home, &mcf_folder)?;
    Ok(cf_command_tokio(&options.cf_binary_name, env_name, &mcf_folder)
        .args(command.to_vec())
        .spawn()
        .context("Could not spawn")?)
}

pub fn cf_command_tokio(cf_binary_name: &String, name: &String, mcf_folder: &PathBuf) -> Command {
    let mut cf: Command = Command::new(cf_binary_name);
    let cf_home: PathBuf = get_cf_home_from_mcf_environment(name, mcf_folder);
    cf.env("CF_HOME", cf_home);
    cf
}

fn get_cf_home_from_mcf_environment(env_name: &String, mcf_folder: &PathBuf) -> PathBuf {
    let mut cf_home = mcf_folder.clone();
    cf_home.push("homes");
    cf_home.push(env_name);
    return cf_home;
}

fn prepare_plugins(name: &String, original_cf_home: &PathBuf, mcf_folder: &PathBuf) -> Result<()> {
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
    use std::ffi::OsStr;
    use tempfile::tempdir;
    use crate::environment::Environment;
    use super::*;

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

    #[test]
    fn test_login_could_not_find_environment_in_list() {
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
        );
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
        );
        assert!(result.is_ok());
    }
}
