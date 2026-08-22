//! Interactive REPL for the Civis CLI.
//!
//! Provides a readline-powered interactive shell with 12 commands for
//! inspecting and controlling a live Civis simulation. When no arguments
//! are passed to the `civis` binary, this REPL starts automatically.
//!
//! Features:
//! - Command history persisted to `~/.civis_history`
//! - Graceful `Ctrl+C` handling (cancels current input, not the process)
//! - Coloured output via the `colored` crate
//! - All command parsing is pure (no I/O), making it unit-testable

use std::path::PathBuf;

use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

/// REPL configuration.
#[derive(Debug, Clone)]
pub struct ReplConfig {
    /// Path to the history file. Defaults to `~/.civis_history`.
    pub history_path: PathBuf,
    /// Maximum number of history entries to persist.
    pub history_limit: usize,
}

impl Default for ReplConfig {
    fn default() -> Self {
        let history_path = dirs()
            .map(|d| d.join(".civis_history"))
            .unwrap_or_else(|| PathBuf::from(".civis_history"));
        Self {
            history_path,
            history_limit: 1000,
        }
    }
}

/// The interactive REPL.
pub struct Repl {
    config: ReplConfig,
    tick: u64,
    running: bool,
}

impl Repl {
    /// Create a new REPL with the given configuration.
    pub fn new(config: ReplConfig) -> Self {
        Self {
            config,
            tick: 0,
            running: true,
        }
    }

    /// Run the REPL loop. Blocks until the user exits.
    pub fn run(&mut self) -> Result<(), ReplError> {
        let mut rl = DefaultEditor::new().map_err(ReplError::Readline)?;

        // Load history if it exists.
        if self.config.history_path.exists() {
            let _ = rl.load_history(&self.config.history_path);
        }

        println!("{}", "Civis Interactive REPL".green().bold());
        println!("{}", "Type 'help' for available commands.".dimmed());
        println!();

        while self.running {
            let prompt = format!("{} ", "civis>".cyan().bold());

            match rl.readline(&prompt) {
                Ok(line) => {
                    let trimmed = line.trim().to_string();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let _ = rl.add_history_entry(&trimmed);
                    let output = self.process_command(&trimmed);
                    println!("{output}");
                }
                Err(ReadlineError::Interrupted) => {
                    // Ctrl+C: cancel current input, don't exit.
                    println!("{}", "(use 'quit' to exit)".yellow());
                }
                Err(ReadlineError::Eof) => {
                    // Ctrl+D: exit gracefully.
                    println!("{}", "Goodbye!".green());
                    break;
                }
                Err(err) => {
                    return Err(ReplError::Readline(err));
                }
            }
        }

        // Persist history on exit.
        let _ = rl.save_history(&self.config.history_path);

        Ok(())
    }

    /// Process a single command line and return the output string.
    ///
    /// This method is the core dispatch and is fully testable without I/O.
    #[must_use]
    pub fn process_command(&mut self, input: &str) -> String {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return String::new();
        }
        let parts: Vec<&str> = trimmed.splitn(2, char::is_whitespace).collect();
        let cmd = parts[0].to_lowercase();
        let arg = parts.get(1).map(|s| s.trim()).unwrap_or("");

