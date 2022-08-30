use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Environment {
    pub name: String,
    pub url: String,
    pub sso: bool,
    pub skip_ssl_validation: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    environments: Vec<Environment>,
}

fn path_to_settings() -> Option<PathBuf> {
    let mut path = dirs::config_dir()?;
    path.push("mcf");
    path.push("settings.yml");
    Some(path)
}

fn create_empty_settings_file(path: &Path) -> Result<()> {
    let parent = match path.parent() {
        Some(some) => some,
        None => bail!(""),
    };
    fs::create_dir_all(parent)?;
    let mut file = File::create(path)?;
    let empty_settings = serde_yaml::to_string(&Settings {
        environments: Vec::new(),
    })?;
    file.write_all(empty_settings.as_bytes())?;
    Ok(())
}

fn read_settings_file(path: &Path) -> Result<Settings> {
    let settings_file_as_string = fs::read_to_string(path)?;
    let settings_file: Settings = serde_yaml::from_str(settings_file_as_string.as_str())?;
    Ok(settings_file)
}

fn write_settings_file(path_buf: &PathBuf, settings: &Settings) -> Result<()> {
    let mut file = File::create(path_buf)?;
    let settings_file_as_string = serde_yaml::to_string(&settings)?;
    file.write_all(settings_file_as_string.as_bytes())?;
    Ok(())
}

impl Settings {
    pub fn new() -> Option<Self> {
        let path_to_settings = path_to_settings()?;
        if !path_to_settings.exists() {
            create_empty_settings_file(&path_to_settings).expect("aaaa")
        };
        read_settings_file(&path_to_settings).ok()
    }

    pub fn get_environments(&self) -> Vec<Environment> {
        self.environments.to_vec()
    }

    pub fn add(&mut self, environment: Environment) {
        self.environments.retain(|env| env.name != environment.name);
        self.environments.push(environment);
    }

    pub fn save(&self) -> bool {
        if let Some(some) = path_to_settings() {
            write_settings_file(&some, self).expect("Could not save file");
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, name: &String) {
        self.environments.retain(|env| env.name != *name);
    }

    pub fn get_by_environment_by_name(&self, name: &String) -> Option<Environment> {
        self.environments
            .iter()
            .find(|env| env.name == *name)
            .cloned()
    }
}
