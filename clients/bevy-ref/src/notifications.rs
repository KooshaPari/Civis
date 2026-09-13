#![cfg(all(feature = "bevy", feature = "egui"))]

//! Event-feed / toast notifications for the Civis gameplay HUD.
//!
//! FR-CIV-NOTIFY-*:
//! - event kinds cover births, deaths, diplomacy, tech and disasters.
//! - notifications are stored in a ring buffer resource.
//! - the overlay is themed through [`crate::ui_theme`].
//! - the UI is gated to [`crate::menus::GameUiMode::Playing`] so menus and
//!   loading screens stay uncluttered.
//!
//! **Server-wired** — sends `sim.subscribe` with `{ "events": true }` on
//! first draw (line ~151) to receive real-time event-feed frames from the
//! server via the [`ServerBridge`](crate::live_stream::ServerBridge).

use std::collections::VecDeque;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::live_stream::ServerBridge;
use crate::menus::{in_playing_state, GameUiMode};
use crate::ui_theme::{
    accent_frame, apply_theme, inner_glow, ACCENT_HI, GOLD, GREEN, RADIUS_SM, RED, TEXT, VIOLET,
};

/// Maximum notifications retained in the ring buffer.
const NOTIFICATION_CAP: usize = 64;
/// Seconds after which a notification fades out of the toast stack.
const TOAST_LIFETIME_SECS: f32 = 8.0;
/// Seconds the fade-IN ramps from 0 → 1 (smoothstep, perceptually soft).
const TOAST_FADE_IN_SECS: f32 = 0.35;
/// Maximum number of newest notifications shown as stacked toasts.
const TOAST_STACK: usize = 6;
/// Bottom-left overlay margin.
const PANEL_MARGIN: f32 = 16.0;

/// Pure helper — visibility alpha for a toast at the given age (seconds).
///
/// Returns a value in `[0.0, 1.0]`:
/// - `age <= 0` or `age >= TOAST_LIFETIME_SECS` → 0.0
/// - `age < TOAST_FADE_IN_SECS` → smoothstep ramp 0 → 1 (soft entrance)
/// - `age in [TOAST_FADE_IN_SECS, TOAST_LIFETIME_SECS]` → 1.0 (fully visible)
/// - `age in [TOAST_LIFETIME_SECS - 1.0, TOAST_LIFETIME_SECS]` → linear ramp to 0
///   (last second is a soft fade-out so the dismiss is gentle, not abrupt).
///
/// The last-second fade-out matches the prior behaviour (`fade = 1 - age/8`)
/// but is now bounded so a notification at exactly `TOAST_LIFETIME_SECS` is
/// invisible rather than partially translucent.
fn toast_alpha(age_secs: f32) -> f32 {
    if age_secs <= 0.0 || age_secs >= TOAST_LIFETIME_SECS {
        return 0.0;
    }
    // Fade-in: smoothstep ramp over TOAST_FADE_IN_SECS.
    if age_secs < TOAST_FADE_IN_SECS {
        let t = age_secs / TOAST_FADE_IN_SECS;
        // smoothstep: 3t² - 2t³
        return t * t * (3.0 - 2.0 * t);
    }
    // Steady state: full opacity through the middle of the lifetime.
    if age_secs < TOAST_LIFETIME_SECS - 1.0_f32 {
        return 1.0;
    }
    // Last-second fade-out: linear ramp to 0 at TOAST_LIFETIME_SECS.
    (TOAST_LIFETIME_SECS - age_secs).clamp(0.0, 1.0)
}

/// Notification categories used by the event feed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NotificationKind {
    /// A birth or new settlement/population event.
    Birth,
    /// A death, loss or casualty event.
    Death,
    /// A diplomacy update.
    Diplomacy,
    /// A technology discovery / unlock.
    Tech,
    /// A disaster or natural hazard.
    Disaster,
}

impl NotificationKind {
    fn accent(&self) -> egui::Color32 {
        match self {
            Self::Birth => GREEN,
            Self::Death => RED,
            Self::Diplomacy => GOLD,
            Self::Tech => ACCENT_HI,
            Self::Disaster => VIOLET,
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Self::Birth => "✦",
            Self::Death => "✕",
            Self::Diplomacy => "✎",
            Self::Tech => "⚙",
            Self::Disaster => "⚠",
        }
    }
}

/// One toast entry retained in the notification feed.
#[derive(Clone, Debug)]
pub struct Notification {
    /// Message shown in the toast feed.
    pub message: String,
    /// Semantic category for theming and future routing.
    pub kind: NotificationKind,
    /// Seconds since the notification was created.
    pub age_secs: f32,
}

/// Ring buffer resource used by gameplay systems to publish notifications.
#[derive(Resource, Debug, Default)]
pub struct Notifications {
    /// Newest-first retained notifications.
    pub items: VecDeque<Notification>,
}

