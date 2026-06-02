#![cfg(all(feature = "bevy", feature = "egui"))]

//! Event-feed / toast notifications for the Civis gameplay HUD.
//!
//! FR-CIV-NOTIFY-*:
//! - event kinds cover births, deaths, diplomacy, tech and disasters.
//! - notifications are stored in a ring buffer resource.
//! - the overlay is themed through [`crate::ui_theme`].
//! - the UI is gated to [`crate::menus::GameUiMode::Playing`] so menus and
//!   loading screens stay uncluttered.

use std::collections::VecDeque;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::menus::GameUiMode;
use crate::ui_theme::{
    accent_frame, apply_theme, inner_glow, ACCENT_HI, GOLD, GREEN, RADIUS_SM, RED, TEXT, VIOLET,
};

/// Maximum notifications retained in the ring buffer.
const NOTIFICATION_CAP: usize = 64;
/// Seconds after which a notification fades out of the toast stack.
const TOAST_LIFETIME_SECS: f32 = 8.0;
/// Maximum number of newest notifications shown as stacked toasts.
const TOAST_STACK: usize = 6;
/// Bottom-left overlay margin.
const PANEL_MARGIN: f32 = 16.0;

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
            .add_systems(Update, age_notifications.run_if(in_playing))
            .add_systems(
                EguiPrimaryContextPass,
                draw_notifications.run_if(in_playing),
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

fn draw_notifications(mut contexts: EguiContexts, mut notifications_mut: ResMut<Notifications>) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    apply_theme(ctx);

    let screen_rect = ctx.content_rect();
    let anchor = egui::pos2(
        screen_rect.left() + PANEL_MARGIN,
        screen_rect.bottom() - PANEL_MARGIN,
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
    let fade = 1.0 - (notification.age_secs / TOAST_LIFETIME_SECS).clamp(0.0, 1.0);
    let alpha = (fade * 235.0).max(40.0) as u8;
    let accent = notification.kind.accent();
    let fill = crate::ui_theme::KC_BG_ELV.gamma_multiply(alpha as f32 / 255.0);
    let text_color = egui::Color32::from_rgba_unmultiplied(TEXT.r(), TEXT.g(), TEXT.b(), alpha);
    let dim = crate::ui_theme::TEXT_MID;
    let dim_color = egui::Color32::from_rgba_unmultiplied(dim.r(), dim.g(), dim.b(), alpha);
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

    let stroke = egui::Stroke::new(1.0, accent.gamma_multiply(0.5));
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
}
