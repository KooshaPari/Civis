//! **FR-CIV-ASSET-QUAL-001** — Inline template validation for `.svg.j2`
//! pre-commit checks.
//!
//! The spec (`docs/specs/CIV-0600-2d-asset-pipeline-spec.md` §2.6) lists
//! eight rules. The Python pre-commit hook (`python3 scripts/validate_templates.py`)
//! runs the full suite. This module exposes a *subset* — three fast,
//! dependency-free textual checks — usable from Rust callers (the asset
//! pipeline exporter + the engine's `civ-build` step) without pulling in
//! an XML parser. The Python hook remains the source of truth for full
//! coverage.
//!
//! Rules implemented here:
//!
//! 1. ViewBox present on the root `<svg>` element.
//! 2. No `<image>` elements with raster `data:` URIs.
//! 3. No `<script>` elements.

/// A named template rule for diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateRule {
    /// Rule 2 — the `<svg>` root element has a `viewBox` attribute.
    ViewBoxPresent,
    /// Rule 4 — no `<image>` element with a raster `data:` URI.
    NoRasterDataUri,
    /// Rule 5 — no `<script>` element anywhere in the document.
    NoScriptElement,
}

impl TemplateRule {
    /// Stable identifier (matches the rule number in §2.6).
    #[must_use]
    pub const fn id(self) -> u8 {
        match self {
            TemplateRule::ViewBoxPresent => 2,
            TemplateRule::NoRasterDataUri => 4,
            TemplateRule::NoScriptElement => 5,
        }
    }
}

/// One violation produced by [`validate_svg_template`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateViolation {
    /// Which rule was violated.
    pub rule: TemplateRule,
    /// Human-readable reason.
    pub message: String,
}

/// **FR-CIV-ASSET-QUAL-001** — Run the textual subset of the §2.6 template
/// validation rules. Returns an empty `Vec` when the template passes every
/// implemented check. The caller decides whether to block on violations
/// (the Python hook treats any violation as a hard fail).
#[must_use]
pub fn validate_svg_template(svg_text: &str) -> Vec<TemplateViolation> {
    let mut violations = Vec::new();
    if !has_root_viewbox(svg_text) {
        violations.push(TemplateViolation {
            rule: TemplateRule::ViewBoxPresent,
            message: "root <svg> element is missing a viewBox attribute".into(),
        });
    }
    if has_image_with_data_uri(svg_text) {
        violations.push(TemplateViolation {
            rule: TemplateRule::NoRasterDataUri,
            message: "<image> element with embedded raster data URI is forbidden".into(),
        });
    }
    if has_script_element(svg_text) {
        violations.push(TemplateViolation {
            rule: TemplateRule::NoScriptElement,
            message: "<script> element is forbidden in asset templates".into(),
        });
    }
    violations
}

fn has_root_viewbox(s: &str) -> bool {
    // Find the first `<svg` and inspect attributes until the `>` or `/>`.
    let lower = s.to_ascii_lowercase();
    let Some(start) = lower.find("<svg") else {
        return false;
    };
    let after_svg = &s[start + 4..];
    let lower_after = &lower[start + 4..];
    let end_rel = lower_after
        .find('>')
        .unwrap_or(usize::MAX)
        .min(lower_after.find("/>").unwrap_or(usize::MAX));
    if end_rel == usize::MAX {
        return false;
    }
    let attrs = &after_svg[..end_rel];
    attrs.to_ascii_lowercase().contains("viewbox=")
}

fn has_image_with_data_uri(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    let mut idx = 0usize;
    while let Some(rel) = lower[idx..].find("<image") {
        let abs = idx + rel;
        let line_end = lower[abs..]
            .find('>')
            .map(|p| abs + p)
            .unwrap_or(lower.len());
        let tag = &lower[abs..=line_end.min(lower.len().saturating_sub(1))];
        if tag.contains("data:image") {
            return true;
        }
        idx = line_end + 1;
    }
    false
}

fn has_script_element(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    lower.contains("<script")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-CIV-ASSET-QUAL-001 — a minimal compliant template passes.
    #[test]
    fn well_formed_template_passes() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16">
            <g id="base"></g>
            <g id="texture"></g>
            <g id="nation_zone"></g>
            <g id="icon_layer"></g>
            <g id="population_marker"></g>
        </svg>"#;
        assert!(validate_svg_template(svg).is_empty());
    }

    /// FR-CIV-ASSET-QUAL-001 — missing viewBox is reported.
    #[test]
    fn missing_viewbox_is_a_violation() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg">
            <g id="base"></g>
        </svg>"#;
        let v = validate_svg_template(svg);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].rule, TemplateRule::ViewBoxPresent);
        assert_eq!(v[0].rule.id(), 2);
    }

    /// FR-CIV-ASSET-QUAL-001 — an embedded raster data URI trips rule 4.
    #[test]
    fn embedded_raster_data_uri_is_a_violation() {
        let svg = r#"<svg viewBox="0 0 1 1">
            <image href="data:image/png;base64,AAAA" />
        </svg>"#;
        let v = validate_svg_template(svg);
        assert!(v.iter().any(|x| x.rule == TemplateRule::NoRasterDataUri));
    }

    /// FR-CIV-ASSET-QUAL-001 — `<script>` tags are forbidden.
    #[test]
    fn script_element_is_a_violation() {
        let svg = r#"<svg viewBox="0 0 1 1">
            <script>alert(1)</script>
        </svg>"#;
        let v = validate_svg_template(svg);
        assert!(v.iter().any(|x| x.rule == TemplateRule::NoScriptElement));
    }
}