impl Notifications {
    /// Push a new notification into the ring buffer.
    pub fn notify(&mut self, kind: NotificationKind, message: impl Into<String>) {
        if self.items.len() >= NOTIFICATION_CAP {
            self.items.pop_back();
        }

        self.items.push_front(Notification {
            message: message.into(),
            kind,
            age_secs: 0.0,
        });
    }
}

/// Convenience API for gameplay systems that already have a mutable resource.
pub fn notify(
    notifications: &mut Notifications,
    kind: NotificationKind,
    message: impl Into<String>,
) {
    notifications.notify(kind, message);
}

/// Plugin that wires the notification resource and toast overlay systems.
pub struct NotificationsPlugin;

impl Plugin for NotificationsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Notifications>()
            .add_systems(Update, age_notifications.run_if(in_playing_state))
            .add_systems(
                EguiPrimaryContextPass,
                draw_notifications.run_if(in_playing_state),
            );
    }
}

fn in_playing(mode: Res<GameUiMode>) -> bool {
    *mode == GameUiMode::Playing
}

fn age_notifications(time: Res<Time>, mut notifications: ResMut<Notifications>) {
    let dt = time.delta_secs();
    for notification in &mut notifications.items {
        notification.age_secs += dt;
    }
}

fn draw_notifications(
    mut contexts: EguiContexts,
    mut notifications_mut: ResMut<Notifications>,
    bridge: Option<Res<ServerBridge>>,
    mut sent_subscribe: Local<bool>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    apply_theme(ctx);
    apply_theme(ctx);
    // Server: subscribe to event frames for real-time alerts
    if let Some(ref bridge) = bridge {
        if !*sent_subscribe {
            bridge.send_rpc("sim.subscribe", serde_json::json!({ "events": true }));
            *sent_subscribe = true;
        }
    }
    let content_rect = ctx.content_rect();
    let anchor = egui::pos2(
        content_rect.left() + PANEL_MARGIN,
        content_rect.bottom() - PANEL_MARGIN,
    );

    let mut dismiss_request = None;

    egui::Area::new(egui::Id::new("civis_notifications_area"))
        .fixed_pos(anchor)
        .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(0.0, 0.0))
        .order(egui::Order::Foreground)
        .interactable(true)
        .show(ctx, |ui| {
            ui.set_max_width(360.0);
            ui.spacing_mut().item_spacing.y = 6.0;
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                for (index, notification) in
                    notifications_mut.items.iter().take(TOAST_STACK).enumerate()
                {
                    if toast_card(ui, index, notification).clicked() {
                        dismiss_request = Some(index);
                    }
                }
            });
        });

    if let Some(index) = dismiss_request {
        notifications_mut.items.remove(index);
    }

    // Age-based expiry happens after the draw so the UI can still see the full
    // lifetime on the current frame before the oldest items drop away.
    notifications_mut
        .items
        .retain(|notification| notification.age_secs < TOAST_LIFETIME_SECS);
}

fn toast_card(ui: &mut egui::Ui, index: usize, notification: &Notification) -> egui::Response {
    let alpha_factor = toast_alpha(notification.age_secs);
    let alpha = (alpha_factor * 235.0).max(40.0) as u8;
    let accent = notification.kind.accent();
    let fill = egui::Color32::from_rgba_premultiplied(16, 20, 30, alpha);
    let text_color = egui::Color32::from_rgba_unmultiplied(TEXT.r(), TEXT.g(), TEXT.b(), alpha);
    let dim_color = egui::Color32::from_rgba_unmultiplied(152, 161, 182, alpha);
    let accent_color =
        egui::Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), alpha);

    let id = ui.make_persistent_id(("civis_notification_toast", index));
    let response = accent_frame(egui::Margin::symmetric(10, 8), accent)
        .fill(fill)
        .show(ui, |ui| {
            ui.set_min_width(300.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(notification.kind.icon())
                        .color(accent_color)
                        .size(16.0),
                );
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(&notification.message)
                            .color(text_color)
                            .size(13.5),
                    );
                    ui.label(
                        egui::RichText::new(kind_label(&notification.kind))
                            .color(dim_color)
                            .small(),
                    );
                });
            });
        })
        .response;

    let response = ui.interact(response.rect, id, egui::Sense::click());

    let stroke = egui::Stroke::new(1.0_f32, accent.gamma_multiply(0.5));
    ui.painter().rect_stroke(
        response.rect,
        egui::CornerRadius::same(RADIUS_SM),
        stroke,
        egui::StrokeKind::Inside,
    );
    inner_glow(ui.painter(), response.rect, accent, RADIUS_SM);
    response
}

