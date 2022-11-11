use anyhow::{Context, Result};
use dirs::data_dir;
use std::os::unix::fs;
use std::{
    path::PathBuf,
    process::{self, Command, Stdio},
};

pub fn cf_command(cf_binary_name: &String, name: &String) -> Command {
    let mut cf: Command = Command::new(cf_binary_name);
    let cf_home: PathBuf = get_cf_home(name);
    cf.env("CF_HOME", cf_home);
    cf
}

fn get_cf_home(name: &String) -> PathBuf {
    let mut cf_home: PathBuf = data_dir().expect("no data dir");
    cf_home.push("mcf");
    cf_home.push("homes");
    cf_home.push(name);

    return cf_home;
}

fn prepare_plugins(name: &String) -> Result<()> {
    let source = &mut dirs::home_dir().context("No home dir")?;
    source.push(".cf/plugins");
    let mut destination = get_cf_home(name);
    destination.push(".cf/plugins");

    if let Ok(metadata) = std::fs::symlink_metadata(&destination) {
        if metadata.is_dir() {
            std::fs::remove_dir(&destination)?;
            fs::symlink(source, destination).context("Symlink creation failed")?;
        } else if metadata.is_file() {
            std::fs::remove_file(&destination)?;
            fs::symlink(source, destination).context("Symlink creation failed")?;
        }
    } else {
        fs::symlink(source, destination).context("Symlink creation failed")?;
    }

    Ok(())
}

pub fn exec(
    cf_binary_name: &String,
    env_name: &String,
    command: &Vec<String>,
) -> Result<process::ChildStdout> {
    prepare_plugins(&env_name)?;
    let child = cf_command(cf_binary_name, env_name)
        .args(command)
        .stdout(Stdio::piped())
        .spawn()
        .context("Could not spawn")?;
    child.stdout.context("Could get stdout")
}
