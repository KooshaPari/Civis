//! God-tool palette state — FR-CIV-GODTOOL-900.
//!
//! The HUD-side data + query layer for the god-tool palette described in
//! `docs/specs/requirements/FR-CIV-GODTOOL.md` (FR-CIV-GODTOOL-900) and
//! `docs/design/GOD_TOOLS_SANDBOX.md`. This module is **substrate-neutral**:
//! it owns the **current-tool model** and the **per-tool parameter set**,
//! plus the registry that defines each tool. Bevy/Godot/Unreal/web clients
//! read this state and project it onto whatever surface they own (rim
//! cluster, radial deck, in-world brush ring). No rendering, no systems,
//! no `cargo run` — pure data and query fns.
//!
//! ## Contract
//!
//! - **Registry** ([`GodToolRegistry`]) — the catalog of every tool the
//!   palette knows about. Each entry ([`GodToolDef`]) declares its id,
//!   label, glyph, and the **schema** of its per-tool parameters
//!   ([`GodToolParamSpec`]).
//! - **State** ([`GodToolState`]) — the *current* tool + a per-tool
//!   parameter set keyed by [`GodToolId`]. Selecting a tool makes it
//!   active and exposes *its* params. Switching tools swaps the active
//!   params; other tools' params are preserved (so toggling back doesn't
//!   lose the player's edits).
//!
//! ## Acceptance (AC-GT-STATE)
//!
//! - **AC-1:** `select(id)` makes `id` the current tool, and
//!   `current_params()` returns `id`'s params (populated from the
//!   registry's spec defaults if no edit has happened).
//! - **AC-2:** Calling `select(other_id)` swaps the active params to
//!   `other_id`'s set; `other_id`'s previously-stored edits are
//!   preserved on the round-trip.
//!
//! ## Relationship to other crates
//!
//! - The **substrate-facing schema** (the 50-verb catalog with substrate
//!   request kinds) lives in `civ-powers::PowerDef` / `PowerRegistry`
//!   (FR-CIV-GODTOOL-901). This module is the HUD mirror: it carries the
//!   *interactive* state the palette UI cares about, not the substrate
//!   write contract.
//! - Render wiring (Bevy resources / Godot nodes / Unreal components) is
//!   deferred — this PR is the pure-logic data/query layer.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Logical id of one god-tool verb.
///
/// The first nine variants mirror the 8-key Keycap Palette rim
/// (`Q`–`F` from `docs/design/GOD_TOOLS_SANDBOX.md` §2.1) plus the
/// `None` "no tool armed" sentinel. The `Custom(String)` variant is the
/// mod-extensibility escape hatch: a tool whose id is not in this enum
/// is round-tripped through `Custom` (mirroring
/// `civ-powers::PowerId::runtime`, which lands in Phase 5).
///
/// `Custom` is `String`-backed rather than `&'static str` so the data
/// layer accepts mod-registered tools without code changes. The
/// `Ord`/`PartialOrd` ordering is total and stable across runs
/// (declaration order for the named variants, then lexicographic by
/// the inner string for `Custom`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GodToolId {
    /// No tool armed — the palette is up but no verb is live. The
    /// in-world brush ring is hidden when this is current.
    None,
    /// Q — Select / Inspect. Inspect hero panel.
    Select,
    /// W — Raise / Lower (toggle).
    Raise,
    /// E — Material — AdditiveDrop. The headline Powder-Toy mechanic.
    AdditiveDrop,
    /// R — Spawn — Organism. Seed life.
    SpawnOrganism,
    /// T — Terraform God — AddLand / DigOcean (toggle).
    TerraformGod,
    /// A — Time — Pause. Pause-to-inspect.
    Pause,
    /// S — Speed +/–. Adjustable sim rate.
    Speed,
    /// F — Disaster — Lightning. One-click CA energy injection.
    Lightning,
    /// Mod-added or out-of-rim tool. Carries a stable, stable-ordered
    /// string id (typically `<tab>.<verb>`, e.g. `"disaster.meteor"`).
    Custom(String),
}

