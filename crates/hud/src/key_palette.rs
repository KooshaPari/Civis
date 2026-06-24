//! 8-key Keycap Palette (the rim cluster).
//!
//! Per `docs/design/SAVELOAD_HUD_PLAN.md` §4.2 — verbatim recipe from
//! `docs/design/GOD_TOOLS_SANDBOX.md` §2.2. The palette is **substrate
//! neutral**: each keycap binds a `verb` string the host client maps to its
//! existing wire call (`power.activate`, `sim.set_speed`, `inspect.probe`).
//!
//! The Keycap Palette **never** mutates the substrate directly — every
//! activation routes through the JSON-RPC catalog (AC-HUD-7).

use serde::{Deserialize, Serialize};

/// Logical id of one keycap. Used by host clients to bind inputs and route
/// the `verb` to a wire method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyId {
    /// 1 — Select / Inspect → `power.activate("Inspect.Probe")`.
    Q,
    /// 2 — Raise / Lower toggle → `power.activate("Terrain.Raise")`.
    W,
    /// 3 — Material — AdditiveDrop → `power.activate("Material.AdditiveDrop")`.
    E,
    /// 4 — Spawn — Organism → `power.activate("Life.SpawnOrganism")`.
    R,
    /// 5 — Terraform God — AddLand / DigOcean toggle → `power.activate("Terrain.AddLand")`.
    T,
    /// 6 — Time — Pause → `sim.set_speed{0}` (WARN dot when paused).
    A,
    /// 7 — Speed +/– → rotates `GameSpeed.multiplier ∈ {1, 2, 5, 10}`.
    S,
    /// 8 — Disaster — Lightning → `power.activate("Disaster.Lightning")`.
    F,
}

/// One keycap definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeycapDef {
    pub id: KeyId,
    /// Single uppercase character the keycap glyph renders (e.g. `"Q"`).
    pub glyph: &'static str,
    /// One-word verb shown on the keycap (`Speed`, `Pause`, `Lightning`…).
    pub verb: &'static str,
    /// Optional secondary verb (e.g. `"Raise/Lower"` for W).
    pub verb_secondary: Option<&'static str>,
    /// Whether this keycap is the currently-active verb (1px `HOLO_GLOW`
    /// edge + 3px halo per §4.2).
    pub active: bool,
}

impl KeycapDef {
    pub const fn new(
        id: KeyId,
        glyph: &'static str,
        verb: &'static str,
        verb_secondary: Option<&'static str>,
    ) -> Self {
        Self {
            id,
            glyph,
            verb,
            verb_secondary,
            active: false,
        }
    }
}

/// The 8-key Keycap Palette — the **rim cluster** shown in every HUD state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeycapPalette {
    pub keys: [KeycapDef; 8],
}

impl Default for KeycapPalette {
    fn default() -> Self {
        Self {
            keys: [
                KeycapDef::new(KeyId::Q, "Q", "Select", None),
                KeycapDef::new(KeyId::W, "W", "Raise", Some("Lower")),
                KeycapDef::new(KeyId::E, "E", "Material", None),
                KeycapDef::new(KeyId::R, "R", "Spawn", None),
                KeycapDef::new(KeyId::T, "T", "Land", Some("Ocean")),
                KeycapDef::new(KeyId::A, "A", "Pause", None),
                KeycapDef::new(KeyId::S, "S", "Speed", None),
                KeycapDef::new(KeyId::F, "F", "Lightning", None),
            ],
        }
    }
}

/// Default palette constant — frozen so other crates can `const`-compare.
pub const KEYCAP_PALETTE_DEFAULT: KeycapPalette = KeycapPalette {
    keys: [
        KeycapDef::new(KeyId::Q, "Q", "Select", None),
        KeycapDef::new(KeyId::W, "W", "Raise", Some("Lower")),
        KeycapDef::new(KeyId::E, "E", "Material", None),
        KeycapDef::new(KeyId::R, "R", "Spawn", None),
        KeycapDef::new(KeyId::T, "T", "Land", Some("Ocean")),
        KeycapDef::new(KeyId::A, "A", "Pause", None),
        KeycapDef::new(KeyId::S, "S", "Speed", None),
        KeycapDef::new(KeyId::F, "F", "Lightning", None),
    ],
};

impl KeycapPalette {
    /// Mark a single keycap as active (1px teal edge + 3px halo).
    pub fn set_active(&mut self, id: KeyId) {
        for k in &mut self.keys {
            k.active = k.id == id;
        }
    }

    /// Find the keycap by id (read-only accessor).
    #[must_use]
    pub fn get(&self, id: KeyId) -> Option<&KeycapDef> {
        self.keys.iter().find(|k| k.id == id)
    }

    /// Find the keycap by id (mutable accessor for input-driven `active`
    /// updates).
    pub fn get_mut(&mut self, id: KeyId) -> Option<&mut KeycapDef> {
        self.keys.iter_mut().find(|k| k.id == id)
    }

    /// Number of keycaps (always 8 — the contract).
    #[must_use]
    pub const fn len(&self) -> usize {
        8
    }

    /// Always false — the palette is fixed-size by the design recipe.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_matches_god_tools_sandbox_recipe() {
        let p = KeycapPalette::default();
        assert_eq!(p.keys.len(), 8);
        assert_eq!(p.keys[0].glyph, "Q");
        assert_eq!(p.keys[0].verb, "Select");
        assert_eq!(p.keys[1].glyph, "W");
        assert_eq!(p.keys[1].verb_secondary, Some("Lower"));
        assert_eq!(p.keys[5].glyph, "A");
        assert_eq!(p.keys[5].verb, "Pause");
        assert_eq!(p.keys[6].glyph, "S");
        assert_eq!(p.keys[6].verb, "Speed");
        assert_eq!(p.keys[7].glyph, "F");
        assert_eq!(p.keys[7].verb, "Lightning");
    }

    #[test]
    fn set_active_marks_only_one_key() {
        let mut p = KeycapPalette::default();
        p.set_active(KeyId::S);
        assert!(p.get(KeyId::S).unwrap().active);
        for k in &p.keys {
            if k.id != KeyId::S {
                assert!(!k.active, "{} should not be active", k.glyph);
            }
        }
    }

    #[test]
    fn const_default_matches_default_constructor() {
        assert_eq!(KEYCAP_PALETTE_DEFAULT.keys.len(), 8);
        let mut p = KeycapPalette::default();
        // Default starts with nothing active.
        assert!(p.keys.iter().all(|k| !k.active));
    }
}