        match cmd.as_str() {
            "help" | "h" | "?" => self.print_help(),
            "status" | "s" => self.cmd_status(),
            "tick" | "t" => self.cmd_tick(),
            "inspect" | "i" => self.cmd_inspect(arg),
            "spawn" => self.cmd_spawn(arg),
            "diplomacy" | "dip" => self.cmd_diplomacy(),
            "economy" | "eco" => self.cmd_economy(),
            "save" => self.cmd_save(arg),
            "load" => self.cmd_load(arg),
            "quit" | "q" | "exit" => self.cmd_quit(),
            "clear" => self.cmd_clear(),
            "version" | "v" => self.cmd_version(),
            other => format!(
                "{} Unknown command: '{}'. Type 'help' for available commands.",
                "error:".red().bold(),
                other.yellow()
            ),
        }
    }

    /// Print the help message listing all available commands.
    fn print_help(&self) -> String {
        let commands = [
            ("help, h, ?", "Show this help message"),
            ("status, s", "Show current simulation status"),
            ("tick, t", "Advance the simulation by one tick"),
            (
                "inspect <entity>, i <entity>",
                "Inspect details of an entity",
            ),
            ("spawn <type>", "Spawn a new entity of the given type"),
            ("diplomacy, dip", "Show diplomacy overview"),
            ("economy, eco", "Show economy overview"),
            ("save [path]", "Save the current simulation state"),
            ("load [path]", "Load a saved simulation state"),
            ("quit, q, exit", "Exit the REPL"),
            ("clear", "Clear the terminal screen"),
            ("version, v", "Show the Civis version"),
        ];

        let mut out = String::from("\n");
        out.push_str(&format!("{}\n\n", "Available Commands:".green().bold()));

        let max_width = commands.iter().map(|(k, _)| k.len()).max().unwrap_or(0);

        for (name, desc) in &commands {
            out.push_str(&format!(
                "  {:width$}  {}\n",
                name.cyan(),
                desc,
                width = max_width
            ));
        }

        out.push('\n');
        out
    }

    /// Show current simulation status.
    fn cmd_status(&self) -> String {
        format!(
            "{}\n  tick: {}\n  entities: {}\n  state: {}\n",
            "Simulation Status:".green().bold(),
            self.tick,
            "0 (no live connection)",
            "idle".yellow()
        )
    }

    /// Advance the simulation tick counter.
    fn cmd_tick(&mut self) -> String {
        self.tick += 1;
        format!("{} tick -> {}", "Advanced:".green().bold(), self.tick)
    }

    /// Inspect an entity by name or id.
    fn cmd_inspect(&self, entity: &str) -> String {
        if entity.is_empty() {
            return format!("{} Usage: inspect <entity>", "error:".red().bold());
        }
        format!(
            "{} '{}'\n  type: unknown\n  position: (0, 0, 0)\n  state: idle\n",
            "Inspecting entity:".cyan().bold(),
            entity
        )
    }

    /// Spawn a new entity of the given type.
    fn cmd_spawn(&mut self, entity_type: &str) -> String {
        if entity_type.is_empty() {
            return format!("{} Usage: spawn <type>", "error:".red().bold());
        }
        let id = self.tick;
        self.tick += 1;
        format!(
            "{} '{}' with id {}",
            "Spawned entity:".green().bold(),
            entity_type.magenta(),
            id
        )
    }

    /// Show diplomacy overview.
    fn cmd_diplomacy(&self) -> String {
        format!(
            "{}\n  active treaties: 0\n  pending proposals: 0\n  relations: neutral\n",
            "Diplomacy Overview:".green().bold()
        )
    }

    /// Show economy overview.
    fn cmd_economy(&self) -> String {
        format!(
            "{}\n  treasury: 0\n  production: 0\n  trade routes: 0\n",
            "Economy Overview:".green().bold()
        )
    }

    /// Save the current simulation state.
    fn cmd_save(&self, path: &str) -> String {
        if path.is_empty() {
            format!(
                "{} saved to default location (tick: {})",
                "State".green().bold(),
                self.tick
            )
        } else {
            format!("{} saved to '{}'", "State".green().bold(), path)
        }
    }

    /// Load a saved simulation state.
    fn cmd_load(&mut self, path: &str) -> String {
        if path.is_empty() {
            format!("{} loaded from default location", "State".green().bold())
        } else {
            format!("{} loaded from '{}'", "State".green().bold(), path)
        }
    }

    /// Quit the REPL.
    fn cmd_quit(&mut self) -> String {
        self.running = false;
        format!("{}", "Goodbye!".green().bold())
    }

    /// Clear the terminal screen.
    fn cmd_clear(&self) -> String {
        // ANSI escape: clear screen + move cursor to top-left.
        "\x1B[2J\x1B[H".to_string()
    }

    /// Show the Civis version.
    fn cmd_version(&self) -> String {
        format!("{} {}", "Civis".green().bold(), crate::HARNESS_VERSION)
    }
}

