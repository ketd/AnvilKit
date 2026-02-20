/// A template file: relative path + content.
pub struct TemplateFile {
    pub path: &'static str,
    pub content: &'static str,
}

/// Describes a project template.
pub struct TemplateInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub files: &'static [TemplateFile],
}

// ---- 3d-basic template ----

static TPL_3D_BASIC: &[TemplateFile] = &[
    TemplateFile { path: "Cargo.toml",             content: include_str!("3d_basic/Cargo.toml.hbs") },
    TemplateFile { path: "anvilkit.toml",           content: include_str!("3d_basic/anvilkit.toml.hbs") },
    TemplateFile { path: "src/main.rs",             content: include_str!("3d_basic/main.rs.hbs") },
    TemplateFile { path: "src/lib.rs",              content: include_str!("3d_basic/lib.rs.hbs") },
    TemplateFile { path: "src/config.rs",           content: include_str!("3d_basic/config.rs.hbs") },
    TemplateFile { path: "src/components.rs",        content: include_str!("3d_basic/components.rs.hbs") },
    TemplateFile { path: "src/resources.rs",         content: include_str!("3d_basic/resources.rs.hbs") },
    TemplateFile { path: "src/systems/mod.rs",       content: include_str!("3d_basic/systems_mod.rs.hbs") },
    TemplateFile { path: "src/systems/input.rs",     content: include_str!("3d_basic/input.rs.hbs") },
    TemplateFile { path: "src/render/mod.rs",        content: include_str!("3d_basic/render_mod.rs.hbs") },
    TemplateFile { path: "src/render/setup.rs",      content: include_str!("3d_basic/setup.rs.hbs") },
    TemplateFile { path: "src/render/colors.rs",     content: include_str!("3d_basic/colors.rs.hbs") },
];

// ---- empty template ----

static TPL_EMPTY: &[TemplateFile] = &[
    TemplateFile { path: "Cargo.toml",             content: include_str!("empty/Cargo.toml.hbs") },
    TemplateFile { path: "anvilkit.toml",           content: include_str!("empty/anvilkit.toml.hbs") },
    TemplateFile { path: "src/main.rs",             content: include_str!("empty/main.rs.hbs") },
    TemplateFile { path: "src/lib.rs",              content: include_str!("empty/lib.rs.hbs") },
];

// ---- topdown template ----

static TPL_TOPDOWN: &[TemplateFile] = &[
    TemplateFile { path: "Cargo.toml",             content: include_str!("topdown/Cargo.toml.hbs") },
    TemplateFile { path: "anvilkit.toml",           content: include_str!("topdown/anvilkit.toml.hbs") },
    TemplateFile { path: "src/main.rs",             content: include_str!("topdown/main.rs.hbs") },
    TemplateFile { path: "src/lib.rs",              content: include_str!("topdown/lib.rs.hbs") },
    TemplateFile { path: "src/config.rs",           content: include_str!("topdown/config.rs.hbs") },
    TemplateFile { path: "src/components.rs",        content: include_str!("topdown/components.rs.hbs") },
    TemplateFile { path: "src/resources.rs",         content: include_str!("topdown/resources.rs.hbs") },
    TemplateFile { path: "src/systems/mod.rs",       content: include_str!("topdown/systems_mod.rs.hbs") },
    TemplateFile { path: "src/systems/input.rs",     content: include_str!("topdown/input.rs.hbs") },
    TemplateFile { path: "src/render/mod.rs",        content: include_str!("topdown/render_mod.rs.hbs") },
    TemplateFile { path: "src/render/setup.rs",      content: include_str!("topdown/setup.rs.hbs") },
    TemplateFile { path: "src/render/colors.rs",     content: include_str!("topdown/colors.rs.hbs") },
];

// ---- first-person template ----

static TPL_FIRST_PERSON: &[TemplateFile] = &[
    TemplateFile { path: "Cargo.toml",             content: include_str!("first_person/Cargo.toml.hbs") },
    TemplateFile { path: "anvilkit.toml",           content: include_str!("first_person/anvilkit.toml.hbs") },
    TemplateFile { path: "src/main.rs",             content: include_str!("first_person/main.rs.hbs") },
    TemplateFile { path: "src/lib.rs",              content: include_str!("first_person/lib.rs.hbs") },
    TemplateFile { path: "src/config.rs",           content: include_str!("first_person/config.rs.hbs") },
    TemplateFile { path: "src/components.rs",        content: include_str!("first_person/components.rs.hbs") },
    TemplateFile { path: "src/resources.rs",         content: include_str!("first_person/resources.rs.hbs") },
    TemplateFile { path: "src/systems/mod.rs",       content: include_str!("first_person/systems_mod.rs.hbs") },
    TemplateFile { path: "src/systems/input.rs",     content: include_str!("first_person/input.rs.hbs") },
    TemplateFile { path: "src/render/mod.rs",        content: include_str!("first_person/render_mod.rs.hbs") },
    TemplateFile { path: "src/render/setup.rs",      content: include_str!("first_person/setup.rs.hbs") },
    TemplateFile { path: "src/render/colors.rs",     content: include_str!("first_person/colors.rs.hbs") },
];

/// All available templates.
pub fn all_templates() -> Vec<TemplateInfo> {
    vec![
        TemplateInfo {
            name: "3d-basic",
            description: "Basic 3D scene with PBR sphere, ground plane, and lighting",
            files: TPL_3D_BASIC,
        },
        TemplateInfo {
            name: "topdown",
            description: "Top-down view with WASD player movement and obstacles",
            files: TPL_TOPDOWN,
        },
        TemplateInfo {
            name: "first-person",
            description: "First-person camera with WASD + mouse look",
            files: TPL_FIRST_PERSON,
        },
        TemplateInfo {
            name: "empty",
            description: "Minimal skeleton with window and camera only",
            files: TPL_EMPTY,
        },
    ]
}

/// Look up a template by name.
pub fn get_template(name: &str) -> Option<TemplateInfo> {
    all_templates().into_iter().find(|t| t.name == name)
}

/// Get template names for display.
pub fn template_names() -> Vec<&'static str> {
    vec!["3d-basic", "topdown", "first-person", "empty"]
}

pub fn template_display_items() -> Vec<String> {
    all_templates()
        .iter()
        .map(|t| format!("{:<15} {}", t.name, t.description))
        .collect()
}