impl GodToolId {
    /// Stable display label for the tool (the verb shown on the keycap
    /// / card). `Custom(s)` returns `s` verbatim so mod-added tools
    /// can ship their own label through the registry.
    #[must_use]
    pub fn as_label(&self) -> &str {
        match self {
            GodToolId::None => "None",
            GodToolId::Select => "Select",
            GodToolId::Raise => "Raise",
            GodToolId::AdditiveDrop => "AdditiveDrop",
            GodToolId::SpawnOrganism => "SpawnOrganism",
            GodToolId::TerraformGod => "TerraformGod",
            GodToolId::Pause => "Pause",
            GodToolId::Speed => "Speed",
            GodToolId::Lightning => "Lightning",
            GodToolId::Custom(s) => s.as_str(),
        }
    }
}

impl core::fmt::Display for GodToolId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_label())
    }
}

/// The kind of a parameter — drives which input widget the host
/// client renders and which JSON-RPC shape the substrate handler
/// receives.
///
/// Mirrors the `BrushSettings` extensions in
/// `docs/design/brush-tool-system.md` §2.2 plus a couple of UI-only
/// variants (`Bool`, `Text`) for the palette's own state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GodToolParamKind {
    /// Integer scalar (e.g. radius, count).
    Int,
    /// Floating-point scalar (e.g. strength, falloff, sea_level_delta).
    Float,
    /// Boolean toggle (e.g. continuous, symmetry).
    Bool,
    /// Free-form text (e.g. genome id, material id, biome id).
    Text,
    /// Pick-one from a fixed option set (e.g. tool toggle
    /// `raise | lower`, `add_land | dig_ocean`).
    Enum,
}

impl GodToolParamKind {
    /// All kinds, declaration order. Used by the registry to enforce
    /// exhaustive coverage in tests and by host clients that want to
    /// iterate every supported input type.
    pub const ALL: [GodToolParamKind; 5] = [
        GodToolParamKind::Int,
        GodToolParamKind::Float,
        GodToolParamKind::Bool,
        GodToolParamKind::Text,
        GodToolParamKind::Enum,
    ];
}

/// A parameter declaration on a [`GodToolDef`].
///
/// The `default` is the value used by [`GodToolState`] when a tool is
/// first selected (and after `reset_params`). It is the only field a
/// host client needs to render an input — the spec is the contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GodToolParamSpec {
    /// Stable param name (e.g. `"radius"`, `"strength"`, `"continuous"`).
    /// Used as the key in the per-tool param set; must be unique within
    /// a single `GodToolDef`.
    pub name: String,
    /// Human-readable label (e.g. `"Radius"`, `"Strength"`, `"Continuous"`).
    pub label: String,
    /// What kind of input the host should render.
    pub kind: GodToolParamKind,
    /// The default value for this param.
    pub default: GodToolParamValue,
    /// Optional short help text (rendered as a tooltip in the palette
    /// UI per `docs/design/onboarding-qol.md` §2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
}

impl GodToolParamSpec {
    /// Construct a parameter spec from `&'static str` literals. The
    /// `default` value is required; pass it explicitly so the call site
    /// documents the starting state of the tool.
    pub fn new(
        name: impl Into<String>,
        label: impl Into<String>,
        kind: GodToolParamKind,
        default: GodToolParamValue,
    ) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            kind,
            default,
            help: None,
        }
    }

    /// Builder: attach a help string.
    #[must_use]
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
}

/// A single parameter's current value.
///
/// `Owned` (not `&'static`) so the struct is `serde`-friendly: serde's
/// `'de` lifetime cannot outlive `'static`, so borrowed-string fields
/// would break `Deserialize`. The spec/registry still use
/// `&'static str` literals via the [`GodToolParamSpec::new`]
/// constructor — only the live value is owned.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum GodToolParamValue {
    /// Integer value.
    Int(i64),
    /// Floating-point value.
    Float(f64),
    /// Boolean value.
    Bool(bool),
    /// Free-form text value.
    Text(String),
    /// Pick-one value (the option set is implied by the
    /// [`GodToolParamSpec`]; this just stores the chosen option id).
    Enum(String),
}

