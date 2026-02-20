use std::path::Path;

use crate::error::Result;

pub fn generate(project_dir: &Path, name: &str) -> Result<()> {
    let resources_path = project_dir.join("src/resources.rs");

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
    content.push_str(&code);
    std::fs::write(&resources_path, content)?;

    Ok(())
}
