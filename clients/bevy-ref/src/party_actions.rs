#![cfg(all(feature = "bevy", feature = "egui"))]

//! Party Actions multiplayer panel — productizes the v0.4.0 multiplayer state
//! sync foundation (`crates/server/src/ws_bridge.rs` +
//! `crates/server/src/session.rs`) into player-facing controls (Phase 6.3).
//!
//! Three surfaces in a single right-edge overlay:
//!
//! 1. **Connected players** — list of currently-known sessions with their
//!    `connection_id`, optional display label, and last-acked tick. The local
//!    session is marked with an accent dot. Populated client-side from
//!    `sim.get_snapshot_for_session` responses (local) and from outbound
//!    attribution events (peer sessions we have observed through the wire).
//!
//! 2. **Action attribution log** — newest-first ring buffer of every god
//!    action / diplomacy action / ready signal the local client has issued,
//!    attributed to the `connection_id` that issued it. Mirrors the
//!    server-side `Simulation::record_god_action` audit log but from the
//!    player perspective (so you can see what *your* client just sent).
//!
//! 3. **Multiplayer-safe action buttons** — every button sends a JSON-RPC
//!    method already wired into the multiplayer bridge:
//!
//!    | Button              | JSON-RPC method                                                  |
//!    |---------------------|------------------------------------------------------------------|
//!    | Propose Trade       | `sim.diplomacy_action { action: "offer_trade", ... }`            |
//!    | Send Message        | `sim.god_action   { action: "chat.send", message: ... }`         |
//!    | Ready / Unready     | `sim.command      { action: "ready" | "unready" }`               |
//!
//!    All three reuse the [`ServerBridge`](crate::live_stream::ServerBridge)
//!    resource pattern (`faction_hud.rs`, `replay_controls.rs`,
//!    `notifications.rs`) so panels can fire JSON-RPC without depending on
//!    the binary-crate `LiveBridge` type.
//!
//! Toggle with `Shift+P`. The panel defaults to open so the v0.4.0 state
//! sync foundation is immediately observable on first launch.

use std::collections::VecDeque;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::faction_hud::PlayerFactionId;
use crate::live_stream::ServerBridge;
use crate::menus::{in_playing_state, GameUiMode};
use crate::ui_theme::{ACCENT, DIM, GOLD, GREEN, PANEL_FILL, RED};

/// Maximum actions retained in the attribution log ring buffer.
///
/// Sized to match `civ_server::session::SESSION_HISTORY_CAP` (32) so the
/// client-side log and the server-side audit log have the same horizon.
pub const ACTION_LOG_CAP: usize = 32;

/// Maximum player entries shown in the connected players list.
///
/// Mirrors `WsBridgeConfig::max_clients` (default 16) so the client cap is
/// never above the server cap.
pub const PLAYER_LIST_CAP: usize = 16;

/// Default action log ring buffer cap (alias for tests).
pub const ACTION_LOG_CAP_DEFAULT: usize = ACTION_LOG_CAP;

// ── Data model ───────────────────────────────────────────────────────────────

/// One row in the action attribution log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionLogEntry {
    /// Stable `connection_id` of the player who issued the action.
    pub connection_id: String,
    /// Tick at which the action was issued (server-recorded; 0 = local pre-tick).
    pub tick: u64,
    /// Action verb (`offer_trade`, `chat.send`, `ready`, `sim.god_action`, …).
    pub action: String,
    /// Optional human-readable detail (message text, target faction, etc.).
    pub detail: String,
}

/// One row in the connected players list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerEntry {
    /// Stable connection id (UUID v4 hex string).
    pub connection_id: String,
    /// Optional display label (e.g. faction name from `PlayerFactionId`).
    pub label: Option<String>,
    /// Latest tick the player has acknowledged.
    pub last_acked_tick: u64,
    /// Whether the player has signalled ready for the next tick advance.
    pub ready: bool,
    /// Whether this entry is the local session.
    pub is_local: bool,
}

