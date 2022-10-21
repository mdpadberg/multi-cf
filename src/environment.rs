use crate::settings::Settings;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tabled::{Table, Tabled};

#[derive(Debug, Deserialize, Serialize, Clone, Tabled, PartialEq, Eq)]
pub struct Environment {
    pub name: String,
    pub url: String,
    pub sso: bool,
    pub skip_ssl_validation: bool,
}

#[derive(clap::Subcommand, Debug)]
pub enum EnvironmentCommands {
    /// Add an environment to the environment list
    Add {
        name: String,
        url: String,
        #[clap(long)]
        sso: bool,
        #[clap(long)]
        skip_ssl_validation: bool,
    },
    #[clap(visible_alias = "rm")]
    /// Remove an environment to the environment list
    Remove { name: String },
    /// List all the environment you stored
    List,
}

pub fn match_environment(
    settings: &Settings,
    override_path: Option<&PathBuf>,
    environment_commands: &EnvironmentCommands,
) -> Result<()> {
    match environment_commands {
        EnvironmentCommands::Add {
            name,
            url,
            sso,
            skip_ssl_validation,
        } => add(
            &settings,
            override_path,
            name,
            url,
            sso,
            skip_ssl_validation,
        ),
        EnvironmentCommands::Remove { name } => remove(&settings, override_path, name),
        EnvironmentCommands::List => list(&settings),
    }
}

pub fn add(
    settings: &Settings,
    override_path: Option<&PathBuf>,
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
    new_settings.save(override_path)
}

pub fn remove(settings: &Settings, override_path: Option<&PathBuf>, name: &String) -> Result<()> {
    let mut environments = settings.environments.clone();
    environments.retain(|env| &env.name != name);
    let new_settings = Settings {
        environments: environments,
    };
    new_settings.save(override_path)
}

pub fn list(settings: &Settings) -> Result<()> {
    let environments = Table::new(settings.environments.clone())
        .with(tabled::Style::markdown())
        .to_string();
    print!("{}", environments);
    Ok(())
}