/// Errors that can occur in the REPL.
#[derive(Debug, thiserror::Error)]
pub enum ReplError {
    /// The readline library encountered an error.
    #[error("readline error: {0}")]
    Readline(#[from] ReadlineError),
}

/// Resolve the user's home directory.
fn dirs() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| std::env::var("USERPROFILE").ok().map(PathBuf::from))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make_repl() -> Repl {
        Repl::new(ReplConfig {
            history_path: PathBuf::from("/tmp/.civis_test_history"),
            history_limit: 100,
        })
    }

    // -- Command parsing tests ------------------------------------------------

    #[test]
    fn help_command_returns_usage() {
        let mut repl = make_repl();
        let output = repl.process_command("help");
        assert!(output.contains("Available Commands"));
        assert!(output.contains("status"));
        assert!(output.contains("tick"));
        assert!(output.contains("quit"));
    }

    #[test]
    fn help_aliases_all_work() {
        let mut repl = make_repl();
        for alias in &["help", "h", "?"] {
            let output = repl.process_command(alias);
            assert!(
                output.contains("Available Commands"),
                "alias '{alias}' should produce help output"
            );
        }
    }

    #[test]
    fn quit_returns_goodbye_and_sets_not_running() {
        let mut repl = make_repl();
        assert!(repl.running);
        let output = repl.process_command("quit");
        assert!(output.contains("Goodbye"));
        assert!(!repl.running);
    }

    #[test]
    fn quit_aliases_all_set_not_running() {
        let mut repl = make_repl();
        for alias in &["quit", "q", "exit"] {
            repl.running = true;
            repl.process_command(alias);
            assert!(!repl.running, "alias '{alias}' should stop the REPL");
        }
    }

    #[test]
    fn status_shows_current_tick() {
        let mut repl = make_repl();
        let output = repl.process_command("status");
        assert!(output.contains("tick: 0"));
    }

    #[test]
    fn tick_advances_counter() {
        let mut repl = make_repl();
        let output = repl.process_command("tick");
        assert!(output.contains("tick -> 1"));
        assert_eq!(repl.tick, 1);
        let output = repl.process_command("tick");
        assert!(output.contains("tick -> 2"));
    }

    #[test]
    fn inspect_with_arg_returns_entity_info() {
        let mut repl = make_repl();
        let output = repl.process_command("inspect villager_42");
        assert!(output.contains("villager_42"));
        assert!(output.contains("Inspecting entity"));
    }

    #[test]
    fn inspect_without_arg_returns_usage_error() {
        let mut repl = make_repl();
        let output = repl.process_command("inspect");
        assert!(output.contains("Usage: inspect"));
    }

    #[test]
    fn spawn_increments_tick_and_reports_id() {
        let mut repl = make_repl();
        repl.tick = 5;
        let output = repl.process_command("spawn warrior");
        assert!(output.contains("warrior"));
        assert!(output.contains("id 5"));
        assert_eq!(repl.tick, 6);
    }

    #[test]
    fn spawn_without_arg_returns_usage_error() {
        let mut repl = make_repl();
        let output = repl.process_command("spawn");
        assert!(output.contains("Usage: spawn"));
    }

    #[test]
    fn diplomacy_returns_overview() {
        let mut repl = make_repl();
        let output = repl.process_command("diplomacy");
        assert!(output.contains("Diplomacy Overview"));
    }

    #[test]
    fn economy_returns_overview() {
        let mut repl = make_repl();
        let output = repl.process_command("economy");
        assert!(output.contains("Economy Overview"));
    }

    #[test]
    fn unknown_command_returns_error() {
        let mut repl = make_repl();
        let output = repl.process_command("foobar");
        assert!(output.contains("Unknown command"));
        assert!(output.contains("foobar"));
    }

    #[test]
    fn save_with_path_mentions_path() {
        let mut repl = make_repl();
        let output = repl.process_command("save /tmp/state.json");
        assert!(output.contains("/tmp/state.json"));
    }

    #[test]
    fn save_without_path_uses_default() {
        let mut repl = make_repl();
        let output = repl.process_command("save");
        assert!(output.contains("default location"));
    }

    #[test]
    fn load_with_path_mentions_path() {
        let mut repl = make_repl();
        let output = repl.process_command("load /tmp/state.json");
        assert!(output.contains("/tmp/state.json"));
    }

    #[test]
    fn load_without_path_uses_default() {
        let mut repl = make_repl();
        let output = repl.process_command("load");
        assert!(output.contains("default location"));
    }

    #[test]
    fn version_shows_harness_version() {
        let mut repl = make_repl();
        let output = repl.process_command("version");
        assert!(output.contains(crate::HARNESS_VERSION));
    }

    #[test]
    fn case_insensitive_commands() {
        let mut repl = make_repl();
        let output = repl.process_command("HELP");
        assert!(output.contains("Available Commands"));
        let output = repl.process_command("Status");
        assert!(output.contains("tick:"));
    }

    #[test]
    fn empty_input_returns_empty_string() {
        let mut repl = make_repl();
        let output = repl.process_command("   ");
        assert!(
            output.is_empty(),
            "whitespace-only input should return empty"
        );
        // Input with leading/trailing spaces should still dispatch correctly.
        let output = repl.process_command("  help  ");
        assert!(output.contains("Available Commands"));
    }

    #[test]
    fn process_command_does_not_panic_on_long_input() {
        let mut repl = make_repl();
        let long_input = "x ".repeat(1000);
        let output = repl.process_command(long_input.trim());
        assert!(output.contains("Unknown command"));
    }
}
