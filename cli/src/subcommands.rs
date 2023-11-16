use crate::environment::EnvironmentCommands;
use clap::Subcommand;
use clap_complete::Shell;

#[derive(Subcommand, Debug)]
pub enum Subcommands {
    /// Add, Remove, List environment (example cf-dev)
    #[command(visible_alias = "env")]
    Environment {
        #[command(subcommand)]
        environment_commands: EnvironmentCommands,
    },
    /// Login to one of the Cloud Foundry environments
    #[command(visible_alias = "l")]
    Login {
        /// Name of the environment (example "cf-dev")
        name: String,
        /// One-time passcode
        #[arg(long)]
        sso_passcode: Option<String>,
         /// Cloudfoundry organization
        #[arg(short,long)]
        org: Option<String>,
         /// Cloudfoundry space
        #[arg(short,long)]
        space: Option<String>,
    },
    /// Execute command on Cloud Foundry environment
    #[command(visible_alias = "e", trailing_var_arg = true)]
    Exec {
        /// Names of the environments (example "cf-dev,cf-prod")
        names: String,
        /// Command you want to execute (example "logs your-application --recent")
        command: Vec<String>,
        /// Execute command sequentially (example "ssh your-application")
        #[arg(short, long)]
        sequential_mode: bool
    },
    /// Generate shell autocompletion files
    Completion {
        #[arg(value_enum)]
        shell: Shell,
    },
}
