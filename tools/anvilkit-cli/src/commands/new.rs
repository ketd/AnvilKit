use console::style;
use dialoguer::Select;
use indicatif::{ProgressBar, ProgressStyle};

use crate::error::{CliError, Result};
use crate::templates::{self, TemplateContext};
use crate::templates::registry;
use crate::workspace;

pub fn run(name: &str, template: Option<&str>) -> Result<()> {
    // Validate name
    if !is_valid_name(name) {
        return Err(CliError::InvalidName(name.to_string()));
    }

    let cwd = std::env::current_dir()?;
    let ws_root = workspace::find_workspace_root(&cwd)?;

    let project_dir = ws_root.join("games").join(name);
    if project_dir.exists() {
        return Err(CliError::ProjectAlreadyExists(name.to_string()));
    }

    // Select template
    let template_name = match template {
        Some(t) => {
            if registry::get_template(t).is_none() {
                return Err(CliError::TemplateNotFound(t.to_string()));
            }
            t.to_string()
        }
        None => {
            let items = registry::template_display_items();
            let names = registry::template_names();
            let selection = Select::new()
                .with_prompt("Select a project template")
                .items(&items)
                .default(0)
                .interact()
                .map_err(|e| CliError::Other(format!("Selection cancelled: {}", e)))?;
            names[selection].to_string()
        }
    };

    let tpl = registry::get_template(&template_name)
        .ok_or_else(|| CliError::TemplateNotFound(template_name.clone()))?;

    println!(
        "{} Creating project {} with template {}",
        style("→").cyan().bold(),
        style(name).green().bold(),
        style(&template_name).yellow()
    );

    // Build template context
    let display_name = name
        .split('_')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let ctx = TemplateContext {
        project_name: name.to_string(),
        display_name,
        crate_depth: "../../".to_string(),
        shader_depth: "../../../../".to_string(),
    };
    let ctx_map = ctx.to_map();

    // Generate files
    let pb = ProgressBar::new(tpl.files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("  {spinner:.green} [{bar:30}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("█▓░"),
    );

    for file in tpl.files {
        let rendered = templates::render_template(file.content, &ctx_map)?;
        let dest = project_dir.join(file.path);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&dest, rendered)?;
        pb.set_message(file.path.to_string());
        pb.inc(1);
    }
    pb.finish_with_message("done");

    // Add to workspace
    let member = format!("games/{}", name);
    workspace::add_workspace_member(&ws_root, &member)?;
    println!(
        "  {} Added {} to workspace members",
        style("✓").green(),
        style(&member).cyan()
    );

    // Run cargo check
    println!(
        "  {} Running cargo check...",
        style("→").cyan()
    );
    let check = std::process::Command::new("cargo")
        .arg("check")
        .arg("-p")
        .arg(name)
        .current_dir(&ws_root)
        .status();

    match check {
        Ok(status) if status.success() => {
            println!(
                "  {} Project compiles successfully!",
                style("✓").green()
            );
        }
        Ok(_) => {
            println!(
                "  {} cargo check failed — template may need adjustment",
                style("⚠").yellow()
            );
        }
        Err(e) => {
            println!(
                "  {} Could not run cargo check: {}",
                style("⚠").yellow(),
                e
            );
        }
    }

    println!();
    println!(
        "{} Project created at {}",
        style("✓").green().bold(),
        style(project_dir.display()).cyan()
    );
    println!();
    println!("  Next steps:");
    println!("    cargo run -p {}", name);
    println!("    anvil generate component PlayerHealth");
    println!("    anvil generate system player_movement");

    Ok(())
}

fn is_valid_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    // Must be valid Rust identifier (snake_case)
    name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        && name.chars().next().map_or(false, |c| c.is_ascii_lowercase())
}