// ── Resource ────────────────────────────────────────────────────────────────

/// Multiplayer state resource for the party actions panel.
///
/// Mutated by:
/// - `PartyActionsState::register_session` when the local session is known
///   (via the JSON-RPC `sim.get_snapshot_for_session` response shape), or
///   when a peer session is observed through outbound attribution.
/// - `PartyActionsState::record_action` from the panel button handlers.
/// - `PartyActionsState::mark_ready` / `mark_unready` / `toggle_ready`.
#[derive(Resource, Debug, Default)]
pub struct PartyActionsState {
    /// Most recent `connection_id` returned by `sim.get_snapshot_for_session`.
    pub local_session_id: Option<String>,
    /// Last-acked tick for the local session, as reported by the server.
    pub local_last_acked_tick: u64,
    /// Connected players observed client-side (ourselves + any peers we have
    /// attributed actions to).
    pub players: Vec<PlayerEntry>,
    /// Newest-first attribution log (most recent action at `front()`).
    pub action_log: VecDeque<ActionLogEntry>,
    /// Whether the local player is ready for the next tick advance.
    pub ready_for_tick: bool,
    /// Tick at which `ready_for_tick` became true (0 = not ready).
    pub ready_since_tick: u64,
    /// Optional faction id used when issuing diplomacy actions (propose trade).
    pub local_faction: Option<u32>,
}

impl PartyActionsState {
    /// Register or update a player entry, bumping its `last_acked_tick`.
    ///
    /// Idempotent: re-registering the same `connection_id` updates the
    /// existing row in place (and merges the label) rather than appending a
    /// duplicate. New entries that would exceed [`PLAYER_LIST_CAP`] evict
    /// the oldest non-local entry first; if all entries are local we still
    /// evict the oldest so the cap holds.
    pub fn register_session(
        &mut self,
        connection_id: &str,
        label: Option<String>,
        tick: u64,
        is_local: bool,
    ) {
        if is_local {
            self.local_session_id = Some(connection_id.to_owned());
            if tick > self.local_last_acked_tick {
                self.local_last_acked_tick = tick;
            }
        }
        if let Some(existing) = self.players.iter_mut().find(|p| p.connection_id == connection_id) {
            if tick > existing.last_acked_tick {
                existing.last_acked_tick = tick;
            }
            if label.is_some() {
                existing.label = label;
            }
            if is_local {
                existing.is_local = true;
            }
            return;
        }
        if self.players.len() >= PLAYER_LIST_CAP {
            // Prefer evicting a non-local entry; fall back to the oldest.
            let evict_idx = self
                .players
                .iter()
                .position(|p| !p.is_local)
                .unwrap_or(0);
            self.players.remove(evict_idx);
        }
        self.players.push(PlayerEntry {
            connection_id: connection_id.to_owned(),
            label,
            last_acked_tick: tick,
            ready: false,
            is_local,
        });
    }

    /// Record an outgoing action for the attribution log.
    ///
    /// Pushes to the front of the ring buffer; drops the oldest entry when
    /// the cap is exceeded. Returns the inserted entry for convenience.
    pub fn record_action(
        &mut self,
        connection_id: &str,
        action: impl Into<String>,
        detail: impl Into<String>,
        tick: u64,
    ) -> ActionLogEntry {
        if self.action_log.len() >= ACTION_LOG_CAP {
            self.action_log.pop_back();
        }
        let entry = ActionLogEntry {
            connection_id: connection_id.to_owned(),
            tick,
            action: action.into(),
            detail: detail.into(),
        };
        self.action_log.push_front(entry.clone());
        entry
    }

    /// Toggle the local player's ready state, stamping `ready_since_tick`
    /// when transitioning to ready. Returns the new value.
    pub fn toggle_ready(&mut self, current_tick: u64) -> bool {
        self.ready_for_tick = !self.ready_for_tick;
        self.ready_since_tick = if self.ready_for_tick {
            current_tick
        } else {
            0
        };
        self.ready_for_tick
    }

