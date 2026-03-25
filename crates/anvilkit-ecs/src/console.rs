//! # Debug Console
//!
//! An in-game debug console that allows registering and executing named commands
//! at runtime. Output is stored in a scrollable log with categorized entries.
//!
//! ## Usage
//!
//! ```rust
//! use anvilkit_ecs::console::DebugConsole;
//!
//! let mut console = DebugConsole::new();
//! console.register("greet", "Say hello", |args| {
//!     if let Some(name) = args.first() {
//!         format!("Hello, {}!", name)
//!     } else {
//!         "Hello, world!".to_string()
//!     }
//! });
//! console.execute("greet Claude");
//! ```

use std::collections::{HashMap, VecDeque};
use bevy_ecs::prelude::Resource;
use crate::app::App;
use crate::plugin::Plugin;

/// The kind of a console output entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputKind {
    /// Informational message.
    Info,
    /// Warning message.
    Warning,
    /// Error message.
    Error,
    /// Result of a command execution.
    CommandResult,
}

/// A single console output entry.
#[derive(Debug, Clone)]
pub struct ConsoleOutput {
    /// The text content.
    pub text: String,
    /// The kind of this output entry.
    pub kind: OutputKind,
}

/// A registered console command.
pub struct ConsoleCommand {
    /// The command name (used for invocation).
    pub name: String,
    /// A short description of what the command does.
    pub description: String,
    /// The handler function: receives argument slices, returns a result string.
    pub handler: Box<dyn Fn(&[&str]) -> String + Send + Sync>,
}

/// Debug console resource.
///
/// Stores registered commands, command history, and output log. Can be toggled
/// visible/hidden for overlay rendering.
#[derive(Resource)]
pub struct DebugConsole {
    /// Registered commands, keyed by name.
    pub commands: HashMap<String, ConsoleCommand>,
    /// Command input history (most recent last).
    pub history: VecDeque<String>,
    /// Output log entries.
    pub output: VecDeque<ConsoleOutput>,
    /// Whether the console overlay is visible.
    pub visible: bool,
    /// Current input buffer text.
    pub input_buffer: String,
    /// Maximum number of history entries to keep.
    pub max_history: usize,
    /// Maximum number of output entries to keep.
    pub max_output: usize,
}

impl DebugConsole {
    /// Create a new debug console with built-in commands (`help`, `clear`).
    pub fn new() -> Self {
        let mut console = Self {
            commands: HashMap::new(),
            history: VecDeque::new(),
            output: VecDeque::new(),
            visible: false,
            input_buffer: String::new(),
            max_history: 100,
            max_output: 200,
        };

        // Register built-in "help" — we store a placeholder and handle it
        // specially in execute() so it can read the command list.
        console.commands.insert(
            "help".to_string(),
            ConsoleCommand {
                name: "help".to_string(),
                description: "List all available commands".to_string(),
                handler: Box::new(|_| String::new()), // placeholder
            },
        );

        console.commands.insert(
            "clear".to_string(),
            ConsoleCommand {
                name: "clear".to_string(),
                description: "Clear console output".to_string(),
                handler: Box::new(|_| String::new()), // placeholder
            },
        );

        console
    }

    /// Register a new command.
    ///
    /// # Parameters
    ///
    /// - `name`: The command name (single word).
    /// - `description`: A short description for the help listing.
    /// - `handler`: A closure that receives argument slices and returns a result string.
    pub fn register<F>(&mut self, name: &str, description: &str, handler: F)
    where
        F: Fn(&[&str]) -> String + Send + Sync + 'static,
    {
        self.commands.insert(
            name.to_string(),
            ConsoleCommand {
                name: name.to_string(),
                description: description.to_string(),
                handler: Box::new(handler),
            },
        );
    }

    /// Execute a command string (command name followed by space-separated arguments).
    pub fn execute(&mut self, input: &str) {
        let input = input.trim();
        if input.is_empty() {
            return;
        }

        // Store in history
        self.history.push_back(input.to_string());
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }

        // Parse command and args
        let parts: Vec<&str> = input.split_whitespace().collect();
        let cmd_name = parts[0];
        let args: Vec<&str> = parts[1..].to_vec();

        // Handle built-in "help" specially so it can enumerate commands
        if cmd_name == "help" {
            let mut lines = vec!["Available commands:".to_string()];
            let mut names: Vec<&String> = self.commands.keys().collect();
            names.sort();
            for name in names {
                if let Some(cmd) = self.commands.get(name) {
                    lines.push(format!("  {} — {}", cmd.name, cmd.description));
                }
            }
            let result = lines.join("\n");
            self.push_output(result, OutputKind::CommandResult);
            return;
        }