impl GodToolParamValue {
    /// `true` if this value matches the type expected by the spec.
    /// Used by [`GodToolState::set_param`] to refuse type-mismatched
    /// writes instead of silently coercing.
    #[must_use]
    pub fn matches_kind(&self, kind: GodToolParamKind) -> bool {
        matches!(
            (self, kind),
            (GodToolParamValue::Int(_), GodToolParamKind::Int)
                | (GodToolParamValue::Float(_), GodToolParamKind::Float)
                | (GodToolParamValue::Bool(_), GodToolParamKind::Bool)
                | (GodToolParamValue::Text(_), GodToolParamKind::Text)
                | (GodToolParamValue::Enum(_), GodToolParamKind::Enum)
        )
    }
}

/// A single god-tool definition — id + label + glyph + ordered
/// parameter spec list.
///
/// The `params` are the **schema** (the spec) of the tool's
/// parameters, not their current values. Current values live in
/// [`GodToolState`], keyed by [`GodToolId`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GodToolDef {
    /// Stable id (the primary key).
    pub id: GodToolId,
    /// Human-readable label (e.g. `"Raise"`).
    pub label: String,
    /// Single-character or short glyph rendered on the rim keycap /
    /// deck card (e.g. `"W"`, `"DRP"`).
    pub glyph: String,
    /// Optional secondary verb (e.g. `"Lower"` for Raise/Lower toggle).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label_secondary: Option<String>,
    /// Parameter specs, in declaration order. May be empty for tools
    /// with no parameters (e.g. `Pause`, `Lightning`).
    pub params: Vec<GodToolParamSpec>,
}

impl GodToolDef {
    /// Construct a `GodToolDef` with an explicit id + label + glyph.
    /// Add params with [`GodToolDef::with_param`] / [`GodToolDef::with_params`].
    pub fn new(id: GodToolId, label: impl Into<String>, glyph: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            glyph: glyph.into(),
            label_secondary: None,
            params: Vec::new(),
        }
    }

    /// Set the secondary label.
    #[must_use]
    pub fn with_secondary(mut self, secondary: impl Into<String>) -> Self {
        self.label_secondary = Some(secondary.into());
        self
    }

    /// Append a single param spec. Builder-style; chain multiple calls
    /// to add several params.
    #[must_use]
    pub fn with_param(mut self, spec: GodToolParamSpec) -> Self {
        self.params.push(spec);
        self
    }

    /// Append a list of param specs in order. Equivalent to chaining
    /// [`GodToolDef::with_param`] but convenient for inline `vec!`s.
    #[must_use]
    pub fn with_params(mut self, specs: impl IntoIterator<Item = GodToolParamSpec>) -> Self {
        self.params.extend(specs);
        self
    }
}

/// The god-tool catalog — the set of every tool the palette knows
/// about.
///
/// The registry is **data**, not a singleton. The host client picks
/// the catalog it wants (e.g. the static default 8-rim set, or a
/// mod-merged catalog) and feeds it to [`GodToolState`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GodToolRegistry {
    /// All known tool defs, in declaration order. The order is the
    /// rim order (the first 8 are the keycap palette rim; the rest
    /// appear in the Holocron Deck modal).
    defs: Vec<GodToolDef>,
}

impl GodToolRegistry {
    /// Construct a registry from an iterator of defs.
    #[must_use]
    pub fn new(defs: impl IntoIterator<Item = GodToolDef>) -> Self {
        Self {
            defs: defs.into_iter().collect(),
        }
    }

    /// Borrow the underlying defs slice.
    #[must_use]
    pub fn defs(&self) -> &[GodToolDef] {
        &self.defs
    }

    /// Number of registered tools.
    #[must_use]
    pub fn len(&self) -> usize {
        self.defs.len()
    }

