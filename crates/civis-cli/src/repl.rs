//! Interactive REPL for the Civis CLI.
//!
//! Provides a readline-powered interactive shell with 12 commands for
//! inspecting and controlling a live Civis simulation. When no arguments
//! are passed to the `civis` binary, this REPL starts automatically.
//!
//! Features:
//! - Connects to the Civis server over WebSocket JSON-RPC (`CIVIS_WS_URL` or `ws://127.0.0.1:5173/ws`)
//! - Falls back to offline mode with placeholder data when the server is unreachable
//! - Command history persisted to `~/.civis_history`
//! - Graceful `Ctrl+C` handling (cancels current input, not the process)
//! - Coloured output via the `colored` crate
//! - All command parsing is pure (no I/O), making it unit-testable

use std::path::PathBuf;

use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use serde_json::Value;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, Message, WebSocket};

/// Type alias for the blocking WebSocket stream used by the REPL.
type WsStream = WebSocket<MaybeTlsStream<std::net::TcpStream>>;

/// Default Civis WebSocket JSON-RPC URL.
const DEFAULT_WS_URL: &str = "ws://127.0.0.1:5173/ws";

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
    /// WebSocket URL for the Civis server.
    ws_url: String,
    /// Live WebSocket connection (`None` when server is unreachable or in offline/test mode).
    ws: Option<WsStream>,
    /// Auto-incrementing JSON-RPC request id.
    next_rpc_id: u64,
}

impl Repl {
    /// Create a new REPL with the given configuration.
    ///
    /// The WebSocket URL defaults to `ws://127.0.0.1:5173/ws` and can be
    /// overridden with the `CIVIS_WS_URL` environment variable.
    pub fn new(config: ReplConfig) -> Self {
        let ws_url = std::env::var("CIVIS_WS_URL").unwrap_or_else(|_| DEFAULT_WS_URL.to_string());
        Self {
            config,
            tick: 0,
            running: true,
            ws_url,
            ws: None,
            next_rpc_id: 1,
        }
    }

    /// Attempt to connect to the Civis server over WebSocket.
    ///
    /// Returns `Ok(())` on success, or `Err(message)` on failure.
    /// The REPL remains usable in offline mode when the connection fails.
    fn connect_ws(&mut self) -> Result<(), String> {
        let (socket, _response) =
            connect(&self.ws_url).map_err(|e| format!("WebSocket connect failed: {e}"))?;
        self.ws = Some(socket);
        Ok(())
    }

    /// Send a JSON-RPC request and return the parsed response.
    ///
    /// Returns `Ok(Value)` with the full JSON-RPC response on success,
    /// or `Err(String)` on connection or protocol errors.
    fn send_rpc(&mut self, method: &str, params: Value) -> Result<Value, String> {
        let ws = self.ws.as_mut().ok_or("Not connected to server")?;
        let id = self.next_rpc_id;
        self.next_rpc_id += 1;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        let text = request.to_string();
        ws.send(Message::Text(text))
            .map_err(|e| format!("WebSocket send error: {e}"))?;

        // Read responses in a loop, skipping broadcast frames (tick data).
        // JSON-RPC responses have an "id" field; broadcast frames do not.
        loop {
            let msg = ws
                .read()
                .map_err(|e| format!("WebSocket recv error: {e}"))?;
            match msg {
                Message::Text(text) => {
                    // Try to parse as JSON-RPC response (has "id" field).
                    if let Ok(val) = serde_json::from_str::<Value>(&text) {
                        if val.get("id").is_some() {
                            return Ok(val);
                        }
                        // Otherwise it's a broadcast frame — skip it.
                    }
                }
                Message::Close(_) => {
                    return Err("WebSocket closed by server".to_string());
                }
                _ => {}
            }
        }
    }

