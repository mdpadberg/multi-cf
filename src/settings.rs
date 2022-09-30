use crate::environment::Environment;
use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Settings {
    pub environments: Vec<Environment>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            environments: vec![],
        }
    }
}

impl Settings {
    pub fn load(override_path: Option<&PathBuf>) -> Result<Self> {
        if let Ok(ok) = path_to_settings_file(override_path) {
            Ok(read_settings_file_from_disk(&ok).unwrap_or_default())
        } else {
            error!("system not support, os={}", std::env::consts::OS);
            bail!("system not supported")
        }
    }

    pub fn save(&self, override_path: Option<&PathBuf>) -> Result<()> {
        if let Ok(ok) = path_to_settings_file(override_path) {
            write_settings_file_to_disk(&ok, self)?;
            Ok(())
        } else {
            error!("system not support, os={}", std::env::consts::OS);
            bail!("system not supported")
        }
    }
}

fn path_to_settings_file(override_path: Option<&PathBuf>) -> Result<PathBuf> {
    let filename = "settings.yml";
    if let Some(override_path) = override_path {
        Ok(override_path.join(filename))
    } else {
        dirs::config_dir()
            .map(|f| f.join("mcf/").join(filename))
            .ok_or(anyhow!("Plaform unsupported by dirs crate"))
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_path_to_settings_file() {
        init();
        let actual_path = path_to_settings_file(None).unwrap();
        let expected_path = dirs::config_dir().unwrap().join("mcf/settings.yml");
        assert_eq!(actual_path, expected_path);
    }

    #[test]
    fn test_write_empty_settings_file_to_disk() {
        init();
        let _ = write_settings_file_to_disk(
            &path_to_settings_file(Some(&std::env::temp_dir().join("mcf-1"))).unwrap(),
            &Settings {
                environments: Vec::new(),
            },
        );
        assert_eq!(
            fs::read_to_string(
                &path_to_settings_file(Some(&std::env::temp_dir().join("mcf-1"))).unwrap()
            )
            .unwrap(),
            String::from("environments: []\n")
        );
    }

    #[test]
    fn load_will_return_empty_settings_file_when_there_is_no_file_on_disk() {
        init();
        assert_eq!(
            Settings::load(Some(&std::env::temp_dir())).unwrap(),
            Settings::default()
        );
    }

    #[test]
    fn load_will_return_the_settings_file_when_there_is_a_file_on_disk() {
        init();
        let expected = Environment {
            name: "name".to_string(),
            url: "url".to_string(),
            sso: false,
            skip_ssl_validation: false,
        };
        let _ = write_settings_file_to_disk(
            &path_to_settings_file(Some(&std::env::temp_dir().join("mcf-2"))).unwrap(),
            &Settings {
                environments: vec![expected.clone()],
            },
        );
        assert_eq!(
            Settings::load(Some(&std::env::temp_dir().join("mcf-2")))
                .unwrap()
                .environments[0],
            expected
        );
    }

    #[test]
    fn save_will_write_file_to_disk() {
        init();
        let expected = Settings {
            environments: vec![Environment {
                name: "name".to_string(),
                url: "url".to_string(),
                sso: false,
                skip_ssl_validation: false,
            }],
        };
        let result = expected.save(Some(&std::env::temp_dir().join("mcf-3")));
        assert!(result.is_ok());
        assert_eq!(
            Settings::load(Some(&std::env::temp_dir().join("mcf-3"))).unwrap(),
            expected
        );
    }
}
