use std::{path::PathBuf, process::Command};

use crate::{cf, settings::Settings};
use anyhow::{bail, Result};

pub fn to_cf(settings: &Settings, override_path: Option<&PathBuf>, name: &String) -> Result<()> {
    if let Some(some) = settings.environments.iter().find(|env| &env.name == name) {
        let mut cf: Command = cf::cf_command(override_path, &some.name);
        cf.arg("login").arg("-a").arg(&some.url);
        if some.skip_ssl_validation {
            cf.arg("--skip-ssl-validation");
        }
        if some.sso {
            cf.arg("--sso");
        }
        let mut child = cf.spawn().expect("Failure in creating child process");
        let _ = child.wait();
    } else {
        bail!(
            "could not find {:#?} in environment list {:#?}",
            name,
            settings.environments
        );
    }

    Ok(())
}