    /// `true` when no tools are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.defs.is_empty()
    }

    /// Look up a tool by id (read-only).
    #[must_use]
    pub fn get(&self, id: &GodToolId) -> Option<&GodToolDef> {
        self.defs.iter().find(|d| &d.id == id)
    }

    /// Iterate every registered tool, in declaration order.
    pub fn iter(&self) -> impl Iterator<Item = &GodToolDef> {
        self.defs.iter()
    }

    /// The default 8-rim keycap palette. Mirrors
    /// `KeycapPalette::default()` from `key_palette.rs` plus
    /// `GodToolId::None` so the registry always has a known
    /// "no tool armed" entry.
    ///
    /// This is the *substrate-neutral* description of the rim; the
    /// substrate-facing 50-verb catalog lives in `civ-powers` and is
    /// a separate concern.
    #[must_use]
    pub fn default_rim() -> Self {
        Self::new([
            GodToolDef::new(GodToolId::None, "None", "·")
                .with_param(GodToolParamSpec::new(
                    "armed",
                    "Armed",
                    GodToolParamKind::Bool,
                    GodToolParamValue::Bool(false),
                )),
            GodToolDef::new(GodToolId::Select, "Select", "Q").with_param(GodToolParamSpec::new(
                "inspect_kind",
                "Inspect kind",
                GodToolParamKind::Enum,
                GodToolParamValue::Enum("entity".into()),
            )),
            GodToolDef::new(GodToolId::Raise, "Raise", "W")
                .with_secondary("Lower")
                .with_params([
                    GodToolParamSpec::new(
                        "radius",
                        "Radius",
                        GodToolParamKind::Float,
                        GodToolParamValue::Float(3.0),
                    )
                    .with_help("Brush footprint radius in cells."),
                    GodToolParamSpec::new(
                        "strength",
                        "Strength",
                        GodToolParamKind::Float,
                        GodToolParamValue::Float(1.0),
                    ),
                    GodToolParamSpec::new(
                        "falloff",
                        "Falloff",
                        GodToolParamKind::Float,
                        GodToolParamValue::Float(0.5),
                    ),
                    GodToolParamSpec::new(
                        "mode",
                        "Mode",
                        GodToolParamKind::Enum,
                        GodToolParamValue::Enum("raise".into()),
                    )
                    .with_help("Toggle: raise vs. lower."),
                ]),
            GodToolDef::new(GodToolId::AdditiveDrop, "AdditiveDrop", "E").with_params([
                GodToolParamSpec::new(
                    "material",
                    "Material",
                    GodToolParamKind::Text,
                    GodToolParamValue::Text("sand".into()),
                ),
                GodToolParamSpec::new(
                    "radius",
                    "Radius",
                    GodToolParamKind::Float,
                    GodToolParamValue::Float(2.0),
                ),
                GodToolParamSpec::new(
                    "drop_height",
                    "Drop height",
                    GodToolParamKind::Float,
                    GodToolParamValue::Float(8.0),
                ),
                GodToolParamSpec::new(
                    "continuous",
                    "Continuous",
                    GodToolParamKind::Bool,
                    GodToolParamValue::Bool(true),
                ),
            ]),
            GodToolDef::new(GodToolId::SpawnOrganism, "SpawnOrganism", "R").with_params([
                GodToolParamSpec::new(
                    "genome",
                    "Genome",
                    GodToolParamKind::Text,
                    GodToolParamValue::Text("default".into()),
                ),
                GodToolParamSpec::new(
                    "count",
                    "Count",
                    GodToolParamKind::Int,
                    GodToolParamValue::Int(1),
                ),
                GodToolParamSpec::new(
                    "age",
                    "Age",
                    GodToolParamKind::Float,
                    GodToolParamValue::Float(0.0),
                ),
            ]),
            GodToolDef::new(GodToolId::TerraformGod, "TerraformGod", "T")
                .with_secondary("DigOcean")
                .with_params([
                    GodToolParamSpec::new(
                        "mode",
                        "Mode",
                        GodToolParamKind::Enum,
                        GodToolParamValue::Enum("add_land".into()),
                    ),
                    GodToolParamSpec::new(
                        "size",
                        "Size",
                        GodToolParamKind::Int,
                        GodToolParamValue::Int(8),
                    ),
                    GodToolParamSpec::new(
                        "strength",
                        "Strength",
                        GodToolParamKind::Float,
                        GodToolParamValue::Float(2.0),
                    ),
                ]),
            GodToolDef::new(GodToolId::Pause, "Pause", "A"),
            GodToolDef::new(GodToolId::Speed, "Speed", "S").with_params([GodToolParamSpec::new(
                "multiplier",
                "Multiplier",
                GodToolParamKind::Float,
                GodToolParamValue::Float(1.0),
            )]),
            GodToolDef::new(GodToolId::Lightning, "Lightning", "F").with_params([
                GodToolParamSpec::new(
                    "voltage",
                    "Voltage",
                    GodToolParamKind::Float,
                    GodToolParamValue::Float(1.0),
                ),
            ]),
        ])
    }
}

