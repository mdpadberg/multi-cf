use anyhow::Result;
use lib::{
    environment::{add, list, remove, Environment},
    options::Options,
    settings::Settings,
};
use prettytable::{Cell, Row, Table};

#[derive(clap::Subcommand, Debug)]
pub enum EnvironmentCommands {
    /// Add an environment to the environment list
    Add {
        name: String,
        url: String,
        #[arg(long)]
        sso: bool,
        #[arg(long)]
        skip_ssl_validation: bool,
    },
    /// Remove an environment to the environment list
    #[command(visible_alias = "rm")]
    Remove { name: String },
    /// List all the environment you stored
    #[command(visible_alias = "ls")]
    List,
}

pub fn match_environment(
    settings: &Settings,
    options: &Options,
    environment_commands: &EnvironmentCommands,
) -> Result<()> {
    match environment_commands {
        EnvironmentCommands::Add {
            name,
            url,
            sso,
            skip_ssl_validation,
        } => add(&settings, options, name, url, sso, skip_ssl_validation),
        EnvironmentCommands::Remove { name } => remove(&settings, options, name),
        EnvironmentCommands::List => {
            let all_envs = list(&settings);
            let mut table = Table::new();
            //HEADER
            table.add_row(Row::new(
                Environment::get_fields()?
                    .iter()
                    .map(|field| Cell::new(field))
                    .collect(),
            ));
            //CONTENT
            for env in all_envs {
                let all_values = env.get_values()?;
                table.add_row(Row::new(
                    all_values.iter().map(|field| Cell::new(field)).collect(),
                ));
            }
            table.printstd();
            Ok(())
        }
    }
}
