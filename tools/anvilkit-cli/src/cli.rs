use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "anvil",
    about = "AnvilKit game engine CLI",
    version,
    propagate_version = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create a new game project
    New {
        /// Project name (snake_case)
        name: String,
        /// Template to use: 3d-basic, topdown, first-person, empty
        #[arg(short, long)]
        template: Option<String>,
    },

    /// Generate ECS boilerplate code
    Generate {
        #[command(subcommand)]
        kind: GenerateKind,
    },

    /// Run the game project
    Run {
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },

    /// Build the game project
    Build {
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },

    /// Run project health checks (cargo check + clippy + config validation)
    Check,

    /// Diagnose development environment (Rust version, GPU backend, workspace state)
    Doctor,
}

#[derive(Subcommand)]
pub enum GenerateKind {
    /// Generate an ECS component struct
    Component {
        /// Component name (PascalCase)
        name: String,
    },
    /// Generate an ECS system function
    System {
        /// System name (snake_case)
        name: String,
    },
    /// Generate an ECS resource struct
    Resource {
        /// Resource name (PascalCase)
        name: String,
    },
}
