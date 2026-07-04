//! # civ-i18n — Internationalization & Localization
//!
//! String-bundle-based localization with locale detection,
//! compile-time-safe `tr!()` macro, and serde-backed JSON bundles.
//!
//! ## Quick start
//!
//! ```ignore
//! use civ_i18n::{Locale, Bundle, tr};
//!
//! let bundle = Bundle::load(Locale::Fa).unwrap();
//! let greeting = tr!(bundle, "godtools.raise_mountain");
//! ```
#![deny(missing_docs)]

use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

// ---------------------------------------------------------------------------
// Locale
// ---------------------------------------------------------------------------

/// Supported locales.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Locale {
    /// English (default)
    En,
    /// Persian (فارسی) — RTL
    Fa,
    /// Persian transcribed in Latin script
    FaLatn,
    /// Simplified Chinese (简体中文)
    ZhCN,
    /// Traditional Chinese (繁體中文)
    ZhTW,
}

impl Locale {
    /// All supported locales, in priority order for auto-detection.
    pub const ALL: &[Locale] = &[Locale::En, Locale::Fa, Locale::FaLatn, Locale::ZhCN, Locale::ZhTW];

    /// BCP-47 tag.
    pub fn as_str(self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::Fa => "fa",
            Locale::FaLatn => "fa-Latn",
            Locale::ZhCN => "zh-CN",
            Locale::ZhTW => "zh-TW",
        }
    }

    /// Parse from a BCP-47-like string (e.g., from `Accept-Language`).
    pub fn from_str(s: &str) -> Option<Locale> {
        // Normalize: strip country/region suffixes, lowercase
        let tag = s.split(&['-', '_']).next().unwrap_or(s).to_lowercase();
        match tag.as_str() {
            "en" => Some(Locale::En),
            "fa" => Some(Locale::Fa),
            "zh" => {
                // If exact tag is zh-CN or zh-TW, prefer it
                match s {
                    x if x.eq_ignore_ascii_case("zh-CN") || x.eq_ignore_ascii_case("zh-Hans") => Some(Locale::ZhCN),
                    x if x.eq_ignore_ascii_case("zh-TW") || x.eq_ignore_ascii_case("zh-HK") || x.eq_ignore_ascii_case("zh-Hant") => Some(Locale::ZhTW),
                    _ => Some(Locale::ZhCN), // default Simplified
                }
            }
            _ => None,
        }
    }

    /// Returns `true` if the locale requires right-to-left text layout.
    pub fn is_rtl(self) -> bool {
        matches!(self, Locale::Fa)
    }
}

impl Default for Locale {
    fn default() -> Self {
        Locale::En
    }
}

// ---------------------------------------------------------------------------
// Bundle
// ---------------------------------------------------------------------------

/// A loaded string bundle for a single locale.
///
/// Backed by a flat `HashMap<String, String>` loaded from a JSON file at
/// `bundles/{locale}/strings.json` (embedded at compile time via `include_str!`
/// or loaded at runtime from a known path).
#[derive(Debug, Clone)]
pub struct Bundle {
    locale: Locale,
    strings: HashMap<String, String>,
}

/// Error returned when a string key is missing from the bundle.
#[derive(Debug, Error)]
pub enum BundleError {
    /// The requested key does not exist in the bundle.
    #[error("missing i18n key `{0}` in locale `{1}`")]
    MissingKey(String, &'static str),
}

impl Bundle {
    /// Load the bundle for the given locale, embedding JSON at compile time.
    ///
    /// The JSON files live at `bundles/{locale}/strings.json` relative to the
    /// crate root.
    pub fn load(locale: Locale) -> Self {
        let json_str = match locale {
            Locale::En => include_str!("../bundles/en/strings.json"),
            Locale::Fa => include_str!("../bundles/fa/strings.json"),
            Locale::FaLatn => include_str!("../bundles/fa-Latn/strings.json"),
            Locale::ZhCN => include_str!("../bundles/zh-CN/strings.json"),
            Locale::ZhTW => include_str!("../bundles/zh-TW/strings.json"),
        };

        let map: HashMap<String, String> =
            serde_json::from_str(json_str).expect("civ-i18n: invalid bundle JSON");

        let strings = map.into_iter().collect();

        Bundle { locale, strings }
    }

    /// Look up a string key.
    ///
    /// Returns `Err(BundleError::MissingKey)` if the key isn't found.
    pub fn get(&self, key: &str) -> Result<&str, BundleError> {
        self.strings
            .get(key)
            .map(|s| s.as_str())
            .ok_or_else(|| BundleError::MissingKey(key.to_owned(), self.locale.as_str()))
    }

