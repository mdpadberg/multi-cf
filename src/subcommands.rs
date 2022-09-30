use clap_complete::Shell;
use crate::environment::EnvironmentCommands;

#[derive(clap::Subcommand, Debug)]
pub enum Subcommands {
    /// Add, Remove, List environment (example cf-dev)
    #[clap(visible_alias = "env")]
    Environment {
        #[clap(subcommand)]
        environment_commands: EnvironmentCommands,
    },
    /// Login to one of the Cloud Foundry environments
    #[clap(visible_alias = "l")]
    Login {
        /// Name of the environment (example "cf-dev")
        name: String,
    },
    /// Execute command on Cloud Foundry environment
    #[clap(visible_alias = "e", trailing_var_arg = true)]
    Exec {
        /// Names of the environments (example "cf-dev,cf-prod")
        names: String,
        /// Command you want to execute (example "logs your-application --recent")
        command: Vec<String>,
    },
    /// Generate shell autocompletion files
    Completion {
        #[clap(arg_enum, value_parser)]
        shell: Shell,
    },
}