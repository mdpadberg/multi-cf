use crate::environment::Environment;
use crate::options::Options;
use anyhow::{anyhow, Result};
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
    pub fn load(options: &Options) -> Result<Self> {
        let override_path = options.get_mcf_home_path_buf();
        let settings_path = path_to_settings_file(override_path);
        Ok(read_settings_file_from_disk(&settings_path).unwrap_or_default())
    }

    pub fn save(&self, options: &Options) -> Result<()> {
        let override_path = options.get_mcf_home_path_buf();
        let settings_path = path_to_settings_file(override_path);
        write_settings_file_to_disk(&settings_path, self)?;
        Ok(())
    }

    pub fn get_environment_by_name(&self, name: &String) -> Option<Environment> {
        self.environments
            .iter()
            .find(|env| env.name == *name)
            .cloned()
    }
}

fn path_to_settings_file(override_path: PathBuf) -> PathBuf {
    let filename = "settings.yml";
    override_path.join(filename)
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
    use tempfile::tempdir;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_path_to_settings_file() {
        init();
        let opts = Options::new(None, Some("/test/mcf/".to_string()));
        let path = opts.get_mcf_home_path_buf();
        let actual_path = path_to_settings_file(path);
        let expected_path = PathBuf::from("/test/mcf/settings.yml");
        assert_eq!(actual_path, expected_path);
    }

    #[test]
    fn test_write_empty_settings_file_to_disk() {
        let tempdir: PathBuf = tempdir().unwrap().into_path();
        init();
        let _ = write_settings_file_to_disk(
            &path_to_settings_file(tempdir.clone()),
            &Settings {
                environments: Vec::new(),
            },
        );
        assert_eq!(
            fs::read_to_string(&path_to_settings_file(tempdir)).unwrap(),
            String::from("environments: []\n")
        );
    }

    #[test]
    fn load_will_return_empty_settings_file_when_there_is_no_file_on_disk() {
        init();
        let tempdir = tempdir().unwrap().into_path().to_str().unwrap().to_string();

        let options = Options::new(None, Some(tempdir));
        assert_eq!(Settings::load(&options).unwrap(), Settings::default());
    }

    #[test]
    fn load_will_return_the_settings_file_when_there_is_a_file_on_disk() {
        let tempdir: PathBuf = tempdir().unwrap().into_path();
        init();
        let expected = Environment {
            name: "name".to_string(),
            url: "url".to_string(),
            sso: false,
            skip_ssl_validation: false,
        };
        let _ = write_settings_file_to_disk(
            &path_to_settings_file(tempdir.clone()),
            &Settings {
                environments: vec![expected.clone()],
            },
        );
        assert_eq!(
            Settings::load(&Options {
                cf_binary_name: "cf".to_string(),
                mcf_home: tempdir
                    .to_str()
                    .unwrap()
                    .to_string()
            })
            .unwrap()
            .environments[0],
            expected
        );
    }

    #[test]
    fn save_will_write_file_to_disk() {
        init();
        let option = Options {
            cf_binary_name: "cf".to_string(),
            mcf_home: tempdir().unwrap().into_path()
                .to_str()
                .unwrap()
                .to_string(),
        };
        let expected = Settings {
            environments: vec![Environment {
                name: "name".to_string(),
                url: "url".to_string(),
                sso: false,
                skip_ssl_validation: false,
            }],
        };

        let result = expected.save(&option);
        assert!(result.is_ok());
        assert_eq!(Settings::load(&option).unwrap(), expected);
    }
}
