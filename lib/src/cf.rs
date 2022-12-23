use anyhow::{bail, Context, Result};
use std::os::unix::fs;
use std::path::Path;
use std::{
    path::PathBuf,
    process::{self, Command, Stdio},
};

pub fn stdout(
    cf_binary_name: &String,
    command: &Vec<String>,
    env_name: &String,
    original_cf_home: &PathBuf,
    mcf_folder: &PathBuf,
) -> Result<process::ChildStdout> {
    prepare_plugins(env_name, original_cf_home, mcf_folder)?;
    Ok(cf_command(cf_binary_name, env_name, mcf_folder)
        .args(command)
        .stdout(Stdio::piped())
        .spawn()
        .context("Could not spawn")?
        .stdout
        .context("Could get stdout")?)
}

pub fn cf_command(cf_binary_name: &String, name: &String, mcf_folder: &PathBuf) -> Command {
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
    let source = original_cf_home.join(".cf/plugins");
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
    fs::symlink(source, destination).context("Symlink creation failed")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;
    use tempfile::tempdir;

    #[test]
    fn test_cf_command() {
        let tempdir: PathBuf = tempdir().unwrap().into_path();
        let result = cf_command(
            &String::from("echo"),
            &String::from("envname"),
            &tempdir.join("mcf-lib-test"),
        );
        assert_eq!(result.get_program().to_str().unwrap(), "echo");
        assert_eq!(
            result
                .get_envs()
                .map(|(key, value)| key)
                .collect::<Vec<&OsStr>>(),
            vec![OsStr::new("CF_HOME")]
        );
        assert_eq!(
            result
                .get_envs()
                .map(|(key, value)| value)
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
        let result: PathBuf = get_cf_home_from_mcf_environment(&String::from("envname"), &tempdir.join("mcf-lib-test"));
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
        let source = &tempdir.join("mcf-lib-source").join(".cf").join("plugins");
        let _ = std::fs::create_dir_all(source);
        let _ = std::fs::File::create(source.join("test-file"));
        let result = prepare_plugins(
            &String::from("envname"),
            &tempdir.join("mcf-lib-source"),
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
            tempdir.join("mcf-lib-source").join(".cf").join("plugins")
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
            &tempdir.join("mcf-lib-source"),
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
        let source = &tempdir.join("mcf-lib-source").join(".cf").join("plugins");
        let _ = std::fs::create_dir_all(source);
        let _ = std::fs::create_dir_all(
            get_cf_home_from_mcf_environment(&String::from("envname"), &tempdir.join("mcf-lib-home"))
                .join(".cf")
                .join("plugins"),
        );
        let result = prepare_plugins(
            &String::from("envname"),
            &tempdir.join("mcf-lib-source"),
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
        let source = &tempdir.join("mcf-lib-source").join(".cf").join("plugins");
        let _ = std::fs::create_dir_all(source);
        let folder =
            get_cf_home_from_mcf_environment(&String::from("envname"), &tempdir.join("mcf-lib-home")).join(".cf");
        let _ = std::fs::create_dir_all(&folder);
        let _ = std::fs::File::create(&folder.join("plugins"));
        let result = prepare_plugins(
            &String::from("envname"),
            &tempdir.join("mcf-lib-source"),
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
}
