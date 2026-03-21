use console::style;

use crate::error::Result;
use crate::workspace;

pub fn run() -> Result<()> {
    println!("{} AnvilKit Environment Diagnostics", style("→").cyan().bold());
    println!();

    // Rust toolchain
    print_section("Rust Toolchain");
    run_check("rustc", &["--version"]);
    run_check("cargo", &["--version"]);
    run_check("rustup", &["show", "active-toolchain"]);

    // GPU / wgpu backend
    print_section("System Info");
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    println!("  OS:   {} ({})", style(os).cyan(), arch);

    let backend = match os {
        "macos" => "Metal",
        "windows" => "DX12 / Vulkan",
        "linux" => "Vulkan",
        _ => "Unknown",
    };
    println!("  GPU Backend (expected): {}", style(backend).cyan());

    // Workspace status
    print_section("Workspace");
    let cwd = std::env::current_dir().unwrap_or_default();
    match workspace::find_workspace_root(&cwd) {
        Ok(root) => {
            println!("  Root: {}", style(root.display()).cyan());

            // Count workspace members using TOML parsing
            let cargo_path = root.join("Cargo.toml");
            if let Ok(content) = std::fs::read_to_string(&cargo_path) {
                if let Ok(doc) = content.parse::<toml::Value>() {
                    let member_count = doc.get("workspace")
                        .and_then(|w| w.get("members"))
                        .and_then(|m| m.as_array())
                        .map(|a| a.len())
                        .unwrap_or(0);
                    println!("  Members: {}", style(member_count).cyan());
                }
            }

            // List game projects
            let games_dir = root.join("games");
            if games_dir.exists() {
                let mut games: Vec<String> = Vec::new();
                if let Ok(entries) = std::fs::read_dir(&games_dir) {
                    for entry in entries.flatten() {
                        if entry.path().is_dir() {
                            games.push(entry.file_name().to_string_lossy().to_string());
                        }
                    }
                }
                if games.is_empty() {
                    println!("  Games: {}", style("(none)").dim());
                } else {
                    println!("  Games: {}", style(games.join(", ")).cyan());
                }
            }
        }
        Err(_) => {
            println!("  {} Not in an AnvilKit workspace", style("⚠").yellow());
        }
    }

    // External tools
    print_section("External Tools");
    check_tool("wgpu-info");
    check_tool("git");

    println!();
    println!("{} Diagnostics complete!", style("✓").green().bold());
    Ok(())
}

fn print_section(name: &str) {
    println!();
    println!("  {}", style(name).bold().underlined());
}

fn run_check(cmd: &str, args: &[&str]) {
    match std::process::Command::new(cmd).args(args).output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let first_line = stdout.lines().next().unwrap_or("(empty)");
            println!("  {} {}: {}", style("✓").green(), cmd, first_line.trim());
        }
        Ok(_) => {
            println!("  {} {}: command failed", style("⚠").yellow(), cmd);
        }
        Err(_) => {
            println!("  {} {}: not found", style("✗").red(), cmd);
        }
    }
}

fn check_tool(name: &str) {
    match which::which(name) {
        Ok(path) => println!("  {} {}: {}", style("✓").green(), name, path.display()),
        Err(_) => println!("  {} {}: not found", style("—").dim(), name),
    }
}