fn kind_label(kind: &NotificationKind) -> &'static str {
    match kind {
        NotificationKind::Birth => "Birth",
        NotificationKind::Death => "Death",
        NotificationKind::Diplomacy => "Diplomacy",
        NotificationKind::Tech => "Technology",
        NotificationKind::Disaster => "Disaster",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_buffer_discards_oldest_item_first() {
        let mut notifications = Notifications::default();
        for i in 0..NOTIFICATION_CAP {
            notifications.notify(NotificationKind::Tech, format!("item {i}"));
        }
        notifications.notify(NotificationKind::Disaster, "newest");

        assert_eq!(notifications.items.len(), NOTIFICATION_CAP);
        assert_eq!(notifications.items.front().unwrap().message, "newest");
        assert_eq!(
            notifications.items.back().unwrap().message,
            "item 1",
            "oldest entry should drop when cap is exceeded"
        );
    }

    #[test]
    fn kind_labels_cover_required_domains() {
        assert_eq!(kind_label(&NotificationKind::Birth), "Birth");
        assert_eq!(kind_label(&NotificationKind::Death), "Death");
        assert_eq!(kind_label(&NotificationKind::Diplomacy), "Diplomacy");
        assert_eq!(kind_label(&NotificationKind::Tech), "Technology");
        assert_eq!(kind_label(&NotificationKind::Disaster), "Disaster");
    }

    #[test]
    fn toast_alpha_is_zero_at_boundaries() {
        assert_eq!(toast_alpha(0.0), 0.0, "at age 0, alpha must be 0 (start of fade-in)");
        assert_eq!(
            toast_alpha(-1.0),
            0.0,
            "negative ages (clock skew) must clamp to 0"
        );
        assert_eq!(
            toast_alpha(TOAST_LIFETIME_SECS),
            0.0,
            "at age == LIFETIME, alpha must be 0 (item is being dismissed)"
        );
        assert_eq!(
            toast_alpha(TOAST_LIFETIME_SECS + 1.0),
            0.0,
            "past lifetime, alpha must remain 0"
        );
    }

    #[test]
    fn toast_alpha_smoothstep_inflection_at_midpoint() {
        let mid = toast_alpha(TOAST_FADE_IN_SECS * 0.5);
        // Smoothstep at t=0.5 is exactly 0.5 by construction:
        //   3(0.25) - 2(0.125) = 0.75 - 0.25 = 0.5
        assert!(
            (mid - 0.5).abs() < 1e-5,
            "smoothstep midpoint should be 0.5, got {mid}"
        );
        // Below midpoint must be sub-linear; above must be super-linear.
        let quarter = toast_alpha(TOAST_FADE_IN_SECS * 0.25);
        let three_quarter = toast_alpha(TOAST_FADE_IN_SECS * 0.75);
        assert!(
            quarter < 0.25,
            "smoothstep at t=0.25 should be < 0.25 (sub-linear), got {quarter}"
        );
        assert!(
            three_quarter > 0.75,
            "smoothstep at t=0.75 should be > 0.75 (super-linear), got {three_quarter}"
        );
    }

    #[test]
    fn toast_alpha_reaches_full_visibility_in_steady_state() {
        assert_eq!(
            toast_alpha(TOAST_FADE_IN_SECS),
            1.0,
            "end of fade-in window must be fully visible"
        );
        assert_eq!(
            toast_alpha(TOAST_LIFETIME_SECS * 0.5),
            1.0,
            "midpoint of lifetime must be fully visible"
        );
        assert_eq!(
            toast_alpha(TOAST_LIFETIME_SECS - 1.0 - 0.001),
            1.0,
            "one second before end must still be fully visible"
        );
    }

    #[test]
    fn toast_alpha_is_monotonically_increasing_through_fade_in() {
        // Sample the fade-in window at 16 points and assert strict monotonicity
        // plus non-overshoot (alpha never exceeds 1.0 mid-fade-in).
        let mut prev = 0.0;
        for step in 1..=16 {
            let age = (step as f32 / 16.0) * TOAST_FADE_IN_SECS;
            let alpha = toast_alpha(age);
            assert!(
                alpha > prev,
                "alpha must strictly increase during fade-in (step {step}, age {age:.3}): {prev:.3} -> {alpha:.3}"
            );
            assert!(alpha <= 1.0, "alpha must not overshoot during fade-in");
            prev = alpha;
        }
        assert_eq!(prev, 1.0, "fade-in must end exactly at 1.0");
    }

    #[test]
    fn toast_alpha_fades_out_smoothly_in_last_second() {
        // The last 1.0s of the lifetime is a linear ramp from 1.0 -> 0.0.
        let start = toast_alpha(TOAST_LIFETIME_SECS - 1.0);
        let half = toast_alpha(TOAST_LIFETIME_SECS - 0.5);
        let end = toast_alpha(TOAST_LIFETIME_SECS);
        assert_eq!(start, 1.0, "last-second fade-out should start at 1.0");
        assert!(
            (half - 0.5).abs() < 1e-5,
            "last-second fade-out midpoint should be 0.5, got {half}"
        );
        assert_eq!(end, 0.0, "last-second fade-out should end at 0.0");
    }
}
