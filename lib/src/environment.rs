use crate::options::Options;
use crate::settings::Settings;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Environment {
    pub name: String,
    pub url: String,
    pub sso: bool,
    pub skip_ssl_validation: bool,
}

pub fn add(
    settings: &Settings,
    options: &Options,
    name: &String,
    url: &String,
    sso: &bool,
    skip_ssl_validation: &bool,
) -> Result<()> {
    let mut environments = settings.environments.clone();
    environments.retain(|env| &env.name != name);
    environments.push(Environment {
        name: name.clone(),
        url: url.clone(),
        sso: *sso,
        skip_ssl_validation: *skip_ssl_validation,
    });
    let new_settings = Settings {
        environments: environments,
    };
    new_settings.save(options)
}

pub fn remove(settings: &Settings, options: &Options, name: &String) -> Result<()> {
    let mut environments = settings.environments.clone();
    environments.retain(|env| &env.name != name);
    let new_settings = Settings {
        environments: environments,
    };
    new_settings.save(options)
}

pub fn list(settings: &Settings) -> Vec<Environment> {
    settings.environments.clone()
}

//TODO: write tests