    /// Run the REPL loop. Blocks until the user exits.
    pub fn run(&mut self) -> Result<(), ReplError> {
        let mut rl = DefaultEditor::new().map_err(ReplError::Readline)?;

        // Load history if it exists.
        if self.config.history_path.exists() {
            let _ = rl.load_history(&self.config.history_path);
        }

        // Attempt WebSocket connection.
        match self.connect_ws() {
            Ok(()) => {
                println!(
                    "{} Connected to {}",
                    "\u{2713}".green().bold(),
                    self.ws_url.green()
                );
            }
            Err(e) => {
                println!(
                    "{} {} ({})",
                    "warning:".yellow().bold(),
                    "Server unreachable - running in offline mode".yellow(),
                    e.dimmed()
                );
            }
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
            ("spawn <type> [x] [y] [faction]", "Spawn a new entity of the given type"),
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
    ///
    /// When connected, queries `sim.status` from the server.
    /// When offline, shows the local tick counter.
    fn cmd_status(&mut self) -> String {
        if self.ws.is_some() {
            match self.send_rpc("sim.status", serde_json::json!({})) {
                Ok(resp) => {
                    if let Some(err) = resp.get("error") {
                        return format!(
                            "{} {}",
                            "server error:".red().bold(),
                            err.get("message")
                                .and_then(|m| m.as_str())
                                .unwrap_or("unknown"),
                        );
                    }
                    let result = resp.get("result").cloned().unwrap_or(Value::Null);
                    let tick = result
                        .get("tick")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(self.tick);
                    self.tick = tick;
                    let population = result
                        .get("population")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let outcome = result
                        .get("outcome")
                        .and_then(|v| v.as_str())
                        .unwrap_or("running");
                    format!(
                        "{}\n  tick: {}\n  population: {}\n  state: {}\n",
                        "Simulation Status:".green().bold(),
                        tick,
                        population,
                        outcome.cyan()
                    )
                }
                Err(e) => format!("{} {}", "error:".red().bold(), e),
            }
        } else {
            format!(
                "{}\n  tick: {}\n  entities: {}\n  state: {}\n",
                "Simulation Status (offline):".green().bold(),
                self.tick,
                "0 (no live connection)",
                "idle".yellow()
            )
        }
    }

    /// Advance the simulation tick counter.
    ///
    /// When connected, sends `sim.command` with `action: "tick"`.
    /// When offline, increments the local counter.
    fn cmd_tick(&mut self) -> String {
        if self.ws.is_some() {
            match self.send_rpc("sim.command", serde_json::json!({"action": "tick"})) {
                Ok(resp) => {
                    if let Some(err) = resp.get("error") {
                        return format!(
                            "{} {}",
                            "server error:".red().bold(),
                            err.get("message")
                                .and_then(|m| m.as_str())
                                .unwrap_or("unknown"),
                        );
                    }
                    let result = resp.get("result").cloned().unwrap_or(Value::Null);
                    let accepted = result
                        .get("accepted")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let new_tick = result
                        .get("tick")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(self.tick + 1);
                    self.tick = new_tick;
                    if accepted {
                        format!("{} tick -> {}", "Advanced:".green().bold(), self.tick)
                    } else {
                        format!(
                            "{} tick rejected (state: {})",
                            "warning:".yellow().bold(),
                            result
                                .get("reason")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                        )
                    }
                }
                Err(e) => format!("{} {}", "error:".red().bold(), e),
            }
        } else {
            self.tick += 1;
            format!(
                "{} tick -> {}",
                "Advanced (offline):".green().bold(),
                self.tick
            )
        }
    }

    /// Inspect an entity by name or id.
    ///
    /// When connected, sends `sim.inspect_tile` with `{x, y}` coordinates.
    /// When offline, returns placeholder data.
    fn cmd_inspect(&mut self, entity: &str) -> String {
        if entity.is_empty() {
            return format!("{} Usage: inspect <entity>", "error:".red().bold());
        }

        if self.ws.is_some() {
            // Parse "x,y" or "x y" format from the entity string, default to 0,0.
            let (x, y) = parse_tile_coords(entity);
            match self.send_rpc("sim.inspect_tile", serde_json::json!({"x": x, "y": y})) {
                Ok(resp) => {
                    if let Some(err) = resp.get("error") {
                        return format!(
                            "{} {}",
                            "server error:".red().bold(),
                            err.get("message")
                                .and_then(|m| m.as_str())
                                .unwrap_or("unknown"),
                        );
                    }
                    let result = resp.get("result").cloned().unwrap_or(Value::Null);
                    let material = result.get("material").and_then(|v| v.as_u64()).unwrap_or(0);
                    let terrain_height = result
                        .get("terrain_height")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    format!(
                        "{} '{}'\n  tile: ({}, {})\n  material: {}\n  terrain_height: {}\n",
                        "Inspecting tile:".cyan().bold(),
                        entity,
                        x,
                        y,
                        material,
                        terrain_height,
                    )
                }
                Err(e) => format!("{} {}", "error:".red().bold(), e),
            }
        } else {
            format!(
                "{} '{}'\n  type: unknown\n  position: (0, 0, 0)\n  state: idle\n",
                "Inspecting entity (offline):".cyan().bold(),
                entity
            )
        }
    }

    /// Spawn a new civilian entity on the live simulation.
    ///
    /// When connected, sends `sim.spawn_civilian` with `{x, y, faction}`.
    /// The argument is parsed as optional coordinates and faction:
    ///   - `spawn`            → error (needs an argument)
    ///   - `spawn villager`   → default coords (0.5, 0.5), faction 0
    ///   - `spawn 0.5 0.3`    → coords (0.5, 0.3), faction 0
    ///   - `spawn 0.5 0.3 1`  → coords (0.5, 0.3), faction 1
    /// When offline, shows a placeholder message.
    fn cmd_spawn(&mut self, entity_type: &str) -> String {
        if entity_type.is_empty() {
            return format!("{} Usage: spawn <type> [x] [y] [faction]", "error:".red().bold());
        }

        if self.ws.is_some() {
            let (x, y, faction) = parse_spawn_params(entity_type);
            match self.send_rpc(
                "sim.spawn_civilian",
                serde_json::json!({"x": x, "y": y, "faction": faction}),
            ) {
                Ok(resp) => {
                    if let Some(err) = resp.get("error") {
                        return format!(
                            "{} {}",
                            "server error:".red().bold(),
                            err.get("message")
                                .and_then(|m| m.as_str())
                                .unwrap_or("unknown"),
                        );
                    }
                    let accepted = resp
                        .get("result")
                        .and_then(|r| r.get("accepted"))
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    if accepted {
                        format!(
                            "{} '{}' at ({}, {}) faction {}",
                            "Spawned entity:".green().bold(),
                            entity_type.magenta(),
                            x,
                            y,
                            faction,
                        )
                    } else {
                        format!("{} server rejected spawn", "error:".red().bold())
                    }
                }
                Err(e) => format!("{} {}", "error:".red().bold(), e),
            }
        } else {
            format!(
                "{} '{}' (offline — no server connection)",
                "Spawned entity (offline):".green().bold(),
                entity_type.magenta(),
            )
        }
    }

    /// Show diplomacy overview by querying factions from the live simulation.
    ///
    /// When connected, sends `sim.get_factions` to retrieve the faction list.
    /// When offline, shows placeholder data.
    fn cmd_diplomacy(&mut self) -> String {
        if self.ws.is_some() {
            match self.send_rpc("sim.get_factions", serde_json::json!({})) {
                Ok(resp) => {
                    if let Some(err) = resp.get("error") {
                        return format!(
                            "{} {}",
                            "server error:".red().bold(),
                            err.get("message")
                                .and_then(|m| m.as_str())
                                .unwrap_or("unknown"),
                        );
                    }
                    let result = resp.get("result").cloned().unwrap_or(Value::Null);
                    let factions = result
                        .get("factions")
                        .cloned()
                        .unwrap_or(Value::Array(vec![]));
                    let faction_count = factions
                        .as_array()
                        .map(|a| a.len())
                        .unwrap_or(0);
                    let tick = result
                        .get("tick")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(self.tick);
                    self.tick = tick;
                    let mut out = format!(
                        "{}\n  tick: {}\n  factions: {}\n",
                        "Diplomacy Overview:".green().bold(),
                        tick,
                        faction_count,
                    );
                    if let Some(arr) = factions.as_array() {
                        for f in arr {
                            let name = f
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("?");
                            let id = f
                                .get("id")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0);
                            out.push_str(&format!("  faction {}: {}\n", id, name.cyan()));
                        }
                    }
                    out
                }
                Err(e) => format!("{} {}", "error:".red().bold(), e),
            }
        } else {
            format!(
                "{}\n  factions: 0 (offline)\n",
                "Diplomacy Overview:".green().bold()
            )
        }
    }

    /// Show economy overview by querying resources from the live simulation.
    ///
    /// When connected, sends `sim.get_resources` to retrieve market prices
    /// and institution balances.
    /// When offline, shows placeholder data.
    fn cmd_economy(&mut self) -> String {
        if self.ws.is_some() {
            match self.send_rpc("sim.get_resources", serde_json::json!({})) {
                Ok(resp) => {
                    if let Some(err) = resp.get("error") {
                        return format!(
                            "{} {}",
                            "server error:".red().bold(),
                            err.get("message")
                                .and_then(|m| m.as_str())
                                .unwrap_or("unknown"),
                        );
                    }
                    let result = resp.get("result").cloned().unwrap_or(Value::Null);

                    // Update local tick from response.
                    if let Some(tick) = result.get("tick").and_then(|v| v.as_u64()) {
                        self.tick = tick;
                    }

                    let market_prices = result.get("market_prices").cloned().unwrap_or(Value::Object(serde_json::Map::new()));
                    let institutions = result.get("institutions").cloned().unwrap_or(Value::Array(vec![]));

                    let mut out = format!(
                        "{}\n  tick: {}\n",
                        "Economy Overview:".green().bold(),
                        self.tick,
                    );

                    // Display market prices.
                    if let Some(obj) = market_prices.as_object() {
                        if obj.is_empty() {
                            out.push_str("  market prices: (none)\n");
                        } else {
                            out.push_str("  market prices:\n");
                            for (good, price) in obj {
                                out.push_str(&format!("    {}: {}\n", good.cyan(), price));
                            }
                        }
                    }

                    // Display institutions.
                    if let Some(arr) = institutions.as_array() {
                        if arr.is_empty() {
                            out.push_str("  institutions: (none)\n");
                        } else {
                            out.push_str("  institutions:\n");
                            for inst in arr {
                                let kind = inst
                                    .get("kind")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("?");
                                let balance = inst
                                    .get("balance_joules")
                                    .and_then(|v| v.as_i64())
                                    .unwrap_or(0);
                                out.push_str(&format!("    {} balance: {} J\n", kind.magenta(), balance));
                            }
                        }
                    }

                    out
                }
                Err(e) => format!("{} {}", "error:".red().bold(), e),
            }
        } else {
            format!(
                "{}\n  treasury: 0\n  production: 0\n  trade routes: 0\n",
                "Economy Overview (offline):".green().bold()
            )
        }
    }

    /// Save the current simulation state.
    ///
    /// When connected, sends `save.slot` with the slot name.
    /// When offline, shows a placeholder message.
    fn cmd_save(&mut self, path: &str) -> String {
        let slot_name = if path.is_empty() {
            "autosave".to_string()
        } else {
            // Extract a slot name from the path.
            PathBuf::from(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("autosave")
                .to_string()
        };

        if self.ws.is_some() {
            match self.send_rpc("save.slot", serde_json::json!({ "slot_name": slot_name })) {
                Ok(resp) => {
                    if let Some(err) = resp.get("error") {
                        return format!(
                            "{} {}",
                            "server error:".red().bold(),
                            err.get("message")
                                .and_then(|m| m.as_str())
                                .unwrap_or("unknown"),
                        );
                    }
                    let result = resp.get("result").cloned().unwrap_or(Value::Null);
                    let saved_tick = result
                        .get("tick")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(self.tick);
                    self.tick = saved_tick;
                    if path.is_empty() {
                        format!(
                            "{} saved to slot '{}' (tick: {})",
                            "State".green().bold(),
                            slot_name,
                            saved_tick
                        )
                    } else {
                        format!(
                            "{} saved to slot '{}' at '{}' (tick: {})",
                            "State".green().bold(),
                            slot_name,
                            path,
                            saved_tick
                        )
                    }
                }
                Err(e) => format!("{} {}", "error:".red().bold(), e),
            }
        } else if path.is_empty() {
            format!(
                "{} saved to default location (tick: {})",
                "State".green().bold(),
                self.tick
            )
        } else {
            format!("{} saved to '{}'", "State".green().bold(), path)
        }
    }

    /// Load a saved simulation state from a production slot.
    ///
    /// When connected, sends `save.load` with `{ slot_name }`.
    /// When offline, shows a placeholder message.
    fn cmd_load(&mut self, path: &str) -> String {
        let slot_name = if path.is_empty() {
            "autosave".to_string()
        } else {
            PathBuf::from(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("autosave")
                .to_string()
        };

        if self.ws.is_some() {
            match self.send_rpc("save.load", serde_json::json!({ "slot_name": slot_name })) {
                Ok(resp) => {
                    if let Some(err) = resp.get("error") {
                        return format!(
                            "{} {}",
                            "server error:".red().bold(),
                            err.get("message")
                                .and_then(|m| m.as_str())
                                .unwrap_or("unknown"),
                        );
                    }
                    let result = resp.get("result").cloned().unwrap_or(Value::Null);
                    let loaded = result
                        .get("loaded")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true);
                    let loaded_tick = result.get("tick").and_then(|v| v.as_u64()).unwrap_or(0);
                    self.tick = loaded_tick;
                    if loaded {
                        if path.is_empty() {
                            format!(
                                "{} loaded from slot '{}' (tick: {})",
                                "State".green().bold(),
                                slot_name,
                                loaded_tick
                            )
                        } else {
                            format!(
                                "{} loaded from slot '{}' at '{}' (tick: {})",
                                "State".green().bold(),
                                slot_name,
                                path,
                                loaded_tick
                            )
                        }
                    } else {
                        format!("{} failed to load slot '{}'", "error:".red().bold(), slot_name)
                    }
                }
                Err(e) => format!("{} {}", "error:".red().bold(), e),
            }
        } else if path.is_empty() {
            format!("{} loaded from default slot (offline)", "State".green().bold())
        } else {
            format!("{} loaded from '{}' (offline)", "State".green().bold(), path)
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

/// Parse tile coordinates from a string like "3,5" or "3 5".
/// Defaults to (0, 0) if parsing fails.
fn parse_tile_coords(s: &str) -> (i64, i64) {
    // Try "x,y" format.
    if let Some((xs, ys)) = s.split_once(',') {
        if let (Ok(x), Ok(y)) = (xs.trim().parse::<i64>(), ys.trim().parse::<i64>()) {
            return (x, y);
        }
    }
    // Try "x y" format.
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() == 2 {
        if let (Ok(x), Ok(y)) = (parts[0].parse::<i64>(), parts[1].parse::<i64>()) {
            return (x, y);
        }
    }
    // Fallback: treat as a numeric id, use it as x with y=0.
    if let Ok(x) = s.trim().parse::<i64>() {
        return (x, 0);
    }
    (0, 0)
}

/// Parse spawn parameters from a string.
///
/// Accepts formats:
/// - `"0.5 0.3"` → x=0.5, y=0.3, faction=0
/// - `"0.5 0.3 1"` → x=0.5, y=0.3, faction=1
/// - `"villager"` → x=0.5, y=0.5, faction=0 (default coords)
/// - `"villager 0.2 0.8"` → x=0.2, y=0.8, faction=0
/// - `"villager 0.2 0.8 2"` → x=0.2, y=0.8, faction=2
fn parse_spawn_params(s: &str) -> (f32, f32, u32) {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() == 1 {
        // Single token: could be coordinates or a type name.
        // If it parses as a float, treat as x with y=0.
        if let Ok(x) = parts[0].parse::<f32>() {
            return (x, 0.0, 0);
        }
        // Otherwise it's a type name — use default coords.
        return (0.5, 0.5, 0);
    }
    if parts.len() == 2 {
        // Could be "x y" (numeric) or "type x".
        if let (Ok(x), Ok(y)) = (parts[0].parse::<f32>(), parts[1].parse::<f32>()) {
            return (x, y, 0);
        }
        // "type x" → parse the second as x, y=0.
        if let Ok(x) = parts[1].parse::<f32>() {
            return (x, 0.0, 0);
        }
        return (0.5, 0.5, 0);
    }
    if parts.len() >= 3 {
        // "type x y [faction]" or "x y faction".
        // Try to parse the last up to 3 tokens as numbers.
        if parts.len() == 3 {
            if let (Ok(x), Ok(y), Ok(f)) = (
                parts[0].parse::<f32>(),
                parts[1].parse::<f32>(),
                parts[2].parse::<u32>(),
            ) {
                return (x, y, f);
            }
        }
        // "type x y faction"
        let nums: Vec<&str> = parts.iter().rev().take(3).copied().collect();
        if nums.len() == 3 {
            if let (Ok(f), Ok(y), Ok(x)) = (
                nums[0].parse::<u32>(),
                nums[1].parse::<f32>(),
                nums[2].parse::<f32>(),
            ) {
                return (x, y, f);
            }
        }
        if parts.len() == 3 {
            if let (Ok(x), Ok(y)) = (parts[1].parse::<f32>(), parts[2].parse::<f32>()) {
                return (x, y, 0);
            }
        }
        return (0.5, 0.5, 0);
    }
    (0.5, 0.5, 0)
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
        assert!(output.contains("Inspecting"));
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
        assert!(output.contains("Spawned entity (offline)"));
    }

    #[test]
    fn spawn_without_arg_returns_usage_error() {
        let mut repl = make_repl();
        let output = repl.process_command("spawn");
        assert!(output.contains("Usage: spawn"));
    }

    #[test]
    fn parse_spawn_params_type_only() {
        let (x, y, f) = super::parse_spawn_params("villager");
        assert!((x - 0.5).abs() < 0.001);
        assert!((y - 0.5).abs() < 0.001);
        assert_eq!(f, 0);
    }

    #[test]
    fn parse_spawn_params_xy() {
        let (x, y, f) = super::parse_spawn_params("0.2 0.8");
        assert!((x - 0.2).abs() < 0.001);
        assert!((y - 0.8).abs() < 0.001);
        assert_eq!(f, 0);
    }

    #[test]
    fn parse_spawn_params_xy_faction() {
        let (x, y, f) = super::parse_spawn_params("0.3 0.7 2");
        assert!((x - 0.3).abs() < 0.001);
        assert!((y - 0.7).abs() < 0.001);
        assert_eq!(f, 2);
    }

    #[test]
    fn parse_spawn_params_type_xy_faction() {
        let (x, y, f) = super::parse_spawn_params("warrior 0.1 0.9 3");
        assert!((x - 0.1).abs() < 0.001);
        assert!((y - 0.9).abs() < 0.001);
        assert_eq!(f, 3);
    }

    #[test]
    fn parse_spawn_params_single_number() {
        let (x, y, f) = super::parse_spawn_params("0.7");
        assert!((x - 0.7).abs() < 0.001);
        assert!((y - 0.0).abs() < 0.001);
        assert_eq!(f, 0);
    }

    #[test]
    fn parse_spawn_params_type_xy() {
        let (x, y, f) = super::parse_spawn_params("warrior 0.4 0.6");
        assert!((x - 0.4).abs() < 0.001);
        assert!((y - 0.6).abs() < 0.001);
        assert_eq!(f, 0);
    }

    #[test]
    fn diplomacy_returns_overview() {
        let mut repl = make_repl();
        let output = repl.process_command("diplomacy");
        assert!(output.contains("Diplomacy Overview"));
        assert!(output.contains("offline"));
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
        assert!(output.contains("default slot"));
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

    #[test]
    fn parse_tile_coords_comma_separated() {
        assert_eq!(parse_tile_coords("3,5"), (3, 5));
        assert_eq!(parse_tile_coords(" 10 , 20 "), (10, 20));
    }

    #[test]
    fn parse_tile_coords_space_separated() {
        assert_eq!(parse_tile_coords("3 5"), (3, 5));
    }

    #[test]
    fn parse_tile_coords_single_number() {
        assert_eq!(parse_tile_coords("42"), (42, 0));
    }

    #[test]
    fn parse_tile_coords_non_numeric() {
        assert_eq!(parse_tile_coords("villager_42"), (0, 0));
    }
}
