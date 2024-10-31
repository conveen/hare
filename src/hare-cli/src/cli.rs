use tracing::debug;

/// Subcommand groups.
#[derive(Debug, clap::Subcommand)]
pub(crate) enum SubcommandGroups {
    /// Control plane commands.
    #[command(alias = "cp")]
    ControlPlane {
        /// Server endpoint URL.
        #[arg(from_global)]
        endpoint: String,
        #[command(subcommand)]
        command: crate::control_plane::ControlPlaneCommands,
    },
}

impl SubcommandGroups {
    pub async fn run(self) {
        match self {
            SubcommandGroups::ControlPlane { command, endpoint } => command.run(endpoint).await,
        }
    }
}

/// Command line parser.
#[derive(Debug, clap::Parser)]
pub(crate) struct Cli {
    /// Server endpoint URL.
    #[arg(long, default_value = "http://127.0.0.1:5001", global = true)]
    endpoint: String,
    #[command(subcommand)]
    command: SubcommandGroups,
}

impl Cli {
    pub async fn run(self) {
        crate::logging::configure_logging();
        debug!(?self, "Command line args");
        self.command.run().await;
    }
}
