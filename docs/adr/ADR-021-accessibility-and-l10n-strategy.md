# ADR-021: Accessibility & Localization Strategy

## Status

Accepted

## Context

The Civis godgame targets a global audience. Two major gaps exist:

1. **Accessibility**: No colorblind support, no high-contrast mode, no screen-reader semantics, no tutorial/onboarding flow.
2. **Localization (L10n)**: Four locale directories exist (`fa/`, `fa-Latn/`, `zh-CN/`, `zh-TW/`) with stub `index.md` files but zero translation infrastructure in code — no string tables, no locale detection, no RTL layout support.

## Decision

### Phase 1 — Foundation (done)

1. Create `crates/i18n` — a dedicated localization crate with:
   - `Locale` enum + detection (`Accept-Language`, OS locale, override)
   - `tr!()` macro for compile-time string lookup
   - JSON string bundles per locale loaded at init
2. Define target locales: English (en), Persian (fa), Persian-Latin (fa-Latn), Simplified Chinese (zh-CN), Traditional Chinese (zh-TW).

### Phase 2 — Accessibility (next sprint)

3. Add colorblind palette modes (deuteranopia, protanopia, tritanopia)
4. Add high-contrast theme toggle
5. Add tutorial/onboarding flow for first-time players
6. Add WCAG AA compliance checklist to PR governance

### Phase 3 — Full L10n rollout (ongoing)

7. Wire iOS system locale detection → `civ-i18n`
8. Translate full UI string table to Persian + Chinese
9. Add RTL layout support for Persian
10. Document i18n workflow in CONTRIBUTING

## Consequences

- **Positive**: All user-facing text flows through `tr!()` → string bundle → UI, enabling incremental translation without code changes
- **Positive**: Colorblind modes improve accessibility for ~8% of male players
- **Negative**: Each new locale adds ~2KB to the binary (JSON bundle)
- **Risk**: RTL layout may require egui fork or custom widget wrapping

## FR Coverage

- FR-CIV-ACCESS-010: Colorblind palette modes (≥3 palettes)
- FR-CIV-ACCESS-020: High-contrast theme
- FR-CIV-L10N-010: String table infrastructure
- FR-CIV-L10N-020: Locale detection
- FR-CIV-L10N-030: Persian translation (≥80% string coverage)
- FR-CIV-L10N-040: Chinese translation (≥80% string coverage)