/// The god-tool palette state — current tool + per-tool parameter
/// set.
///
/// The state model is the **data side** of the FR-CIV-GODTOOL-900
/// contract:
///
/// - The `current` field names the active tool; the host client reads
///   it to know which keycap glows teal and which tool card is
///   pressed-in.
/// - The `params` field is a per-tool map of `param_name -> value`.
///   Each tool's params are populated on first [`GodToolState::select`]
///   from the registry's spec defaults; subsequent edits live in this
///   map and are preserved when the player switches tools and back.
///
/// Switching tools **swaps** the active params (the API returns the
/// new tool's params, not the old tool's). Other tools' params are
/// untouched in the map — toggling back to a tool you previously
/// edited returns the edited values, not the spec defaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GodToolState {
    /// The currently-armed tool. `GodToolId::None` means "no tool
    /// armed" — the in-world brush ring is hidden.
    current: GodToolId,
    /// Per-tool parameter values, keyed by tool id. Populated lazily
    /// on first `select` (using the registry's spec defaults), and
    /// updated by [`GodToolState::set_param`].
    params: BTreeMap<GodToolId, BTreeMap<String, GodToolParamValue>>,
}

impl GodToolState {
    /// Build an empty state with no current tool.
    #[must_use]
    pub fn new() -> Self {
        Self {
            current: GodToolId::None,
            params: BTreeMap::new(),
        }
    }

    /// Build a state pre-populated from `registry`. The current tool
    /// is `GodToolId::None`; tool params are populated on first
    /// `select`.
    #[must_use]
    pub fn with_registry(registry: &GodToolRegistry) -> Self {
        // No params are pre-populated; `select` does the work lazily
        // so the state struct doesn't carry spec defaults for tools
        // the player never touches.
        let _ = registry;
        Self::new()
    }

    /// The currently-armed tool id.
    #[must_use]
    pub fn current(&self) -> &GodToolId {
        &self.current
    }

