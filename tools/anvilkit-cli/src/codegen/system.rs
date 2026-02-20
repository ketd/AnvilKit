use std::path::Path;

use crate::error::Result;

pub fn generate(project_dir: &Path, name: &str) -> Result<()> {
    let systems_dir = project_dir.join("src/systems");
    std::fs::create_dir_all(&systems_dir)?;

    // Create system file
    let system_path = systems_dir.join(format!("{}.rs", name));
    let code = format!(
        r#"use bevy_ecs::prelude::*;
use anvilkit_core::math::Transform;
use anvilkit_ecs::physics::DeltaTime;

pub fn {name}_system(
    _dt: Res<DeltaTime>,
    mut _query: Query<&mut Transform>,
) {{
    // TODO: implement {name}
}}
"#,
        name = name
    );
    std::fs::write(&system_path, code)?;

    // Add mod declaration to systems/mod.rs
    let mod_path = systems_dir.join("mod.rs");
    let mod_line = format!("pub mod {};\n", name);
    let mut content = std::fs::read_to_string(&mod_path).unwrap_or_default();
    if !content.contains(&format!("pub mod {};", name)) {
        content.push_str(&mod_line);
        std::fs::write(&mod_path, content)?;
    }

    Ok(())
}
