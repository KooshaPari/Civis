#![cfg(all(feature = "bevy", feature = "egui"))]

//! Live event feed and toast notification overlay for the Civis gameplay HUD.
//!
//! Provides [`EventFeedPlugin`] which registers:
//! - [`EventFeed`]     — resource that other systems push events into.
//! - [`EventLogOpen`]  — toggle resource for the scrollable history window.
//! - `age_events`      — Update system that ages events and prunes old ones.
//! - `draw_event_feed` — EguiPrimaryContextPass system rendering toasts and the log window.
//!
//! Other systems emit events by calling `event_feed.push(kind, text)`.
//! Press **L** in-game to toggle the full scrollable event log.

use std::collections::VecDeque;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use crate::ui_theme::CHIP_FILL;

// ---------------------------------------------------------------------------
// Palette — canonical Keycap tokens from ui_theme; local CHIP_FILL not in ui_theme
// ---------------------------------------------------------------------------

const PANEL_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(17, 20, 31, 235);
const ACCENT: egui::Color32 = egui::Color32::from_rgb(80, 200, 240);
use crate::ui_theme::{ACCENT, PANEL_FILL};

// CHIP_FILL: local tint not present in ui_theme (different from GRAPHITE_700)
const CHIP_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(31, 37, 52, 235);

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

/// Category of a simulation event; controls colour and emoji badge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventKind {
    /// A new citizen was born.
    Birth,
    /// A citizen or unit died.
    Death,
    /// A technology was unlocked.
    Tech,
    /// A battle was fought.
    Battle,
    /// A diplomatic act occurred.
    Diplomacy,
    /// A natural disaster struck.
    Disaster,
    /// Client or transport status (WebSocket attach, HUD system messages).
    System,
}

impl EventKind {
    /// Short emoji prefix shown on every toast card.
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Birth => "🌱",
            Self::Death => "💀",
            Self::Tech => "⚛",
            Self::Battle => "⚔",
            Self::Diplomacy => "🤝",
            Self::Disaster => "🌋",
            Self::System => "📡",
        }
    }

    /// Primary accent colour for this event kind.
    pub fn color(&self) -> egui::Color32 {
        match self {
            Self::Birth => egui::Color32::from_rgb(100, 210, 120),
            Self::Death => egui::Color32::from_rgb(160, 90, 90),
            Self::Tech => ACCENT,
            Self::Battle => egui::Color32::from_rgb(220, 70, 70),
            Self::Diplomacy => egui::Color32::from_rgb(230, 190, 70),
            Self::Disaster => egui::Color32::from_rgb(230, 130, 40),
            Self::System => egui::Color32::from_rgb(140, 170, 210),
        }
    }
}

/// Toast copy for a live WebSocket session state transition.
#[must_use]
pub fn connection_toast_message(state: crate::WsConnectionState) -> &'static str {
    match state {
        crate::WsConnectionState::Connected => "Live server connected",
        crate::WsConnectionState::Reconnecting => "Reconnecting to live server…",
        crate::WsConnectionState::Disconnected => "Live server disconnected",
    }
}

/// A single simulation event stored in the feed.
#[derive(Clone, Debug)]
pub struct GameEvent {
    /// Category of the event.
    pub kind: EventKind,
    /// Human-readable description.
    pub text: String,
    /// Seconds since the event was pushed (incremented by `age_events`).
    pub age: f32,
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Maximum events retained in history.
const FEED_CAP: usize = 50;
/// Seconds after which an event is no longer shown as a toast.
const TOAST_LIFETIME: f32 = 8.0;
/// Number of most-recent events shown as stacked toasts.
const TOAST_COUNT: usize = 5;

/// Central resource holding the live simulation event log.
///
/// Push new events via [`EventFeed::push`].  Other resources and systems can
/// read `events` directly; the deque is newest-front.
#[derive(Resource, Default)]
pub struct EventFeed {
    /// All retained events, newest first.
    pub events: VecDeque<GameEvent>,
}

impl EventFeed {
    /// Push a new event to the front of the feed (newest-first).
    ///
    /// If the feed has already reached [`FEED_CAP`], the oldest entry is
    /// dropped to make room.
    pub fn push(&mut self, kind: EventKind, text: impl Into<String>) {
        if self.events.len() >= FEED_CAP {
            self.events.pop_back();
        }
        self.events.push_front(GameEvent {
            kind,
            text: text.into(),
            age: 0.0,
        });
    }

