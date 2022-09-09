use crate::environment::Environment;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};
type SystemNotSupported;

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub environments: Vec<Environment>,
}

fn path_to_settings_file() -> Result<PathBuf> {
    dirs::config_dir()
        .map(|f| f.join("mcf/settings.yml"))
        .ok_or(anyhow!("Plaform unsupported by dirs crate"))
}

fn write_settings_file_to_disk(path: &PathBuf, settings: &Settings) -> Result<()> {
    fs::create_dir_all(
        &path
            .parent()
            .ok_or(anyhow!("Dirs crate didn't provide an parent folder"))?,
    )?;
    File::create(&path)?.write_all(serde_yaml::to_string(settings)?.as_bytes())?;
    Ok(())
}

fn read_settings_file_from_disk(path: &PathBuf) -> Result<Settings> {
    let settings_file_as_string = fs::read_to_string(path)?;
    let settings_file: Settings = serde_yaml::from_str(settings_file_as_string.as_str())?;
    Ok(settings_file)
}

impl Settings {
    pub fn load() -> Result<Self,SystemNotSupported> {
 
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn test_path_to_settings_file() {
        let actual_path = path_to_settings_file().unwrap();
        let expected_path = dirs::config_dir().unwrap().join("mcf/settings.yml");
        assert_eq!(actual_path, expected_path);
    }

    #[test]
    fn test_write_empty_settings_file_to_disk() {
        let _ = write_settings_file_to_disk(
            &env::temp_dir().join("mcf"),
            &Settings {
                environments: Vec::new(),
            },
        );
        assert_eq!(
            fs::read_to_string(env::temp_dir().join("mcf")).unwrap(),
            String::from("environments: []\n")
        )
    }
}
