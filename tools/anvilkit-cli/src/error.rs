use std::fmt;

#[derive(Debug)]
pub enum CliError {
    NotInWorkspace,
    NotInGameProject,
    ProjectAlreadyExists(String),
    InvalidName(String),
    TemplateNotFound(String),
    IoError(std::io::Error),
    TomlError(String),
    CargoFailed(String),
    InvalidInput(String),
    Other(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInWorkspace => write!(f, "Not inside an AnvilKit workspace. Run this command from the workspace root or a game project directory."),
            Self::NotInGameProject => write!(f, "Not inside a game project directory. Run this command from a games/<project> directory."),
            Self::ProjectAlreadyExists(name) => write!(f, "Project '{}' already exists.", name),
            Self::InvalidName(name) => write!(f, "Invalid project name '{}'. Use snake_case (e.g., my_game).", name),
            Self::TemplateNotFound(name) => write!(f, "Template '{}' not found. Available: 3d-basic, topdown, first-person, empty", name),
            Self::IoError(e) => write!(f, "I/O error: {}", e),
            Self::TomlError(msg) => write!(f, "TOML error: {}", msg),
            Self::CargoFailed(msg) => write!(f, "Cargo command failed: {}", msg),
            Self::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for CliError {}

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

pub type Result<T> = std::result::Result<T, CliError>;
