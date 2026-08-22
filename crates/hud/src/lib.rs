//! `civ-hud` — HUD dashboard panels and overlays for Civis.
//!
//! Pure data types for population, economy, diplomacy, technology, and
//! society indicators. All structs are `serde`-serialisable for JSON wire
//! transport to any client (web, Bevy, Godot, Unreal).
//!
//! ## Module overview
//!
//! | Module | Core types |
//! |--------|-----------|
//! | [`population_panel`] | `PopulationPanel`, `AgeBand`, `FactionBreakdown` |
//! | [`economy_panel`] | `EconomyPanel`, `ResourceStock`, `EmploymentSector` |
//! | [`diplomacy_panel`] | `DiplomacyPanel`, `RelationEntry`, `ThreatLevel` |
//! | [`technology_panel`] | `TechnologyPanel`, `ResearchProject`, `TechTreeCoverage` |
//! | [`overlay_registry`] | `OverlayId`, `OverlayRegistry` |
//! | [`overlay_legend`] | Legend entries for visual overlays |
//! | [`env_overlay`] | Environment/weather overlay data |
//! | [`society_overlay`] | Society/culture overlay data |
//! | [`notifications`] | HUD notification entries |
//! | [`godtool_brush`] | God-tool brush data |
//! | [`god_tool_state`] | God-tool persistent state |
//!
//! ## Design contract
//!
//! 1. **Pure data, no engine.** No Bevy, no rendering, no systems.
//! 2. **Additive only.** This crate does not modify any existing public surface.
//! 3. **Serialisation-safe.** Every public field is `serde`-friendly.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod diplomacy_panel;
pub mod economy_panel;
pub mod env_overlay;
pub mod god_tool_state;
pub mod godtool_brush;
pub mod notifications;
pub mod overlay_legend;
pub mod overlay_registry;
pub mod population_panel;
pub mod society_overlay;
pub mod technology_panel;
