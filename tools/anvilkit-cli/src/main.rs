use clap::Parser;
use console::style;

mod cli;
mod config;
mod error;
mod workspace;
mod commands;
mod codegen;
mod templates;

use cli::{Cli, Command, GenerateKind};

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::New { name, template } => commands::new::run(&name, template.as_deref()),
        Command::Generate { kind } => match kind {
            GenerateKind::Component { name } => commands::generate::component(&name),
            GenerateKind::System { name } => commands::generate::system(&name),
            GenerateKind::Resource { name } => commands::generate::resource(&name),
        },
        Command::Run { release, watch } => commands::run::run(release, watch),
        Command::Build { release } => commands::build::run(release),
        Command::Check => commands::check::run(),
        Command::Doctor => commands::doctor::run(),
    };

    if let Err(e) = result {
        eprintln!("{} {}", style("error:").red().bold(), e);
        std::process::exit(1);
    }
}
