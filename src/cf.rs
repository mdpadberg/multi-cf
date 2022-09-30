use dirs::data_dir;
use std::{
    path::PathBuf,
    process::{self, Command},
};

pub fn command(override_path: Option<&PathBuf>, name: &String) -> Command {
    let mut cf: Command = process::Command::new("cf");
    let mut cf_home: PathBuf = if override_path.is_some() {
        override_path.unwrap().clone()
    } else {
        data_dir().expect("no data dir")
    };
    cf_home.push("mcf");
    cf_home.push("homes");
    cf_home.push(name);
    cf.env("CF_HOME", cf_home);
    cf
}
