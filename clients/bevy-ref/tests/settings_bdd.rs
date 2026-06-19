#![cfg(all(feature = "bevy", feature = "egui"))]

use bevy::prelude::{KeyCode, MouseButton};
use civ_bevy_ref::settings_ui::{
    AntiAliasing, GameSettings, GraphicsSettings, KeyBinding, QualityPreset, ShadowQuality,
    TextureQuality,
};

#[test]
fn bdd_given_ultra_preset_when_applied_then_all_rich_fields_are_max() {
    let mut g = GraphicsSettings::default();

    g.apply_preset(QualityPreset::Ultra);

    assert_eq!(g.quality, QualityPreset::Ultra);
    assert!((g.resolution_scale - 2.0).abs() < f32::EPSILON);
    assert_eq!(g.shadow_quality, ShadowQuality::Ultra);
    assert_eq!(g.anti_aliasing, AntiAliasing::MSAA);
    assert_eq!(g.view_distance, 1024);
    assert_eq!(g.texture_quality, TextureQuality::High);
    assert!(g.ambient_occlusion);
    assert!(g.motion_blur);
    assert!(g.bloom);
    assert!(g.gi);
    assert!(g.vfx);
}

#[test]
fn bdd_given_manual_shadow_change_when_applied_then_quality_becomes_custom() {
    let mut g = GraphicsSettings::default();

    g.apply_preset(QualityPreset::High);
    g.shadow_quality = ShadowQuality::Low;
    g.mark_custom();

    assert_eq!(g.quality, QualityPreset::Custom);
}

#[test]
fn key_lookup_is_authoritative() {
    let s = GameSettings::default();
    assert_eq!(
        s.key_for("Toggle Settings"),
        Some(KeyBinding::Key(KeyCode::KeyO))
    );
    assert_eq!(
        s.key_for("Zoom Camera"),
        Some(KeyBinding::Mouse(MouseButton::Middle))
    );
}
