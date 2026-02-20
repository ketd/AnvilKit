use console::style;

use crate::error::{CliError, Result};
use crate::workspace;

pub fn run() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let ws_root = workspace::find_workspace_root(&cwd)?;

    println!("{} Running project health checks...", style("→").cyan().bold());

    // 1. cargo check
    println!("  {} cargo check --workspace", style("→").dim());
    let status = std::process::Command::new("cargo")
        .args(["check", "--workspace"])
        .current_dir(&ws_root)
        .status()
        .map_err(|e| CliError::CargoFailed(e.to_string()))?;

    if status.success() {
        println!("  {} cargo check passed", style("✓").green());
    } else {
        println!("  {} cargo check failed", style("✗").red());
        return Err(CliError::CargoFailed("cargo check failed".into()));
    }

    // 2. clippy
    println!("  {} cargo clippy --workspace", style("→").dim());
    let status = std::process::Command::new("cargo")
        .args(["clippy", "--workspace", "--", "-D", "warnings"])
        .current_dir(&ws_root)
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("  {} clippy passed", style("✓").green());
        }
        Ok(_) => {
            println!("  {} clippy found warnings", style("⚠").yellow());
        }
        Err(_) => {
            println!("  {} clippy not available", style("⚠").yellow());
        }
    }

    // 3. Validate anvilkit.toml if in game project
    if let Ok(project_dir) = workspace::find_game_project(&cwd) {
        let toml_path = project_dir.join("anvilkit.toml");
        if toml_path.exists() {
            let content = std::fs::read_to_string(&toml_path)?;
            match toml::from_str::<crate::config::AnvilKitConfig>(&content) {
                Ok(_) => println!("  {} anvilkit.toml is valid", style("✓").green()),
                Err(e) => println!("  {} anvilkit.toml error: {}", style("✗").red(), e),
            }
        }
    }

    println!();
    println!("{} Health check complete!", style("✓").green().bold());
    Ok(())
}
