//! External coverage tests for crates/civis-cli (FR-CIV-TEST-014).
//!
//! Covers: SampleRgb methods (mean_channel, is_gray, is_near_black, hue_bucket),
//!         config helpers (verify_settle_frames_from_env, dump_policy_name_from_env),
//!         lib-level workspace_root + DumpPolicy::from_name, DumpReport::with_policy.
use civis_cli::{
    census::CensusConfig,
    config::{dump_policy_name_from_env, verify_settle_frames_from_env},
    dump::{DumpPolicy, DumpReport},
    pixels::SampleRgb,
    workspace_root,
};

// ── SampleRgb helpers ────────────────────────────────────────────────────────

#[test]
fn sample_rgb_mean_channel_averages_all_three() {
    let px = SampleRgb::new(0, 150, 255);
    let mean = px.mean_channel();
    let expected = (0.0f32 + 150.0 + 255.0) / 3.0;
    assert!((mean - expected).abs() < 0.01, "mean={mean}");
}

#[test]
fn sample_rgb_is_gray_and_is_near_black() {
    // Pure gray mid-tone: R == G == B, above near-black threshold
    let gray = SampleRgb::new(128, 128, 128);
    assert!(gray.is_gray());
    assert!(!gray.is_near_black(8), "128 is not near-black at threshold 8");

    // Near-black: all channels <= threshold
    let black = SampleRgb::new(5, 5, 5);
    assert!(black.is_gray(), "5,5,5 is still gray (R==G==B)");
    assert!(black.is_near_black(8));

    // Chromatic: not gray
    let red = SampleRgb::new(255, 0, 0);
    assert!(!red.is_gray());
    assert!(!red.is_near_black(8));
}

#[test]
fn sample_rgb_hue_bucket_gray_returns_none_color_returns_some() {
    // Gray pixel has no hue
    assert!(SampleRgb::new(128, 128, 128).hue_bucket().is_none());

    // Pure red ≈ 0°
    let bucket_red = SampleRgb::new(255, 0, 0).hue_bucket();
    assert!(bucket_red.is_some(), "red should have a hue bucket");
    let h = bucket_red.unwrap();
    assert!(h < 30 || h > 330, "red hue should be near 0/360°, got {h}");

    // Pure green ≈ 120°
    let bucket_green = SampleRgb::new(0, 255, 0).hue_bucket();
    assert!(bucket_green.is_some());
    let h = bucket_green.unwrap();
    assert!((90..=150).contains(&h), "green hue ≈ 120°, got {h}");
}

// ── config helpers ────────────────────────────────────────────────────────────

#[test]
fn verify_settle_frames_from_env_returns_default_60() {
    // Without CIV_VERIFY_SETTLE_FRAMES set the default is 60
    if std::env::var("CIV_VERIFY_SETTLE_FRAMES").is_err() {
        assert_eq!(verify_settle_frames_from_env(), 60);
    }
}

#[test]
fn dump_policy_name_from_env_returns_none_when_unset() {
    if std::env::var("CIV_DUMP_POLICY").is_err() {
        assert!(dump_policy_name_from_env().is_none());
    }
}

// ── DumpPolicy::from_name + DumpReport helpers ────────────────────────────────

#[test]
fn dump_policy_from_name_accepts_headless_and_headful() {
    let hl = DumpPolicy::from_name("headless").expect("headless");
    assert!(!hl.require_animation_when_actors);

    let hf = DumpPolicy::from_name("headful").expect("headful");
    assert!(hf.require_animation_when_actors);

    let err = DumpPolicy::from_name("bogus");
    assert!(err.is_err(), "unknown policy should error");
}

#[test]
fn dump_report_with_policy_and_with_baseline_attach_labels() {
    let report = DumpReport::pass()
        .with_policy("headless")
        .with_baseline("fixtures/good.json");
    assert!(report.passed);
    assert_eq!(report.policy.as_deref(), Some("headless"));
    assert_eq!(report.baseline.as_deref(), Some("fixtures/good.json"));
}

// ── workspace_root ────────────────────────────────────────────────────────────

#[test]
fn workspace_root_returns_some_path() {
    let root = workspace_root();
    // In a normal cargo workspace this is the parent of the crate manifest dir
    assert!(root.is_some(), "workspace_root should return Some in cargo builds");
}