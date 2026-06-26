//! Risk tier for a verb — drives confirmation prompts, undo availability,
//! and visual styling in the HolocronPanel.

use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum RiskTier {
    /// Pure inspection — no side effects on world state.
    ReadOnly,
    /// Local, easily reversible action (spawn 1 villager, build 1 structure).
    Minor,
    /// Affects many entities or persists for a long time (bless a settlement,
    /// shift market prices).
    Major,
    /// World-altering or irreversible (smite faction, end era, apocalypse).
    Critical,
}

impl RiskTier {
    /// Color key for the keycap palette (ADR-012).
    /// Returns the semantic role token name, not an RGB triple — the HUD
    /// resolves role → color at render time.
    pub fn palette_role(self) -> &'static str {
        match self {
            Self::ReadOnly => "keycap.inert",
            Self::Minor => "keycap.calm",
            Self::Major => "keycap.warn",
            Self::Critical => "keycap.danger",
        }
    }

    /// Whether firing this verb should require a confirmation modal.
    pub fn requires_confirmation(self) -> bool {
        matches!(self, Self::Major | Self::Critical)
    }

    /// Whether firing this verb should append an undo entry (if the engine
    /// supports undo for this verb).
    pub fn supports_undo(self) -> bool {
        matches!(self, Self::Minor | Self::Major)
    }

    /// Sort key — minor first, critical last.
    pub fn sort_key(self) -> u8 {
        match self {
            Self::ReadOnly => 0,
            Self::Minor => 1,
            Self::Major => 2,
            Self::Critical => 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn critical_requires_confirmation() {
        assert!(RiskTier::Critical.requires_confirmation());
        assert!(RiskTier::Major.requires_confirmation());
        assert!(!RiskTier::Minor.requires_confirmation());
        assert!(!RiskTier::ReadOnly.requires_confirmation());
    }

    #[test]
    fn readonly_cannot_undo() {
        assert!(!RiskTier::ReadOnly.supports_undo());
        assert!(!RiskTier::Critical.supports_undo());
        assert!(RiskTier::Minor.supports_undo());
        assert!(RiskTier::Major.supports_undo());
    }

    #[test]
    fn palette_role_unique() {
        let roles: Vec<&str> = [
            RiskTier::ReadOnly,
            RiskTier::Minor,
            RiskTier::Major,
            RiskTier::Critical,
        ]
        .iter()
        .map(|r| r.palette_role())
        .collect();
        let mut uniq = roles.clone();
        uniq.sort();
        uniq.dedup();
        assert_eq!(uniq.len(), roles.len(), "palette_role must be unique per tier");
    }
}