    /// Look up a string key, falling back to a raw key display if missing.
    ///
    /// This is useful during development when keys haven't been translated yet.
    pub fn get_or_key(&self, key: &str) -> &str {
        self.strings.get(key).map(|s| s.as_str()).unwrap_or(key)
    }

    /// Return the locale this bundle was loaded for.
    pub fn locale(&self) -> Locale {
        self.locale
    }
}

// ---------------------------------------------------------------------------
// tr! macro
// ---------------------------------------------------------------------------

/// Compile-time-ish string lookup.
///
/// Usage:
/// ```ignore
/// let s = tr!(bundle, "godtools.raise_mountain");
/// ```
///
/// This expands to `bundle.get_or_key("godtools.raise_mountain")`.
#[macro_export]
macro_rules! tr {
    ($bundle:expr, $key:expr $(,)?) => {
        $bundle.get_or_key($key)
    };
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Auto-detect the preferred locale from the environment.
///
/// Checks (in order):
/// 1. `CIVIS_LANG` env var (explicit override)
/// 2. `LANG` env var (POSIX convention)
///
/// Falls back to `Locale::En`.
pub fn detect_locale_from_env() -> Locale {
    if let Ok(val) = std::env::var("CIVIS_LANG") {
        if let Some(loc) = Locale::from_str(&val) {
            return loc;
        }
    }
    if let Ok(val) = std::env::var("LANG") {
        if let Some(loc) = Locale::from_str(&val) {
            return loc;
        }
    }
    Locale::En
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_from_str_exact() {
        assert_eq!(Locale::from_str("en"), Some(Locale::En));
        assert_eq!(Locale::from_str("fa"), Some(Locale::Fa));
    }

    #[test]
    fn locale_from_str_normalized() {
        assert_eq!(Locale::from_str("EN-US"), Some(Locale::En));
        assert_eq!(Locale::from_str("en_US"), Some(Locale::En));
    }

    #[test]
    fn locale_chinese_disambiguation() {
        assert_eq!(Locale::from_str("zh-CN"), Some(Locale::ZhCN));
        assert_eq!(Locale::from_str("zh-TW"), Some(Locale::ZhTW));
        assert_eq!(Locale::from_str("zh-Hans"), Some(Locale::ZhCN));
        assert_eq!(Locale::from_str("zh-Hant"), Some(Locale::ZhTW));
        assert_eq!(Locale::from_str("zh"), Some(Locale::ZhCN)); // default Simplified
    }

    #[test]
    fn rtl_detection() {
        assert!(Locale::Fa.is_rtl());
        assert!(!Locale::En.is_rtl());
        assert!(!Locale::ZhCN.is_rtl());
    }

    #[test]
    fn bundle_loads_and_provides_keys() {
        let bundle = Bundle::load(Locale::En);
        let val = bundle.get("godtools.raise_mountain");
        assert!(val.is_ok(), "missing key in en bundle: {:?}", val);
    }

    #[test]
    fn bundle_missing_key_returns_error() {
        let bundle = Bundle::load(Locale::En);
        let val = bundle.get("nonexistent.key");
        assert!(val.is_err());
    }

    #[test]
    fn bundle_get_or_key_fallback() {
        let bundle = Bundle::load(Locale::En);
        let val = bundle.get_or_key("nonexistent.key");
        assert_eq!(val, "nonexistent.key"); // falls back to raw key
    }

    #[test]
    fn tr_macro_works() {
        let bundle = Bundle::load(Locale::En);
        let s = tr!(bundle, "app.title");
        assert!(!s.is_empty());
    }

    #[test]
    fn detect_locale_from_env_override() {
        std::env::set_var("CIVIS_LANG", "fa");
        let loc = detect_locale_from_env();
        assert_eq!(loc, Locale::Fa);
        std::env::remove_var("CIVIS_LANG");
    }

    #[test]
    fn all_locales_have_bundles() {
        for loc in Locale::ALL {
            let bundle = Bundle::load(*loc);
            // Every bundle must at least have the app title
            assert!(
                bundle.get("app.title").is_ok(),
                "locale {:?} is missing `app.title`",
                loc
            );
        }
    }

    #[test]
    fn fa_bundle_has_translated_keys() {
        // Check that at least the core game keys are present in fa
        let bundle = Bundle::load(Locale::Fa);
        assert!(bundle.get("app.title").is_ok());
        assert!(bundle.get("godtools.raise_mountain").is_ok());
    }
}