    /// Mark the local player ready at `current_tick`. Idempotent.
    pub fn mark_ready(&mut self, current_tick: u64) {
        self.ready_for_tick = true;
        if self.ready_since_tick == 0 {
            self.ready_since_tick = current_tick;
        }
    }

    /// Mark the local player unready.
    pub fn mark_unready(&mut self) {
        self.ready_for_tick = false;
        self.ready_since_tick = 0;
    }

    /// Number of ready players in the player list (for HUD readouts).
    #[must_use]
    pub fn ready_count(&self) -> usize {
        self.players.iter().filter(|p| p.ready).count()
    }

    /// Clear all panel state (used on disconnect / world reset).
    pub fn clear(&mut self) {
        self.players.clear();
        self.action_log.clear();
        self.local_session_id = None;
        self.local_last_acked_tick = 0;
        self.ready_for_tick = false;
        self.ready_since_tick = 0;
        self.local_faction = None;
    }
}

// ── Plugin / open-state resource ────────────────────────────────────────────

/// HUD open/closed toggle state. Defaults to open so the multiplayer panel
/// is immediately observable on first launch.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartyActionsOpen(pub bool);

impl Default for PartyActionsOpen {
    fn default() -> Self {
        Self(true)
    }
}

/// Plugin that wires the multiplayer party actions panel + state.
pub struct PartyActionsPlugin;

impl Plugin for PartyActionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PartyActionsState>()
            .init_resource::<PartyActionsOpen>()
            .init_resource::<PartyActionsDraft>()
            .add_systems(Update, toggle_party_actions_panel)
            .add_systems(
                EguiPrimaryContextPass,
                draw_party_actions_panel.run_if(in_playing_state),
            );
    }
}

/// Draft chat message buffer shared between the egui textbox and the Send
/// button so we don't lose keystrokes between frames.
#[derive(Resource, Debug, Default)]
pub struct PartyActionsDraft {
    /// Outgoing chat text.
    pub message: String,
}

// ── Systems ─────────────────────────────────────────────────────────────────

/// Toggle the panel with `Shift+P`. Stays in sync with `notifications.rs`'s
/// in-Playing run_if so the keypress only fires during gameplay.
fn toggle_party_actions_panel(
    keys: Res<ButtonInput<KeyCode>>,
    mode: Res<GameUiMode>,
    mut open: ResMut<PartyActionsOpen>,
) {
    if *mode != GameUiMode::Playing {
        return;
    }
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if shift && keys.just_pressed(KeyCode::KeyP) {
        open.0 = !open.0;
    }
}

fn draw_party_actions_panel(
    mut contexts: EguiContexts,
    open: Res<PartyActionsOpen>,
    mut state: ResMut<PartyActionsState>,
    mut draft: ResMut<PartyActionsDraft>,
    bridge: Option<Res<ServerBridge>>,
    player_faction: Res<PlayerFactionId>,
) {
    if !open.0 {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    // Mirror the local faction id for outbound diplomacy buttons.
    if state.local_faction.is_none() {
        state.local_faction = Some(player_faction.0);
    }

    egui::Window::new("Party Actions")
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-8.0, 8.0))
        .resizable(true)
        .collapsible(true)
        .title_bar(true)
        .default_width(320.0)
        .min_width(280.0)
        .max_width(420.0)
        .frame(
            egui::Frame::NONE
                .fill(PANEL_FILL)
                .inner_margin(egui::Margin::same(12))
                .corner_radius(egui::CornerRadius::same(10)),
        )
        .show(ctx, |ui| {
            ui.set_min_width(260.0);

            ui.label(
                egui::RichText::new(format!(
                    "Local session: {}",
                    state
                        .local_session_id
                        .as_deref()
                        .unwrap_or("(connecting…)")
                ))
                .color(ACCENT)
                .small(),
            );
            ui.label(
                egui::RichText::new(format!(
                    "Last-acked tick: {} • Ready: {}/{}",
                    state.local_last_acked_tick,
                    state.ready_count(),
                    state.players.len(),
                ))
                .color(DIM)
                .small(),
            );

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(4.0);

            draw_connected_players(ui, &state);
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            draw_action_log(ui, &state);
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            draw_action_buttons(ui, &mut state, &mut draft, bridge.as_deref());

            ui.add_space(6.0);
            ui.label(
                egui::RichText::new("[Shift+P] to hide")
                    .color(DIM)
                    .small()
                    .italics(),
            );
        });
}

