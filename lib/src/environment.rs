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

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_add() {
        let env_one = Environment {
            name: "one".to_string(),
            url: "url_one".to_string(),
            sso: false,
            skip_ssl_validation: false,
        };
        let env_two = Environment {
            name: "two".to_string(),
            url: "url_two".to_string(),
            sso: true,
            skip_ssl_validation: true,
        };
        let settings = Settings {
            environments: vec![env_one.clone()],
        };
        let tempdir: PathBuf = tempdir().unwrap().into_path();
        let source = &tempdir.join("environment-test-add");
        let _ = std::fs::create_dir_all(source);
        let result = add(
            &settings,
            &Options {
                cf_binary_name: String::from("cf"),
                mcf_home: source.to_str().unwrap().to_string(),
            },
            &env_two.name,
            &env_two.url,
            &env_two.sso,
            &env_two.skip_ssl_validation,
        );
        assert!(result.is_ok());
        assert_eq!(
            fs::read_to_string(source.join("settings.yml")).unwrap(), 
            String::from("environments:\n- name: one\n  url: url_one\n  sso: false\n  skip_ssl_validation: false\n- name: two\n  url: url_two\n  sso: true\n  skip_ssl_validation: true\n")
        );
    }

    #[test]
    fn test_remove() {
        let env_one = Environment {
            name: "one".to_string(),
            url: "url_one".to_string(),
            sso: false,
            skip_ssl_validation: false,
        };
        let env_two = Environment {
            name: "two".to_string(),
            url: "url_two".to_string(),
            sso: true,
            skip_ssl_validation: true,
        };
        let settings = Settings {
            environments: vec![env_one.clone(), env_two.clone()],
        };
        let tempdir: PathBuf = tempdir().unwrap().into_path();
        let source = &tempdir.join("environment-test-add");
        let _ = std::fs::create_dir_all(source);
        let result = remove(
            &settings,
            &Options {
                cf_binary_name: String::from("cf"),
                mcf_home: source.to_str().unwrap().to_string(),
            },
            &String::from("one")
        );
        assert!(result.is_ok());
        assert_eq!(
            fs::read_to_string(source.join("settings.yml")).unwrap(), 
            String::from("environments:\n- name: two\n  url: url_two\n  sso: true\n  skip_ssl_validation: true\n")
        );
    }

    #[test]
    fn test_list() {
        let env_one = Environment {
            name: "one".to_string(),
            url: "url_one".to_string(),
            sso: false,
            skip_ssl_validation: false,
        };
        let env_two = Environment {
            name: "two".to_string(),
            url: "url_two".to_string(),
            sso: true,
            skip_ssl_validation: true,
        };
        let settings = Settings {
            environments: vec![env_one.clone(), env_two.clone()],
        };
        assert_eq!(list(&settings), vec![env_one, env_two]);
    }
}