        // Handle built-in "clear"
        if cmd_name == "clear" {
            self.output.clear();
            return;
        }

        // Look up and execute
        if let Some(cmd) = self.commands.get(cmd_name) {
            let result = (cmd.handler)(&args);
            self.push_output(result, OutputKind::CommandResult);
        } else {
            self.push_output(
                format!("Unknown command: '{}'", cmd_name),
                OutputKind::Error,
            );
        }
    }

    /// Toggle console visibility.
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Log an informational message to the console output.
    pub fn log(&mut self, text: &str) {
        self.push_output(text.to_string(), OutputKind::Info);
    }

    /// Log a warning message to the console output.
    pub fn warn(&mut self, text: &str) {
        self.push_output(text.to_string(), OutputKind::Warning);
    }

    /// Log an error message to the console output.
    pub fn error(&mut self, text: &str) {
        self.push_output(text.to_string(), OutputKind::Error);
    }

    /// Internal helper to push an output entry and enforce the max_output limit.
    fn push_output(&mut self, text: String, kind: OutputKind) {
        self.output.push_back(ConsoleOutput { text, kind });
        while self.output.len() > self.max_output {
            self.output.pop_front();
        }
    }
}

impl Default for DebugConsole {
    fn default() -> Self {
        Self::new()
    }
}

/// Plugin that inserts the [`DebugConsole`] resource into the ECS world.
pub struct ConsolePlugin;

impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DebugConsole::new());
    }

    fn name(&self) -> &str {
        "ConsolePlugin"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_console_register_execute() {
        let mut console = DebugConsole::new();
        console.register("echo", "Echo back arguments", |args| {
            args.join(" ")
        });

        console.execute("echo hello world");

        assert_eq!(console.output.len(), 1);
        assert_eq!(console.output[0].text, "hello world");
        assert_eq!(console.output[0].kind, OutputKind::CommandResult);
    }

    #[test]
    fn test_debug_console_help() {
        let mut console = DebugConsole::new();
        console.register("test_cmd", "A test command", |_| "ok".to_string());

        console.execute("help");

        assert_eq!(console.output.len(), 1);
        assert_eq!(console.output[0].kind, OutputKind::CommandResult);
        let text = &console.output[0].text;
        assert!(
            text.contains("Available commands:"),
            "Help output should contain header"
        );
        assert!(
            text.contains("test_cmd"),
            "Help output should list registered commands"
        );
        assert!(
            text.contains("help"),
            "Help output should list the help command itself"
        );
    }

    #[test]
    fn test_debug_console_unknown_command() {
        let mut console = DebugConsole::new();

        console.execute("nonexistent_cmd");

        assert_eq!(console.output.len(), 1);
        assert_eq!(console.output[0].kind, OutputKind::Error);
        assert!(console.output[0].text.contains("Unknown command"));
    }

    #[test]
    fn test_debug_console_clear() {
        let mut console = DebugConsole::new();
        console.log("some log");
        console.warn("some warning");
        assert_eq!(console.output.len(), 2);

        console.execute("clear");
        assert!(console.output.is_empty(), "clear should remove all output");
    }

    #[test]
    fn test_debug_console_toggle() {
        let mut console = DebugConsole::new();
        assert!(!console.visible);
        console.toggle();
        assert!(console.visible);
        console.toggle();
        assert!(!console.visible);
    }

    #[test]
    fn test_debug_console_log_warn_error() {
        let mut console = DebugConsole::new();
        console.log("info");
        console.warn("warning");
        console.error("error");

        assert_eq!(console.output.len(), 3);
        assert_eq!(console.output[0].kind, OutputKind::Info);
        assert_eq!(console.output[1].kind, OutputKind::Warning);
        assert_eq!(console.output[2].kind, OutputKind::Error);
    }

    #[test]
    fn test_debug_console_history() {
        let mut console = DebugConsole::new();
        console.execute("help");
        console.execute("help");

        assert_eq!(console.history.len(), 2);
        assert_eq!(console.history[0], "help");
    }

    #[test]
    fn test_console_plugin() {
        let mut app = crate::app::App::new();
        let plugin = ConsolePlugin;
        plugin.build(&mut app);

        let console = app.world.get_resource::<DebugConsole>();
        assert!(console.is_some(), "ConsolePlugin should insert DebugConsole resource");
    }
}