fn draw_connected_players(ui: &mut egui::Ui, state: &PartyActionsState) {
    ui.label(egui::RichText::new("Connected players").strong());
    if state.players.is_empty() {
        ui.label(
            egui::RichText::new("  (no peer sessions observed yet)")
                .color(DIM)
                .italics()
                .small(),
        );
        return;
    }
    for player in &state.players {
        let marker = if player.is_local { "●" } else { "○" };
        let color = if player.is_local { ACCENT } else { DIM };
        let label = player.label.as_deref().unwrap_or("(anonymous)");
        let ready_marker = if player.ready { "[R]" } else { "[ ]" };
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(marker).color(color).strong());
            ui.label(
                egui::RichText::new(format!(
                    "{} {} tick={}",
                    ready_marker,
                    label,
                    player.last_acked_tick,
                ))
                .color(if player.is_local { ACCENT } else { DIM })
                .small(),
            );
        });
        ui.label(
            egui::RichText::new(format!("  id: {}", short_id(&player.connection_id)))
                .color(DIM)
                .small(),
        );
    }
}

fn draw_action_log(ui: &mut egui::Ui, state: &PartyActionsState) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Action attribution").strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!("{} entries", state.action_log.len()))
                    .color(DIM)
                    .small(),
            );
        });
    });
    if state.action_log.is_empty() {
        ui.label(
            egui::RichText::new("  (no actions attributed yet)")
                .color(DIM)
                .italics()
                .small(),
        );
        return;
    }
    egui::ScrollArea::vertical()
        .max_height(140.0)
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for entry in state.action_log.iter() {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("t={:>5}", entry.tick))
                            .color(GOLD)
                            .small(),
                    );
                    ui.label(
                        egui::RichText::new(format!("{} → {}", short_id(&entry.connection_id), entry.action))
                            .color(GREEN)
                            .small(),
                    );
                });
                if !entry.detail.is_empty() {
                    ui.label(
                        egui::RichText::new(format!("    {}", entry.detail))
                            .color(DIM)
                            .small(),
                    );
                }
            }
        });
}