    /// Look up the def of the currently-armed tool in `registry`.
    /// Returns `None` when either no tool is armed or the armed tool
    /// is not in the registry.
    #[must_use]
    pub fn current_def<'a>(&self, registry: &'a GodToolRegistry) -> Option<&'a GodToolDef> {
        registry.get(&self.current)
    }

    /// The current tool's params (in declaration order from the
    /// registry's spec). The returned vec pairs each spec with its
    /// current value, so a host client can render a labelled form
    /// without re-querying the registry for labels.
    ///
    /// Returns an empty vec when no tool is armed or the current
    /// tool is not in the registry. Tools with no declared params
    /// also return an empty vec.
    ///
    /// Values are resolved by: `self.params[current][name]`, falling
    /// back to the spec's `default`. Calling `select` populates the
    /// lazy map so subsequent reads hit the explicit-stored path;
    /// the fallback is what makes a freshly-constructed state still
    /// return useful defaults.
    #[must_use]
    pub fn current_params<'a>(
        &'a self,
        registry: &'a GodToolRegistry,
    ) -> Vec<(&'a GodToolParamSpec, &'a GodToolParamValue)> {
        let Some(def) = registry.get(&self.current) else {
            return Vec::new();
        };
        let values = self.params.get(&self.current);
        def.params
            .iter()
            .map(move |spec| {
                let value: &'a GodToolParamValue = values
                    .and_then(|m| m.get(&spec.name))
                    .unwrap_or(&spec.default);
                (spec, value)
            })
            .collect()
    }

    /// The current tool's params as owned `(spec, value)` pairs. Use
    /// this when the host client needs a `'static`-friendly
    /// `Vec<(GodToolParamSpec, GodToolParamValue)>` (e.g. for
    /// `serde` round-trip or `wasm_bindgen` export).
    #[must_use]
    pub fn current_params_owned(
        &self,
        registry: &GodToolRegistry,
    ) -> Vec<(GodToolParamSpec, GodToolParamValue)> {
        let Some(def) = registry.get(&self.current) else {
            return Vec::new();
        };
        def.params
            .iter()
            .map(|spec| {
                let value = self
                    .params
                    .get(&self.current)
                    .and_then(|m| m.get(&spec.name))
                    .cloned()
                    .unwrap_or_else(|| spec.default.clone());
                (spec.clone(), value)
            })
            .collect()
    }

    /// Select a tool — makes it the current tool and ensures its
    /// params are populated. If `id` has never been selected, its
    /// params are initialised from the registry's spec defaults.
    ///
    /// Returns `true` if the current tool changed.
    ///
    /// Selecting an unknown id (`registry.get(&id).is_none()`) is a
    /// **no-op** that returns `false`: the state never lies about
    /// which tool is armed.
    pub fn select(&mut self, registry: &GodToolRegistry, id: GodToolId) -> bool {
        if registry.get(&id).is_none() {
            return false;
        }
        // Lazy-populate the param map from the spec defaults.
        self.params.entry(id.clone()).or_insert_with(|| {
            let Some(def) = registry.get(&id) else {
                return BTreeMap::new();
            };
            def.params
                .iter()
                .map(|s| (s.name.clone(), s.default.clone()))
                .collect()
        });
        let changed = self.current != id;
        self.current = id;
        changed
    }

    /// Set a single parameter on `id`. Returns `true` if the value
    /// was written.
    ///
    /// The write is rejected (returns `false`) if:
    /// - `id` is not in the registry.
    /// - `name` is not a declared param of `id`.
    /// - `value` does not match the declared param's
    ///   [`GodToolParamKind`].
    ///
    /// Selecting `id` *before* editing is not required — `set_param`
    /// will lazy-init the param map from the spec defaults on the
    /// fly. But callers that want the edit to also become "active"
    /// should call [`GodToolState::select`] first.
    pub fn set_param(
        &mut self,
        registry: &GodToolRegistry,
        id: &GodToolId,
        name: &str,
        value: GodToolParamValue,
    ) -> bool {
        let Some(def) = registry.get(id) else {
            return false;
        };
        let Some(spec) = def.params.iter().find(|s| s.name == name) else {
            return false;
        };
        if !value.matches_kind(spec.kind) {
            return false;
        }
        let map = self.params.entry(id.clone()).or_insert_with(|| {
            def.params
                .iter()
                .map(|s| (s.name.clone(), s.default.clone()))
                .collect()
        });
        map.insert(name.to_owned(), value);
        true
    }

    /// Read a single parameter on `id`. Returns `None` if the tool
    /// or the param is unknown.
    #[must_use]
    pub fn param(
        &self,
        _registry: &GodToolRegistry,
        id: &GodToolId,
        name: &str,
    ) -> Option<&GodToolParamValue> {
        let map = self.params.get(id)?;
        map.get(name)
    }

    /// Reset a single tool's params to the registry's spec defaults.
    /// No-op if `id` is not in the registry.
    pub fn reset_params(&mut self, registry: &GodToolRegistry, id: &GodToolId) {
        if let Some(def) = registry.get(id) {
            let entry = self.params.entry(id.clone()).or_default();
            entry.clear();
            for s in &def.params {
                entry.insert(s.name.clone(), s.default.clone());
            }
        }
    }

    /// Reset every tool's params. The `current` tool is left
    /// unchanged.
    pub fn reset_all_params(&mut self, registry: &GodToolRegistry) {
        for def in registry.iter() {
            self.reset_params(registry, &def.id);
        }
    }
}

