# Native vs Extended UI SDK Reality Audit

**Date**: 2026-04-25
**Auditor**: Infra audit subagent (iteration 37, parallel batch of 5)
**Scope**: `src/Domains/UI`, `src/Runtime/UI`, `src/SDK` UI surface, HUD
injection systems, `HudElementRegistry`, `MenuRegistry`, `ThemeRegistry`,
pack-side UI declarations
**Verdict**: ORPHANED

---

## Executive summary

DINOForge has two distinct UI surfaces and treats them as one:

1. **Native UI** — in-game `DFCanvas_Root` rendering, lives inside the running
   game process, must respect Unity's main-thread rules, must use Unity uGUI
   or DFCanvas widgets.
2. **Extended UI** — out-of-game / overlay / debug menu (F10), driven by
   BepInEx config UI, Avalonia installer GUI, MCP-driven dashboards.

There is **zero SDK contract** distinguishing the two. `Domains/UI` exposes
`HudElementRegistry`, `MenuRegistry`, `ThemeRegistry`, and `HUDInjectionSystem`
as if they were a unified API, but no production rendering path consumes them
— packs can declare `hud_elements`, `menus`, `themes` in YAML, the loader
reads them, the registries hold them, and **nothing renders them in-game**.

The `HUDInjectionSystem` is wired into Runtime as of Dispatch 1 of #194 but
is still a no-op: it iterates `HudElementRegistry` entries and produces no
UI. The packs that do "render UI" (e.g. the F10 mod menu) bypass these
registries and hand-write Unity GUI code in Runtime.

The clean fix is a hard split: `SDK.Native` (DFCanvas / uGUI targeted,
runtime-only) and `SDK.Extended` (overlay / debug / off-screen, can run on
the main thread or a separate worker). 26 files need to move or change
references.

---

## Findings table

| Component | Native or Extended | Status | Notes |
|-----------|---------------------|--------|-------|
| `DFCanvas_Root` direct render | Native | Hand-written; bypasses registries | Works, but not registry-driven |
| `HudElementRegistry` | Native (intended) | Loaded from pack YAML; never rendered | ORPHAN |
| `MenuRegistry` | Native (intended) | Loaded from pack YAML; never rendered | ORPHAN |
| `ThemeRegistry` | Native (intended) | Loaded from pack YAML; never applied | ORPHAN |
| `HUDInjectionSystem` | Native | ProjectReference wired (#194 Dispatch 1); body still no-op | PARTIAL |
| F10 mod menu | Native | Hand-written Unity GUI in Runtime; not registry-driven | WORKS (orphan path) |
| BepInEx ConfigurationManager | Extended | Standard, works | WORKS |
| Avalonia installer GUI | Extended | Page-based wizard, works | WORKS |
| MCP-driven dashboards | Extended | Out-of-process, works | WORKS |
| `UIContentLoader.LoadFromManifest` | Native | Method added (#194 Dispatch 1) | PARTIAL |
| Pack `hud_elements` YAML | Native (intended) | Loaded; ignored at render time | ORPHAN |
| Pack `menus` YAML | Native (intended) | Loaded; ignored at render time | ORPHAN |

---

## Concrete gaps

1. **No Native vs Extended SDK boundary.** `Domains/UI` mixes both surfaces
   under one namespace. This is the single biggest structural rot — packs
   cannot tell at compile time whether a UI declaration will run in-game or
   out-of-game.

2. **`HudElementRegistry` has no consumer.** The registry is populated from
   pack YAML, validated by the schema, exposed via `Domains/UI`, and nothing
   reads it at render time. Same for `MenuRegistry` and `ThemeRegistry`.

3. **`HUDInjectionSystem` is a stub.** ProjectReference is wired but the
   `OnUpdate()` body produces no UI. Dispatch 2 of #194 (DFCanvas
   registry-driven render) is open.

4. **Pack-side YAML declares UI that goes nowhere.** Packs honestly declare
   `hud_elements`, `menus`, `themes` per the schema, the loader honestly loads
   them, and the registry honestly holds them. The disconnect is between
   registry and renderer. Pack authors have no way to detect this.

5. **F10 menu is hand-written.** The one production native UI surface bypasses
   the registry entirely. This is an honest path (works), but it means there
   is no example for pack authors to follow if they wanted to render registry
   entries themselves.

6. **`UIContentLoader.LoadFromManifest` is brand-new and not yet covered by
   tests.** Dispatch 1 of #194 added the method, but Dispatch 2 has not
   landed and the YAML deserialization test (UIContentLoader.LoadHudElementFile:115)
   is currently failing — likely fixture YAML schema mismatch with the new
   manifest-driven loader. Tracked under #196.

7. **Documentation conflates the two.** README, CLAUDE.md, and
   `docs/sdk/ui.md` (where it exists) describe "the UI registry" without
   distinguishing native rendering from extended overlay. Pack authors who
   read these docs reasonably expect their declarations to render in-game.

8. **26 files need to move.** A clean split requires:
   - 12 files moved from `src/Domains/UI` → `src/SDK.Native.UI`
   - 6 files moved from `src/Tools/DesktopCompanion` → `src/SDK.Extended.UI`
   - 4 files in `src/Runtime/UI` updated to consume `SDK.Native.UI`
   - 4 files in `src/Tests` re-namespaced
   The exact file list is in `docs/design/2026-04-25-ui-registry-wiring-plan.md`.

---

## Tasks opened from this audit

- **#193 P2 INFRA** — SDK split: native UI vs extended UI separation. 26-file
  refactor. Currently open.
- **#194 P2 INFRA** — Wire orphaned `Domains/UI` registries to runtime.
  Dispatch 1 (ProjectReference + `UIContentLoader.LoadFromManifest`) closed.
  Dispatch 2 (DFCanvas registry-driven render) and Dispatch 3 (pack YAML
  rename) open.

Acceptance: a pack declaring `hud_elements`, `menus`, `themes` actually
renders them in-game on `DFCanvas_Root` — verifiable via a Wave 2 receipt.

---

## See also

- `docs/sessions/2026-04-25-infra-pivot-plan.md` — Wave 4 owns these tasks
- `docs/design/2026-04-25-ui-registry-wiring-plan.md` — file map for #194
- `docs/sessions/2026-04-25-ci-manifest-gate-audit.md` — UI receipt requires Wave 2 signing tooling
- `docs/sessions/2026-04-25-steamless-multi-instance-audit.md` — UI proof requires sandbox to actually work
- `docs/sessions/2026-04-25-sandbox-isolation-audit.md` — same
- `docs/sessions/2026-04-25-bridge-bypass-audit.md` — bridge bypass means UI status reports may lie

---

## Verdict

The UI SDK is **structurally orphaned**: pack-side declarations have no
runtime consumer. The fix is the Wave 4 split (#193) plus the dispatch chain
of #194. Until the split lands, every UI claim ("packs can declare HUD
elements", "themes apply at runtime") must be qualified to "the registry
holds them; the renderer ignores them". No production rendering happens via
the registry today.