fn draw_action_buttons(
    ui: &mut egui::Ui,
    state: &mut PartyActionsState,
    draft: &mut PartyActionsDraft,
    bridge: Option<&ServerBridge>,
) {
    ui.label(egui::RichText::new("Multiplayer actions").strong());

    let local_id = state
        .local_session_id
        .clone()
        .unwrap_or_else(|| "local".to_string());
    let faction = state.local_faction.unwrap_or(0);
    let last_tick = state.local_last_acked_tick;

    // Propose Trade → sim.diplomacy_action { action: offer_trade, ... }
    let trade_btn = egui::Button::new(
        egui::RichText::new("Propose Trade")
            .color(egui::Color32::from_rgb(9, 10, 12))
            .strong()
            .size(12.0),
    )
    .fill(ACCENT);
    if ui.add_sized([ui.available_width(), 24.0], trade_btn).clicked() {
        if let Some(bridge) = bridge {
            bridge.send_rpc(
                "sim.diplomacy_action",
                serde_json::json!({
                    "action": "offer_trade",
                    "source_faction": faction,
                    "target_faction": faction.wrapping_add(1),
                }),
            );
        }
        state.record_action(
            &local_id,
            "sim.diplomacy_action",
            format!("offer_trade f{} → f{}", faction, faction.wrapping_add(1)),
            last_tick,
        );
    }

    ui.add_space(4.0);

    // Send Message: textbox + button. Empty text disables the button.
    ui.horizontal(|ui| {
        let resp = ui.add_sized(
            [ui.available_width() - 80.0, 22.0],
            egui::TextEdit::singleline(&mut draft.message),
        );
        let has_text = !draft.message.trim().is_empty();
        let send_btn = egui::Button::new(
            egui::RichText::new("Send")
                .color(if has_text {
                    egui::Color32::from_rgb(9, 10, 12)
                } else {
                    DIM
                })
                .strong()
                .size(11.0),
        )
        .fill(if has_text { GOLD } else { DIM.gamma_multiply(0.4) });
        let clicked = ui.add_sized([70.0, 22.0], send_btn).clicked()
            || (resp.lost_focus()
                && ui.input(|i| i.key_pressed(egui::Key::Enter))
                && has_text);
        if clicked && has_text {
            let text = draft.message.trim().to_string();
            if let Some(bridge) = bridge {
                bridge.send_rpc(
                    "sim.god_action",
                    serde_json::json!({
                        "action": "chat.send",
                        "message": text,
                    }),
                );
            }
            state.record_action(
                &local_id,
                "sim.god_action",
                format!("chat.send: {}", truncate(&text, 48)),
                last_tick,
            );
            draft.message.clear();
        }
    });

    ui.add_space(4.0);

    // Ready / Unready for tick advance.
    let (label, fill, color) = if state.ready_for_tick {
        ("Unready (currently READY)", RED.gamma_multiply(0.7), RED)
    } else {
        ("Ready for next tick", GREEN.gamma_multiply(0.7), GREEN)
    };
    let ready_btn = egui::Button::new(
        egui::RichText::new(label)
            .color(color)
            .strong()
            .size(12.0),
    )
    .fill(fill);
    if ui.add_sized([ui.available_width(), 24.0], ready_btn).clicked() {
        let new_state = state.toggle_ready(last_tick);
        if let Some(bridge) = bridge {
            let action = if new_state { "ready" } else { "unready" };
            bridge.send_rpc(
                "sim.command",
                serde_json::json!({ "action": action }),
            );
        }
        state.record_action(
            &local_id,
            if new_state { "ready" } else { "unready" },
            format!("since tick {}", state.ready_since_tick),
            last_tick,
        );
    }

    ui.add_space(4.0);
    ui.label(
        egui::RichText::new("Sends: sim.diplomacy_action / sim.god_action / sim.command")
            .color(DIM)
            .small()
            .italics(),
    );
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Short, human-readable id form: first 8 chars of the UUID hex + ellipsis.
fn short_id(connection_id: &str) -> String {
    if connection_id.len() <= 8 {
        connection_id.to_string()
    } else {
        format!("{}…", &connection_id[..8])
    }
}

/// Truncate a string for the attribution log detail line.
fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        let mut out: String = text.chars().take(max_chars).collect();
        out.push('…');
        out
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_session_updates_player_list_with_last_acked_tick() {
        let mut state = PartyActionsState::default();

        // Local session comes up first.
        state.register_session("conn-local", Some("Ardani".to_string()), 5, true);
        assert_eq!(state.local_session_id.as_deref(), Some("conn-local"));
        assert_eq!(state.local_last_acked_tick, 5);
        assert_eq!(state.players.len(), 1);
        assert!(state.players[0].is_local);
        assert_eq!(state.players[0].label.as_deref(), Some("Ardani"));
        assert_eq!(state.players[0].last_acked_tick, 5);

        // Re-registering the same id is idempotent and monotonic.
        state.register_session("conn-local", None, 3, true);
        assert_eq!(
            state.players.len(),
            1,
            "duplicate register_session must not append"
        );
        assert_eq!(
            state.players[0].last_acked_tick, 5,
            "stale ticks must not regress last_acked_tick"
        );

        // A new peer session appears.
        state.register_session("conn-peer-1", Some("Velthari".to_string()), 7, false);
        assert_eq!(state.players.len(), 2);
        assert_eq!(state.players[1].connection_id, "conn-peer-1");
        assert!(!state.players[1].is_local);
        assert_eq!(state.players[1].last_acked_tick, 7);

        // Updating the peer bumps its tick without growing the list.
        state.register_session("conn-peer-1", None, 9, false);
        assert_eq!(state.players.len(), 2);
        assert_eq!(state.players[1].last_acked_tick, 9);
        assert_eq!(
            state.players[1].label.as_deref(),
            Some("Velthari"),
            "label should be retained when the second registration passes None",
        );
    }

    #[test]
    fn record_outgoing_action_attributes_to_session() {
        let mut state = PartyActionsState::default();
        state.register_session("conn-local", Some("Ardani".to_string()), 10, true);

        let entry = state.record_action(
            "conn-local",
            "sim.diplomacy_action",
            "offer_trade f0 → f1",
            10,
        );
        assert_eq!(entry.connection_id, "conn-local");
        assert_eq!(entry.action, "sim.diplomacy_action");
        assert_eq!(entry.detail, "offer_trade f0 → f1");
        assert_eq!(entry.tick, 10);

        // Newest-first ordering.
        let front = state.action_log.front().expect("front entry");
        assert_eq!(front.action, "sim.diplomacy_action");

        state.record_action("conn-local", "ready", "since tick 12", 12);
        assert_eq!(state.action_log.len(), 2);
        let front = state.action_log.front().expect("front entry");
        assert_eq!(front.action, "ready");
        assert_eq!(front.tick, 12);

        // Overflow evicts the oldest entry, keeping cap invariant.
        for i in 0..(ACTION_LOG_CAP + 4) {
            state.record_action(
                "conn-local",
                "noop",
                format!("filler {i}"),
                100 + i as u64,
            );
        }
        assert_eq!(
            state.action_log.len(),
            ACTION_LOG_CAP,
            "ring buffer must respect ACTION_LOG_CAP"
        );
        // Newest-first: the most recent filler is at the front.
        let front = state.action_log.front().expect("front entry");
        assert_eq!(front.action, "noop");
        assert_eq!(front.tick, 100 + (ACTION_LOG_CAP + 3) as u64);
    }

    #[test]
    fn ready_toggle_flips_state_with_tick_coordination() {
        let mut state = PartyActionsState::default();
        assert!(!state.ready_for_tick, "ready_for_tick defaults to false");
        assert_eq!(state.ready_since_tick, 0);

        // Toggle on at tick 42 → ready_since_tick stamped, value flips.
        let new_value = state.toggle_ready(42);
        assert!(new_value, "toggle_ready returns the new value (true)");
        assert!(state.ready_for_tick);
        assert_eq!(
            state.ready_since_tick, 42,
            "ready_since_tick must record the tick at which we became ready"
        );

        // Toggle off → ready_since_tick resets to 0.
        let new_value = state.toggle_ready(43);
        assert!(!new_value);
        assert!(!state.ready_for_tick);
        assert_eq!(
            state.ready_since_tick, 0,
            "ready_since_tick must reset when un-readying"
        );

        // mark_ready is idempotent and does not move the stamp once set.
        state.mark_ready(50);
        assert!(state.ready_for_tick);
        assert_eq!(state.ready_since_tick, 50);
        state.mark_ready(99);
        assert_eq!(
            state.ready_since_tick, 50,
            "mark_ready must not move the original stamp once set"
        );

        // mark_unready clears the flag and the stamp.
        state.mark_unready();
        assert!(!state.ready_for_tick);
        assert_eq!(state.ready_since_tick, 0);

        // ready_count reflects the player list (empty here, so zero).
        assert_eq!(state.ready_count(), 0);
    }
}