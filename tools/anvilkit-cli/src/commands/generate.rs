use console::style;

use crate::error::Result;
use crate::workspace;
use crate::codegen;

pub fn component(name: &str) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let project_dir = workspace::find_game_project(&cwd)?;

    codegen::component::generate(&project_dir, name)?;

    println!(
        "{} Generated component {} in {}",
        style("✓").green().bold(),
        style(name).cyan(),
        style("src/components.rs").yellow()
    );
    Ok(())
}

pub fn system(name: &str) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let project_dir = workspace::find_game_project(&cwd)?;

    codegen::system::generate(&project_dir, name)?;

    println!(
        "{} Generated system {} in {}",
        style("✓").green().bold(),
        style(name).cyan(),
        style(format!("src/systems/{}.rs", name)).yellow()
    );
    Ok(())
}

pub fn resource(name: &str) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let project_dir = workspace::find_game_project(&cwd)?;

    codegen::resource::generate(&project_dir, name)?;

    println!(
        "{} Generated resource {} in {}",
        style("✓").green().bold(),
        style(name).cyan(),
        style("src/resources.rs").yellow()
    );
    Ok(())
}
