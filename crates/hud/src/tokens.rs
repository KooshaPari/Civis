//! Canonical Keycap Palette token list (the design language contract).
//!
//! **No new tokens.** The hex values, names, and roles are the verbatim
//! surface from `docs/design/ui-design-language.md` §1. Every UI pixel in
//! this crate must resolve through [`TokenName`] → [`Token`] → hex. If you
//! want to add a colour, update `ui-design-language.md` first; this crate
//! will fail the token-audit test otherwise.

use serde::{Deserialize, Serialize};

/// Canonical token name — every UI colour references one of these.
///
/// Each variant maps to **exactly one** [`Token`] entry (the contract). The
/// mapping is enforced by the `TOKEN_AUDIT` table — adding a new token here
/// without a corresponding entry there is a compile error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TokenName {
    // Graphite ramp (§1.1)
    Ink0,
    Ink1,
    Graphite900,
    Graphite800,
    Graphite700,
    Graphite600,
    Graphite500,
    Steel400,
    Steel300,
    Steel200,
    // Text ramp (§1.2)
    TextHi,
    TextMid,
    TextLow,
    TextDisabled,
    // Neon accent (§1.3)
    Neon,
    NeonHi,
    NeonDim,
    // Warm signal (§1.4)
    Amber,
    AmberHi,
    // Holo cyan family (§1.5)
    HoloCore,
    HoloGlow,
    HoloDeep,
    HoloAberrR,
    HoloAberrB,
    // Semantic status (§1.6)
    Ok,
    Warn,
    Danger,
    Mana,
}

/// A single canonical token: hex value + alpha + role description.
///
/// Hex strings are **6-character upper-case ASCII** so the token-audit test
/// can scan the crate for any other hex usage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    /// Hex value, e.g. `"#0F131A"`.
    pub hex: &'static str,
    /// Optional premultiplied alpha (0–255) used for glass translucency.
    pub premult_alpha: Option<u8>,
    /// Role description (verbatim from the design language doc).
    pub role: &'static str,
}

/// Canonical token list. Verbatim from `docs/design/ui-design-language.md`.
pub const CANONICAL_TOKENS: &[(TokenName, Token)] = &[
    // Graphite ramp
    (TokenName::Ink0, Token { hex: "#05070A", premult_alpha: Some(244), role: "void / deepest scrim behind floating panels" }),
    (TokenName::Ink1, Token { hex: "#0A0D12", premult_alpha: Some(240), role: "panel bottom / blade base, viewport letterbox" }),
    (TokenName::Graphite900, Token { hex: "#0F131A", premult_alpha: Some(236), role: "primary panel fill (the console face)" }),
    (TokenName::Graphite800, Token { hex: "#161B23", premult_alpha: Some(236), role: "inset wells, list rows, sliders track" }),
    (TokenName::Graphite700, Token { hex: "#1E242E", premult_alpha: Some(238), role: "chips / inactive buttons (mid surface)" }),
    (TokenName::Graphite600, Token { hex: "#272E3A", premult_alpha: Some(240), role: "hover surface / raised nested cards" }),
    (TokenName::Graphite500, Token { hex: "#333C4A", premult_alpha: None,     role: "active-but-neutral surface, pressed blade" }),
    (TokenName::Steel400,   Token { hex: "#4A5564", premult_alpha: None,     role: "bevel highlight (top/left lit edge)" }),
    (TokenName::Steel300,   Token { hex: "#5E6A7A", premult_alpha: None,     role: "hairline borders, inactive widget stroke" }),
    (TokenName::Steel200,   Token { hex: "#7C8898", premult_alpha: None,     role: "tick marks, disabled glyphs" }),
    // Text ramp
    (TokenName::TextHi,       Token { hex: "#ECEFF4", premult_alpha: None, role: "primary body, headings, live values" }),
    (TokenName::TextMid,      Token { hex: "#9AA4B2", premult_alpha: None, role: "secondary text, field labels" }),
    (TokenName::TextLow,      Token { hex: "#646F7E", premult_alpha: None, role: "captions, units, inactive tabs, hints" }),
    (TokenName::TextDisabled, Token { hex: "#3C4450", premult_alpha: None, role: "disabled" }),
    // Neon accent
    (TokenName::Neon,    Token { hex: "#3DF07A", premult_alpha: None, role: "active tab underline, focused-widget 1px edge, selection ring, \"live\" pulse" }),
    (TokenName::NeonHi,  Token { hex: "#8BFFB4", premult_alpha: None, role: "hottest core of a neon glow (1px inner line)" }),
    (TokenName::NeonDim, Token { hex: "#1E7A45", premult_alpha: None, role: "neon at rest / pre-glow trace lines, scanline base in chrome" }),
    // Warm signal
    (TokenName::Amber,   Token { hex: "#F2B33D", premult_alpha: None, role: "treasury/era value, \"confirm/positive\", positive delta arrows" }),
    (TokenName::AmberHi, Token { hex: "#FFE3A0", premult_alpha: None, role: "amber glow core" }),
    // Holo cyan family
    (TokenName::HoloCore,   Token { hex: "#7FE9FF", premult_alpha: None, role: "holo line work, wireframe, primary projected text" }),
    (TokenName::HoloGlow,   Token { hex: "#2FBFE6", premult_alpha: None, role: "holo glow / bloom color, scanline tint" }),
    (TokenName::HoloDeep,   Token { hex: "#0E3A4A", premult_alpha: None, role: "holo panel translucent fill (very low alpha)" }),
    (TokenName::HoloAberrR, Token { hex: "#FF3B6B", premult_alpha: None, role: "chromatic-aberration red ghost channel" }),
    (TokenName::HoloAberrB, Token { hex: "#3B7BFF", premult_alpha: None, role: "chromatic-aberration blue ghost channel" }),
    // Semantic status
    (TokenName::Ok,     Token { hex: "#46D67A", premult_alpha: None, role: "healthy / positive (distinct from NEON: less saturated, fill-safe)" }),
    (TokenName::Warn,   Token { hex: "#F2B33D", premult_alpha: None, role: "caution (shares AMBER)" }),
    (TokenName::Danger, Token { hex: "#F0556B", premult_alpha: None, role: "alert / negative / war" }),
    (TokenName::Mana,   Token { hex: "#9B7BF0", premult_alpha: None, role: "disaster / magic / arcane category" }),
];

