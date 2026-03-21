use std::path::{Path, PathBuf};

use crate::error::{CliError, Result};

/// Walk up from `start` looking for a `Cargo.toml` with a `[workspace]` section.
/// Uses proper TOML parsing to avoid false positives from comments.
pub fn find_workspace_root(start: &Path) -> Result<PathBuf> {
    let mut dir = if start.is_file() {
        start.parent().unwrap_or(start).to_path_buf()
    } else {
        start.to_path_buf()
    };

    loop {
        let cargo_path = dir.join("Cargo.toml");
        if cargo_path.exists() {
            let content = std::fs::read_to_string(&cargo_path)
                .map_err(CliError::IoError)?;
            if let Ok(doc) = content.parse::<toml::Value>() {
                if doc.get("workspace").is_some() {
                    return Ok(dir);
                }
            }
        }
        if !dir.pop() {
            return Err(CliError::NotInWorkspace);
        }
    }
}

/// Add a member path (e.g. `"games/my_game"`) to the workspace `Cargo.toml`.
pub fn add_workspace_member(workspace_root: &Path, member: &str) -> Result<()> {
    let cargo_path = workspace_root.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_path)?;

    let mut doc = content.parse::<toml_edit::DocumentMut>()
        .map_err(|e| CliError::TomlError(e.to_string()))?;

    let members = doc["workspace"]["members"]
        .as_array_mut()
        .ok_or_else(|| CliError::TomlError("workspace.members is not an array".into()))?;

    // Check if already present
    let already = members.iter().any(|v| v.as_str() == Some(member));
    if already {
        return Ok(());
    }

    members.push(member);

    std::fs::write(&cargo_path, doc.to_string())?;
    Ok(())
}

/// Detect if the current directory is a game project (has anvilkit.toml).
pub fn find_game_project(start: &Path) -> Result<PathBuf> {
    let mut dir = start.to_path_buf();
    loop {
        if dir.join("anvilkit.toml").exists() {
            return Ok(dir);
        }
        // Don't go above workspace root
        let cargo = dir.join("Cargo.toml");
        if cargo.exists() {
            if let Ok(content) = std::fs::read_to_string(&cargo) {
                if let Ok(doc) = content.parse::<toml::Value>() {
                    if doc.get("workspace").is_some() {
                        return Err(CliError::NotInGameProject);
                    }
                }
            }
        }
        if !dir.pop() {
            return Err(CliError::NotInGameProject);
        }
    }
}

/// Read the package name from anvilkit.toml in the given project dir.
pub fn read_package_name(project_dir: &Path) -> Result<String> {
    let toml_path = project_dir.join("anvilkit.toml");
    let content = std::fs::read_to_string(&toml_path)?;
    let config: toml::Value = content.parse()
        .map_err(|e: toml::de::Error| CliError::TomlError(e.to_string()))?;
    config.get("project")
        .and_then(|p| p.get("package"))
        .and_then(|p| p.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| CliError::TomlError("missing project.package in anvilkit.toml".into()))
}
