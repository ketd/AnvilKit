use std::path::Path;

use crate::error::Result;

pub fn generate(project_dir: &Path, name: &str) -> Result<()> {
    let components_path = project_dir.join("src/components.rs");

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
    content.push_str(&code);
    std::fs::write(&components_path, content)?;

    Ok(())
}