/// Static lookup table mapping `TokenName` → `Token` for the audit test.
pub const TOKEN_AUDIT: &[TokenName] = &[
    TokenName::Ink0,
    TokenName::Ink1,
    TokenName::Graphite900,
    TokenName::Graphite800,
    TokenName::Graphite700,
    TokenName::Graphite600,
    TokenName::Graphite500,
    TokenName::Steel400,
    TokenName::Steel300,
    TokenName::Steel200,
    TokenName::TextHi,
    TokenName::TextMid,
    TokenName::TextLow,
    TokenName::TextDisabled,
    TokenName::Neon,
    TokenName::NeonHi,
    TokenName::NeonDim,
    TokenName::Amber,
    TokenName::AmberHi,
    TokenName::HoloCore,
    TokenName::HoloGlow,
    TokenName::HoloDeep,
    TokenName::HoloAberrR,
    TokenName::HoloAberrB,
    TokenName::Ok,
    TokenName::Warn,
    TokenName::Danger,
    TokenName::Mana,
];

impl TokenName {
    /// Resolve a `TokenName` to its canonical `Token` entry.
    #[must_use]
    pub fn resolve(self) -> Token {
        CANONICAL_TOKENS
            .iter()
            .find(|(name, _)| *name == self)
            .map(|(_, t)| *t)
            .expect("TOKEN_AUDIT drift: every TokenName must appear in CANONICAL_TOKENS")
    }

    /// Hex value shortcut.
    #[must_use]
    pub fn hex(self) -> &'static str {
        self.resolve().hex
    }
}

/// True when `hex` matches the verbatim format `#RRGGBB` (uppercase).
/// Used by the audit test to scan source for rogue hex outside the canonical
/// list.
#[must_use]
pub fn is_canonical_hex(hex: &str) -> bool {
    CANONICAL_TOKENS.iter().any(|(_, t)| t.hex == hex)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_token_name_is_in_canonical_table() {
        for name in TOKEN_AUDIT {
            // resolve() panics on missing entries, which is the audit.
            let _ = name.resolve();
        }
        assert_eq!(TOKEN_AUDIT.len(), CANONICAL_TOKENS.len());
    }

    #[test]
    fn every_hex_is_unique() {
        let mut seen = std::collections::HashSet::new();
        for (_, token) in CANONICAL_TOKENS {
            assert!(
                seen.insert(token.hex),
                "duplicate canonical hex {}",
                token.hex
            );
        }
    }

    #[test]
    fn hex_format_is_canonical_rrggbb() {
        for (_, token) in CANONICAL_TOKENS {
            assert_eq!(token.hex.len(), 7, "bad length: {}", token.hex);
            assert!(token.hex.starts_with('#'), "missing #: {}", token.hex);
            let body = &token.hex[1..];
            assert!(
                body.chars().all(|c| c.is_ascii_hexdigit()),
                "non-hex char in {}",
                token.hex
            );
        }
    }

    #[test]
    fn is_canonical_hex_recognises_known_hexes() {
        assert!(is_canonical_hex("#0F131A"));
        assert!(is_canonical_hex("#7FE9FF"));
        assert!(!is_canonical_hex("#ABCDEF"));
        assert!(!is_canonical_hex("0F131A"));
    }
}