    /// Seed the feed with representative sample events for UI testing.
    pub fn demo_seed(&mut self) {
        self.push(EventKind::Birth, "A child was born in Ironmere.");
        self.push(EventKind::Tech, "Ironmere unlocked Iron Smelting.");
        self.push(
            EventKind::Battle,
            "Clash at Dustford — Ironmere victorious.",
        );
        self.push(
            EventKind::Diplomacy,
            "Ironmere and Thalmark signed a ceasefire.",
        );
        self.push(
            EventKind::Disaster,
            "Volcanic eruption buried the eastern mines.",
        );
        self.push(
            EventKind::Death,
            "General Varen fell in the Battle of Crestholm.",
        );
    }
}

/// Toggle resource for the scrollable event log window.  Bound to **L**.
#[derive(Resource, Default)]
pub struct EventLogOpen(pub bool);

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Registers the event feed resources and rendering systems.
pub struct EventFeedPlugin;

impl Plugin for EventFeedPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EventFeed>()
            .init_resource::<EventLogOpen>()
            .add_systems(Update, age_events)
            .add_systems(EguiPrimaryContextPass, draw_event_feed);
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Advance the age of all events each frame and prune those past twice the
/// toast lifetime so they are still visible in the log for a while.
pub fn age_events(time: Res<Time>, mut feed: ResMut<EventFeed>) {
    let dt = time.delta_secs();
    for ev in &mut feed.events {
        ev.age += dt;
    }
    // Keep history up to FEED_CAP entries regardless of age; only toasts use
    // the age for opacity.  No hard expiry from history.
}

/// Draw the toast overlay and (when open) the full event log window.
pub fn draw_event_feed(
    mut contexts: EguiContexts,
    feed: Res<EventFeed>,
    mut log_open: ResMut<EventLogOpen>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::KeyL) {
        log_open.0 = !log_open.0;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    draw_toasts(ctx, &feed);

    if log_open.0 {
        draw_log_window(ctx, &feed, &mut log_open);
    }
}

// ---------------------------------------------------------------------------
// UI helpers  (each ≤ 40 lines)
// ---------------------------------------------------------------------------

/// Render the most-recent [`TOAST_COUNT`] events as stacked glass cards,
/// anchored to the bottom-right corner above the bottom HUD bar (~60 px).
fn draw_toasts(ctx: &egui::Context, feed: &EventFeed) {
    let screen = ctx.screen_rect();
    // Anchor: bottom-right with margin above the ~60 px bottom HUD bar.
    let anchor = egui::pos2(screen.right() - 16.0, screen.bottom() - 68.0);

    egui::Area::new(egui::Id::new("civis_toast_area"))
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-16.0, -68.0))
        .order(egui::Order::Foreground)
        .interactable(false)
        .show(ctx, |ui| {
            ui.set_max_width(320.0);
            ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                for ev in feed.events.iter().take(TOAST_COUNT) {
                    toast_card(ui, ev);
                    ui.add_space(4.0);
                }
            });
        });

    // suppress unused variable warning — anchor is only used conceptually.
    let _ = anchor;
}

/// Render a single rounded glass toast card with emoji badge, text, and
/// fade-out alpha proportional to how close the event is to expiry.
fn toast_card(ui: &mut egui::Ui, ev: &GameEvent) {
    let alpha_frac = 1.0 - (ev.age / TOAST_LIFETIME).clamp(0.0, 1.0);
    let alpha = (alpha_frac * 235.0) as u8;

    let fill = egui::Color32::from_rgba_premultiplied(17, 20, 31, alpha.saturating_add(30));
    let kind_color = ev.kind.color();
    let text_color = egui::Color32::from_rgba_unmultiplied(220, 225, 235, alpha);
    let accent_color = egui::Color32::from_rgba_unmultiplied(
        kind_color.r(),
        kind_color.g(),
        kind_color.b(),
        alpha,
    );

    egui::Frame::NONE
        .fill(fill)
        .stroke(egui::Stroke::new(1.0, accent_color.gamma_multiply(0.6)))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(ev.kind.emoji())
                        .color(accent_color)
                        .size(16.0),
                );
                ui.label(egui::RichText::new(&ev.text).color(text_color).size(13.0));
            });
        });
}

