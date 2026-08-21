#!/usr/bin/env python3
"""Prepend imports and doc comment to emergence_coupling.rs."""
import os

path = os.path.join(os.path.dirname(__file__), "emergence_coupling.rs")

header = """\
//! Emergence-coupling free functions extracted from `engine.rs`.
//!
//! Contains standalone free functions, constants, and helper types that
//! implement the emergence-coupling logic (FR-CIV-0100). Extracted from the
//! monolithic `engine.rs` to reduce file size and improve modularity.

use crate::engine::{
    Building, BuildingType, Citizen, ClusterStocks, CohesionEvent, CohesionEventKind,
    CohesionSnapshot, CombatDamagePulse, EconomicFocus, FabricTier, InvariantError,
    InstitutionEvent, JobType, KinshipEdge, KinshipKind, LanguageState, MembershipPayoffTotals,
    MilitaryUnit, MoodSnapshot, Position, Production, ReligionEvent,
    ReligionEventKind, ReligiousProfile, ResearchCache, Resources, ResourceType, Sim,
    SimSeed, Simulation, SimulationSnapshot, StratBand, StratificationEvent,
    StratificationEventKind, StratificationReport, TradeRoute, UnitType, WorldState,
};
use crate::fixed_math::{Fixed, FixedFromNum};
use crate::SCALE;
use civ_agents::{
    Alignment, Civilian as AgentCivilian, ClusterId, ClusterMember, DiplomacyMatrix,
    DiplomacyOutcome, DiplomacySignal, LodTier, Needs, Position3d, Psyche, SocialGraph,
    Tools, Wardrobe,
};
use civ_agents::culture::{cultural_distance, CultureProfile};
use civ_agents::diplomacy::GriefAccumulator;
use civ_build::{BuildSite, DemandSignals};
use civ_economy::{Good, SettlementTradeFlow};
use civ_genetics::sentience::{
    cognition_score, CognitionTraitProfile, SentienceThreshold,
};
use civ_genetics::Dna;
use civ_planet::{BiomeKind, WeatherCell};
use civ_tactics::{CombatEngagement, DoctrineLibrary};
use civ_voxel::{WorldCoord, FIXED_SCALE};
use hecs::World;
use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::culture::{
    culture_cooperation_signal, culture_openness_signal, FactionIdeologyState,
};

"""

with open(path, "r", encoding="utf-8") as f:
    existing = f.read()

with open(path, "w", encoding="utf-8") as f:
    f.write(header)
    f.write(existing)

# Verify
with open(path, "r", encoding="utf-8") as f:
    lines = f.readlines()
print(f"Total lines: {len(lines)}")
print(f"Line 1: {lines[0].rstrip()}")
print(f"Line 5: {lines[4].rstrip()}")
# Find MembershipPayoff impl
for i, l in enumerate(lines, 1):
    if "impl" in l and "MembershipPayoff" in l and "struct" not in l:
        print(f"Line {i}: {l.rstrip()}")
        break
# Check for biome_yield_factor
for i, l in enumerate(lines, 1):
    if "fn biome_yield_factor" in l:
        print(f"Line {i}: {l.rstrip()}")
        break
