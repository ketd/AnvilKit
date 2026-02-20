pub mod registry;

use std::collections::HashMap;
use handlebars::Handlebars;

use crate::error::{CliError, Result};

/// Context variables available to all templates.
pub struct TemplateContext {
    pub project_name: String,
    pub display_name: String,
    pub crate_depth: String,
    pub shader_depth: String,
}

impl TemplateContext {
    pub fn to_map(&self) -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("project_name".into(), self.project_name.clone());
        m.insert("display_name".into(), self.display_name.clone());
        m.insert("crate_depth".into(), self.crate_depth.clone());
        m.insert("shader_depth".into(), self.shader_depth.clone());
        m
    }
}

/// Render a template string with the given context map.
pub fn render_template(template: &str, ctx: &HashMap<String, String>) -> Result<String> {
    let mut hbs = Handlebars::new();
    hbs.set_strict_mode(false);
    hbs.register_template_string("tpl", template)
        .map_err(|e| CliError::Other(format!("Template parse error: {}", e)))?;
    hbs.render("tpl", ctx)
        .map_err(|e| CliError::Other(format!("Template render error: {}", e)))
}