/// Render the scrollable "📜 Event Log" window, toggled by [`EventLogOpen`].
fn draw_log_window(ctx: &egui::Context, feed: &EventFeed, log_open: &mut EventLogOpen) {
    let mut open = log_open.0;
    egui::Window::new("📜 Event Log")
        .open(&mut open)
        .resizable(true)
        .default_size([360.0, 420.0])
        .frame(
            egui::Frame::NONE
                .fill(PANEL_FILL)
                .stroke(egui::Stroke::new(1.0, ACCENT.gamma_multiply(0.4)))
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(egui::Margin::same(12)),
        )
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new(format!("{} events", feed.events.len()))
                    .color(egui::Color32::from_rgb(150, 158, 178))
                    .small(),
            );
            ui.add_space(4.0);
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    for ev in feed.events.iter() {
                        log_row(ui, ev);
                    }
                });
        });
    log_open.0 = open;
}

/// Render a single row inside the log window.
fn log_row(ui: &mut egui::Ui, ev: &GameEvent) {
    let color = ev.kind.color();
    egui::Frame::NONE
        .fill(CHIP_FILL)
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(ev.kind.emoji()).color(color).size(14.0));
                ui.label(egui::RichText::new(&ev.text).size(13.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("{:.0}s", ev.age))
                            .color(egui::Color32::from_rgb(100, 108, 128))
                            .small(),
                    );
                });
            });
        });
    ui.add_space(2.0);
}

// ---------------------------------------------------------------------------
// Pure helper (also used by tests)
// ---------------------------------------------------------------------------

/// Filter a slice of events, returning only those younger than `max_age_secs`.
///
/// This is a pure function so it can be unit-tested without Bevy running.
pub fn events_younger_than(events: &VecDeque<GameEvent>, max_age_secs: f32) -> Vec<&GameEvent> {
    events.iter().filter(|e| e.age < max_age_secs).collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-CIV-BEVY-023 — feed cap + ordering semantics are validated for event log HUD/toasts.
    fn make_event(kind: EventKind, age: f32) -> GameEvent {
        GameEvent {
            kind,
            text: "test".into(),
            age,
        }
    }

    /// FR-CIV-BEVY-023 — feed cap and ordering semantics are validated.
    #[test]
    fn push_respects_cap() {
        let mut feed = EventFeed::default();
        for i in 0..=(FEED_CAP + 5) {
            feed.push(EventKind::Birth, format!("event {i}"));
        }
        assert_eq!(feed.events.len(), FEED_CAP, "feed must not exceed FEED_CAP");
    }

    #[test]
    fn push_newest_front() {
        let mut feed = EventFeed::default();
        feed.push(EventKind::Birth, "first");
        feed.push(EventKind::Death, "second");
        assert_eq!(feed.events.front().unwrap().text, "second");
        assert_eq!(feed.events.back().unwrap().text, "first");
    }

    #[test]
    fn events_younger_than_filters_correctly() {
        let mut deque: VecDeque<GameEvent> = VecDeque::new();
        deque.push_back(make_event(EventKind::Birth, 2.0));
        deque.push_back(make_event(EventKind::Tech, 9.0));
        deque.push_back(make_event(EventKind::Battle, 5.0));

        let young = events_younger_than(&deque, TOAST_LIFETIME);
        assert_eq!(
            young.len(),
            2,
            "only events younger than 8 s should be returned"
        );
        assert!(young.iter().all(|e| e.age < TOAST_LIFETIME));
    }

    #[test]
    fn events_younger_than_empty_feed() {
        let deque: VecDeque<GameEvent> = VecDeque::new();
        assert!(events_younger_than(&deque, TOAST_LIFETIME).is_empty());
    }

    #[test]
    fn demo_seed_populates_feed() {
        let mut feed = EventFeed::default();
        feed.demo_seed();
        assert!(
            !feed.events.is_empty(),
            "demo_seed must add at least one event"
        );
        assert!(feed.events.len() <= FEED_CAP);
    }

    #[test]
    fn connection_toast_message_matches_ws_states() {
        use crate::WsConnectionState;

        assert_eq!(
            connection_toast_message(WsConnectionState::Connected),
            "Live server connected"
        );
        assert!(connection_toast_message(WsConnectionState::Reconnecting).contains("Reconnect"));
        assert!(connection_toast_message(WsConnectionState::Disconnected).contains("disconnect"));
    }

    #[test]
    fn event_kind_emoji_and_color_are_consistent() {
        let kinds = [
            EventKind::Birth,
            EventKind::Death,
            EventKind::Tech,
            EventKind::Battle,
            EventKind::Diplomacy,
            EventKind::Disaster,
            EventKind::System,
        ];
        for kind in &kinds {
            assert!(!kind.emoji().is_empty());
            // color() must not panic and must return a non-transparent value
            let c = kind.color();
            assert!(c.a() > 0, "color alpha should be > 0 for kind {:?}", kind);
        }
    }
}
