use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::environment::Environment;

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub environments: Vec<Environment>,
}

fn path_to_settings_file() -> Option<PathBuf> {
    dirs::config_dir().map(|f|f.join("mcf/settings.yml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_settings() {
        let actual_path = path_to_settings_file().unwrap();
        let expected_path = dirs::config_dir().unwrap().join("mcf/settings.yml");
        assert_eq!(actual_path, expected_path);
    }
}