impl Default for GodToolState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a fresh registry + state for each test.
    fn fresh() -> (GodToolRegistry, GodToolState) {
        (GodToolRegistry::default_rim(), GodToolState::new())
    }

    /// AC-1 (FR-CIV-GODTOOL-900): selecting a tool sets it active and
    /// exposes its params.
    #[test]
    fn selecting_a_tool_sets_it_active_and_exposes_its_params() {
        let (reg, mut state) = fresh();
        assert_eq!(state.current(), &GodToolId::None);
        // Initially: no current tool → empty params vec.
        assert!(state.current_params(&reg).is_empty());

        // Select Raise. State flips to Raise.
        assert!(state.select(&reg, GodToolId::Raise));
        assert_eq!(state.current(), &GodToolId::Raise);

        // Raise's spec has 4 params; current_params returns all 4
        // with their default values.
        let params = state.current_params(&reg);
        assert_eq!(params.len(), 4);
        let names: Vec<&str> = params.iter().map(|(s, _)| s.name.as_str()).collect();
        assert_eq!(names, vec!["radius", "strength", "falloff", "mode"]);
        // Default for `mode` is the `raise` enum option.
        assert!(matches!(
            params.iter().find(|(s, _)| s.name == "mode").unwrap().1,
            GodToolParamValue::Enum(opt) if opt == "raise"
        ));
        // Default for `radius` is 3.0.
        assert!(matches!(
            params.iter().find(|(s, _)| s.name == "radius").unwrap().1,
            GodToolParamValue::Float(v) if (*v - 3.0).abs() < f64::EPSILON
        ));
    }

    /// AC-2 (FR-CIV-GODTOOL-900): switching tools swaps the active
    /// params. Other tools' params are preserved across the swap.
    #[test]
    fn switching_tools_swaps_params() {
        let (reg, mut state) = fresh();
        // Arm Raise, edit its `radius` to 9.5, edit its `mode` to
        // "lower" (Raise's secondary verb).
        assert!(state.select(&reg, GodToolId::Raise));
        assert!(state.set_param(
            &reg,
            &GodToolId::Raise,
            "radius",
            GodToolParamValue::Float(9.5),
        ));
        assert!(state.set_param(
            &reg,
            &GodToolId::Raise,
            "mode",
            GodToolParamValue::Enum("lower".into()),
        ));
        // The edits are visible on the current tool.
        let cur = state.current_params(&reg);
        assert!(matches!(
            cur.iter().find(|(s, _)| s.name == "radius").unwrap().1,
            GodToolParamValue::Float(v) if (*v - 9.5).abs() < f64::EPSILON
        ));
        assert!(matches!(
            cur.iter().find(|(s, _)| s.name == "mode").unwrap().1,
            GodToolParamValue::Enum(opt) if opt == "lower"
        ));

        // Switch to AdditiveDrop — the headline Powder-Toy tool.
        assert!(state.select(&reg, GodToolId::AdditiveDrop));
        assert_eq!(state.current(), &GodToolId::AdditiveDrop);
        // AdditiveDrop has 4 params: material, radius, drop_height,
        // continuous. They are the spec defaults — not Raise's.
        let swapped = state.current_params(&reg);
        let names: Vec<&str> = swapped.iter().map(|(s, _)| s.name.as_str()).collect();
        assert_eq!(names, vec!["material", "radius", "drop_height", "continuous"]);
        // The new current has its own `radius` default (2.0), NOT
        // Raise's edited 9.5.
        assert!(matches!(
            swapped.iter().find(|(s, _)| s.name == "radius").unwrap().1,
            GodToolParamValue::Float(v) if (*v - 2.0).abs() < f64::EPSILON
        ));
        // The new current has its own `material` text default.
        assert!(matches!(
            swapped.iter().find(|(s, _)| s.name == "material").unwrap().1,
            GodToolParamValue::Text(t) if t == "sand"
        ));

        // Critical: Raise's edits are still in the state. Switching
        // back returns the edited values, not the spec defaults.
        assert!(state.select(&reg, GodToolId::Raise));
        let raised = state.current_params(&reg);
        assert!(matches!(
            raised.iter().find(|(s, _)| s.name == "radius").unwrap().1,
            GodToolParamValue::Float(v) if (*v - 9.5).abs() < f64::EPSILON
        ));
        assert!(matches!(
            raised.iter().find(|(s, _)| s.name == "mode").unwrap().1,
            GodToolParamValue::Enum(opt) if opt == "lower"
        ));
    }

    /// Selecting an unknown id is a no-op: the state never lies
    /// about which tool is armed.
    #[test]
    fn select_unknown_id_is_a_noop() {
        let (reg, mut state) = fresh();
        let bogus = GodToolId::Custom("does.not.exist".into());
        assert!(!state.select(&reg, bogus.clone()));
        assert_eq!(state.current(), &GodToolId::None);
    }

    /// `set_param` rejects unknown tools, unknown param names, and
    /// type mismatches.
    #[test]
    fn set_param_validates_inputs() {
        let (reg, mut state) = fresh();
        state.select(&reg, GodToolId::Raise);

        // Unknown tool.
        assert!(!state.set_param(
            &reg,
            &GodToolId::Custom("nope".into()),
            "radius",
            GodToolParamValue::Float(1.0),
        ));
        // Unknown param name.
        assert!(!state.set_param(
            &reg,
            &GodToolId::Raise,
            "no_such_param",
            GodToolParamValue::Float(1.0),
        ));
        // Kind mismatch: `radius` is Float, not Int.
        assert!(!state.set_param(
            &reg,
            &GodToolId::Raise,
            "radius",
            GodToolParamValue::Int(7),
        ));
        // Kind mismatch: `mode` is Enum, not Text.
        assert!(!state.set_param(
            &reg,
            &GodToolId::Raise,
            "mode",
            GodToolParamValue::Text("raise".into()),
        ));
        // Valid: a Float into `radius` is accepted.
        assert!(state.set_param(
            &reg,
            &GodToolId::Raise,
            "radius",
            GodToolParamValue::Float(5.0),
        ));
        assert!(matches!(
            state.param(&reg, &GodToolId::Raise, "radius"),
            Some(GodToolParamValue::Float(v)) if (*v - 5.0).abs() < f64::EPSILON
        ));
    }

    /// `reset_params` snaps a tool back to the spec defaults; the
    /// current tool is unaffected.
    #[test]
    fn reset_params_returns_to_defaults() {
        let (reg, mut state) = fresh();
        state.select(&reg, GodToolId::Raise);
        state.set_param(
            &reg,
            &GodToolId::Raise,
            "radius",
            GodToolParamValue::Float(11.0),
        );
        // Edit is visible.
        assert!(matches!(
            state.param(&reg, &GodToolId::Raise, "radius"),
            Some(GodToolParamValue::Float(v)) if (*v - 11.0).abs() < f64::EPSILON
        ));
        // Reset.
        state.reset_params(&reg, &GodToolId::Raise);
        assert!(matches!(
            state.param(&reg, &GodToolId::Raise, "radius"),
            Some(GodToolParamValue::Float(v)) if (*v - 3.0).abs() < f64::EPSILON
        ));
        // Current tool unchanged.
        assert_eq!(state.current(), &GodToolId::Raise);
    }

    /// The default rim registry contains 9 entries (None + the 8
    /// keycap palette verbs).
    #[test]
    fn default_rim_has_nine_entries() {
        let reg = GodToolRegistry::default_rim();
        assert_eq!(reg.len(), 9);
        assert!(reg.get(&GodToolId::None).is_some());
        assert!(reg.get(&GodToolId::Select).is_some());
        assert!(reg.get(&GodToolId::Raise).is_some());
        assert!(reg.get(&GodToolId::AdditiveDrop).is_some());
        assert!(reg.get(&GodToolId::SpawnOrganism).is_some());
        assert!(reg.get(&GodToolId::TerraformGod).is_some());
        assert!(reg.get(&GodToolId::Pause).is_some());
        assert!(reg.get(&GodToolId::Speed).is_some());
        assert!(reg.get(&GodToolId::Lightning).is_some());
        // Unknown id returns None.
        assert!(reg.get(&GodToolId::Custom("nope".into())).is_none());
    }

    /// `GodToolParamValue::matches_kind` is the source of truth for
    /// type-checking; spot-check every variant.
    #[test]
    fn param_value_matches_kind() {
        for kind in GodToolParamKind::ALL {
            let value = match kind {
                GodToolParamKind::Int => GodToolParamValue::Int(0),
                GodToolParamKind::Float => GodToolParamValue::Float(0.0),
                GodToolParamKind::Bool => GodToolParamValue::Bool(false),
                GodToolParamKind::Text => GodToolParamValue::Text(String::new()),
                GodToolParamKind::Enum => GodToolParamValue::Enum(String::new()),
            };
            assert!(value.matches_kind(kind), "{kind:?} should match itself");
        }
        // Cross-kind mismatches.
        assert!(!GodToolParamValue::Int(0).matches_kind(GodToolParamKind::Float));
        assert!(!GodToolParamValue::Text("x".into()).matches_kind(GodToolParamKind::Enum));
    }
}
