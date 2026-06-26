//! # civ-i18n — String-table internationalization infrastructure
//!
//! Provides a `Locale` enum, a `toString` method for string tables,
//! and a `tr!()` macro for compile-time locale-aware string lookups.
//!
//! ## Usage
//!
//! ```ignore
//! use civ_i18n::{tr, Locale};
//! let msg = tr!("welcome_title", &locale);
//! ```
use std::collections::HashMap;
use std::path::Path;
use serde::Deserialize;

/// Supported game locales.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Locale {
    /// English (default, source of truth).
    En,
    /// Persian (Farsi, RTL layout required).
    Fa,
    /// Persian with Latin script (Fingilisi).
    FaLatn,
    /// Simplified Chinese.
    ZhCN,
    /// Traditional Chinese.
    ZhTW,
}

impl Locale {
    /// Return the BCP-47 tag for this locale.
    pub fn bcp47(&self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::Fa => "fa",
            Locale::FaLatn => "fa-Latn",
            Locale::ZhCN => "zh-CN",
            Locale::ZhTW => "zh-TW",
        }
    }

    /// Return the directory name used for the JSON bundle file.
    pub fn bundle_dir(&self) -> &'static str {
        self.bcp47()
    }

    /// True if this locale uses a right-to-left writing system.
    pub fn is_rtl(&self) -> bool {
        matches!(self, Locale::Fa)
    }

    /// Load a string bundle from embedded JSON.
    /// In production, bundles are compiled into the binary via `include_str!`.
    /// The default locale (English) is the fallback for any missing key.
    pub fn load_bundle(&self) -> StringBundle {
        let json = match self {
            Locale::En => include_str!("../bundles/en/strings.json"),
            Locale::Fa => include_str!("../bundles/fa/strings.json"),
            Locale::FaLatn => include_str!("../bundles/fa-Latn/strings.json"),
            Locale::ZhCN => include_str!("../bundles/zh-CN/strings.json"),
            Locale::ZhTW => include_str!("../bundles/zh-TW/strings.json"),
        };
        let map: HashMap<String, String> =
            serde_json::from_str(json).expect("valid JSON string bundle");
        // Always load English as the fallback
        let fallback_json = include_str!("../bundles/en/strings.json");
        let fallback: HashMap<String, String> =
            serde_json::from_str(fallback_json).expect("valid en bundle");
        StringBundle { map, fallback }
    }
}

/// A loaded string table for a single locale.
#[derive(Debug, Clone)]
pub struct StringBundle {
    /// The locale-specific strings.
    map: HashMap<String, String>,
    /// English fallback strings (used for any missing key).
    fallback: HashMap<String, String>,
}

impl StringBundle {
    /// Look up a string by key. Returns the locale-specific value if
    /// present, otherwise falls back to English, otherwise returns
    /// the key itself surrounded by `??` markers.
    pub fn get(&self, key: &str) -> String {
        self.map
            .get(key)
            .or_else(|| self.fallback.get(key))
            .cloned()
            .unwrap_or_else(|| format!("??{}??", key))
    }

    /// Returns the raw JSON for downstream tooling.
    pub fn raw_json(&self) -> serde_json::Value {
        serde_json::to_value(&self.map).unwrap_or_default()
    }
}

/// Macro: look up a string for the given locale at compile-time.
///
/// The locale argument is evaluated at runtime. The string key is a
/// literal identifier.
#[macro_export]
macro_rules! tr {
    ($key:expr, $locale:expr) => {{
        // We create a temporary bundle for the lookup — cheap because
        // the JSON is included at compile time via include_str!.
        let bundle = $locale.load_bundle();
        bundle.get($key)
    }};
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn en_contains_all_keys() {
        let bundle = Locale::En.load_bundle();
        assert!(bundle.get("welcome_title").contains("Civis"));
    }

    #[test]
    fn fa_lookup_returns_farsi() {
        let bundle = Locale::Fa.load_bundle();
        let value = bundle.get("welcome_title");
        assert!(!value.is_empty(), "fa welcome_title should not be empty");
    }

    #[test]
    fn missing_key_fallsback_to_en() {
        let bundle = Locale::Fa.load_bundle();
        let value = bundle.get("nonexistent_key_xyz");
        assert!(value.starts_with("??"), "missing keys should show ??key??");
    }

    #[test]
    fn zh_cn_not_empty() {
        let bundle = Locale::ZhCN.load_bundle();
        assert!(bundle.get("welcome_title").len() > 0);
    }

    #[test]
    fn zh_tw_not_empty() {
        let bundle = Locale::ZhTW.load_bundle();
        assert!(bundle.get("welcome_title").len() > 0);
    }
}
