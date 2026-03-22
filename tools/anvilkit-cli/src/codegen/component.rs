use std::path::Path;

use crate::error::Result;
use super::validate_identifier;

pub fn generate(project_dir: &Path, name: &str) -> Result<()> {
    validate_identifier(name)?;

    let components_path = project_dir.join("src/components.rs");

    let import_line = "use bevy_ecs::prelude::*;\n";
    let code = format!(
        r#"
/// {name} component.
#[derive(Debug, Clone, Component)]
pub struct {name} {{
    pub value: f32,
}}
"#,
        name = name
    );

    let mut content = std::fs::read_to_string(&components_path)?;
    // Ensure the import exists
    if !content.contains("use bevy_ecs::prelude::*;") {
        content = format!("{}{}", import_line, content);
    }
    content.push_str(&code);
    std::fs::write(&components_path, content)?;

    Ok(())
}
