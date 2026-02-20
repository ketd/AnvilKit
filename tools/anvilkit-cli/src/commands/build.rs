use console::style;

use crate::error::{CliError, Result};
use crate::workspace;

pub fn run(release: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let ws_root = workspace::find_workspace_root(&cwd)?;

    let package = match workspace::find_game_project(&cwd) {
        Ok(project_dir) => workspace::read_package_name(&project_dir)?,
        Err(_) => {
            return Err(CliError::NotInGameProject);
        }
    };

    println!(
        "{} Building {}{}",
        style("→").cyan().bold(),
        style(&package).green(),
        if release { style(" (release)").yellow().to_string() } else { String::new() }
    );

    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("build").arg("-p").arg(&package);
    if release {
        cmd.arg("--release");
    }
    cmd.current_dir(&ws_root);

    let status = cmd.status()
        .map_err(|e| CliError::CargoFailed(e.to_string()))?;

    if !status.success() {
        return Err(CliError::CargoFailed(format!("exit code: {:?}", status.code())));
    }

    println!("{} Build complete!", style("✓").green().bold());
    Ok(())
}
