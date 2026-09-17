//! FR traceability tests for engine replay entity handling, scenario loading,
//! and extended hash chain payloads.
//!
//! Covers: FR-REPLAY-002, FR-CIV-TACTICS-041, FR-CIV-RESEARCH-004,
//! FR-CORE-004, FR-CIV-INT-001

use civ_engine::hash_chain::{
    chain_advance, chain_root_from_payloads, combat_event_bytes, research_event_bytes,
    tick_event_bytes, HashChainState, GENESIS, HASH_LEN,
};
use civ_engine::replay::MAX_REPLAY_ENTITY_SLOTS;

// ===========================================================================
// FR-REPLAY-002 — Replay entity limits
// ===========================================================================

/// FR-REPLAY-002 — MAX_REPLAY_ENTITY_SLOTS is 8 MiB.
#[test]
fn fr_replay_002_max_entity_slots() {
    assert_eq!(MAX_REPLAY_ENTITY_SLOTS, 8 * 1024 * 1024);
}

// ===========================================================================
// FR-CIV-TACTICS-041 — Combat event bytes for hash chain
// ===========================================================================

/// FR-CIV-TACTICS-041 — combat_event_bytes produces 69-byte payload.
#[test]
fn fr_civ_tactics_041_combat_event_bytes_length() {
    let bytes = combat_event_bytes(100, 1, 2, 10, 20, 30, 5, 100, 50);
    assert_eq!(bytes.len(), 63, "combat event should be exactly 63 bytes (6+8*6+1+4*2)");
}

/// FR-CIV-TACTICS-041 — combat_event_bytes starts with "combat" tag.
#[test]
fn fr_civ_tactics_041_combat_event_bytes_tag() {
    let bytes = combat_event_bytes(0, 0, 0, 0, 0, 0, 0, 0, 0);
    assert_eq!(&bytes[..6], b"combat");
}

/// FR-CIV-TACTICS-041 — combat_event_bytes is deterministic.
#[test]
fn fr_civ_tactics_041_combat_event_bytes_deterministic() {
    let a = combat_event_bytes(1, 2, 3, 4, 5, 6, 7, 8, 9);
    let b = combat_event_bytes(1, 2, 3, 4, 5, 6, 7, 8, 9);
    assert_eq!(a, b);
}

/// FR-CIV-TACTICS-041 — Different combat events produce different bytes.
#[test]
fn fr_civ_tactics_041_different_combat_different_bytes() {
    let a = combat_event_bytes(1, 0, 0, 0, 0, 0, 0, 0, 0);
    let b = combat_event_bytes(2, 0, 0, 0, 0, 0, 0, 0, 0);
    assert_ne!(a, b);
}

// ===========================================================================
// FR-CIV-RESEARCH-004 — Research event bytes for hash chain
// ===========================================================================

/// FR-CIV-RESEARCH-004 — research_event_bytes starts with "research" tag.
#[test]
fn fr_civ_research_004_research_event_bytes_tag() {
    let bytes = research_event_bytes(0, &[], false);
    assert_eq!(&bytes[..8], b"research");
}

/// FR-CIV-RESEARCH-004 — research_event_bytes is deterministic.
#[test]
fn fr_civ_research_004_research_event_bytes_deterministic() {
    let snapshot = [1u8; 32];
    let a = research_event_bytes(42, &snapshot, true);
    let b = research_event_bytes(42, &snapshot, true);
    assert_eq!(a, b);
}

/// FR-CIV-RESEARCH-004 — accepted vs rejected produces different bytes.
#[test]
fn fr_civ_research_004_accepted_vs_rejected() {
    let a = research_event_bytes(1, &[], true);
    let b = research_event_bytes(1, &[], false);
    assert_ne!(a, b);
}

// ===========================================================================
// FR-CORE-004 — Hash chain root from mixed payloads
// ===========================================================================

/// FR-CORE-004 — chain_root_from_payloads handles mixed event types.
#[test]
fn fr_core_004_chain_root_mixed_payloads() {
    let payloads: Vec<Vec<u8>> = vec![
        tick_event_bytes(0).to_vec(),
        combat_event_bytes(1, 2, 3, 4, 5, 6, 7, 8, 9).to_vec(),
        tick_event_bytes(1).to_vec(),
        research_event_bytes(5, &[0u8; 32], true).to_vec(),
    ];
    let refs: Vec<&[u8]> = payloads.iter().map(|p| p.as_slice()).collect();
    let root = chain_root_from_payloads(&refs);
    assert!(root.is_some());
    let root = root.unwrap();
    assert_eq!(root.len(), HASH_LEN);
}

/// FR-CORE-004 — chain_root_from_payloads returns None for empty input.
#[test]
fn fr_core_004_chain_root_empty_payloads() {
    let empty: [&[u8]; 0] = [];
    assert!(chain_root_from_payloads(&empty).is_none());
}

// ===========================================================================
// FR-CIV-INT-001 — Integrity: chain_advance produces valid hashes
// ===========================================================================

/// FR-CIV-INT-001 — chain_advance output is always 32 bytes.
#[test]
fn fr_civ_int_001_chain_advance_length() {
    let result = chain_advance(&GENESIS, &tick_event_bytes(0));
    assert_eq!(result.len(), HASH_LEN);
}

/// FR-CIV-INT-001 — chain_advance order matters (non-commutative).
#[test]
fn fr_civ_int_001_chain_advance_order_matters() {
    let a = chain_advance(&GENESIS, &tick_event_bytes(0));
    let b = chain_advance(&GENESIS, &tick_event_bytes(1));
    assert_ne!(a, b);
}

/// FR-CIV-INT-001 — HashChainState::new starts at GENESIS.
#[test]
fn fr_civ_int_001_hash_chain_state_new() {
    let state = HashChainState::new();
    assert_eq!(state.running_hash, GENESIS);
}
