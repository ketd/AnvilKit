use std::path::Path;

use crate::error::Result;
use super::validate_identifier;

pub fn generate(project_dir: &Path, name: &str) -> Result<()> {
    validate_identifier(name)?;

    let resources_path = project_dir.join("src/resources.rs");

    let import_line = "use bevy_ecs::prelude::*;\n";
    let code = format!(
        r#"
/// {name} resource.
#[derive(Debug, Default, Resource)]
pub struct {name} {{
    // TODO: add fields
}}
"#,
        name = name
    );

    let mut content = std::fs::read_to_string(&resources_path)?;
    if !content.contains("use bevy_ecs::prelude::*;") {
        content = format!("{}{}", import_line, content);
    }
    content.push_str(&code);
    std::fs::write(&resources_path, content)?;

    Ok(())
}
