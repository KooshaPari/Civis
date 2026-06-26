# ADR-021: Accessibility & L10n Strategy

## Status
Approved

## Context
Civis targets a global audience with diverse accessibility and language needs. Currently:
- Four locale directories exist as stubs (`fa/`, `fa-Latn/`, `zh-CN/`, `zh-TW/`) with no translations
- No string extraction/infrastructure exists
- No colorblind/tutorial/screen-reader support
- No ADR governs the approach

## Decision
Adopt a phased strategy:

### Phase 1 — Foundation (current)
1. `crates/i18n/` with `Locale` enum + `tr!()` macro + JSON string bundles
2. `.ci/` quality composites for reusable CI actions
3. `ADR-021` strategy document

### Phase 2 — Translations
1. Full Persian (fa) RTL layout + Arabic-script rendering
2. Chinese (zh-CN, zh-TW) CJK font fallback
3. Complete UI string tables (68+ keys)

### Phase 3 — Accessibility
1. Colorblind palette modes (deuteranopia, protanopia, tritanopia)
2. Tutorial/onboarding flow with first-run detection
3. High-contrast mode + WCAG AA compliance

### Phase 4 — Screen Reader
1. ARIA-compatible UI tree
2. Screen-reader announcement queue
3. Keyboard-navigation parity

## Consequences
- Translation effort shifts from ad-hoc to structured
- `crates/i18n/` becomes a dependency for all UI crates
- New PRs must include `tr!()` for user-facing strings
- RTL support requires layout-agnostic UI framework testing
