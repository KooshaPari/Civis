//! Online integrity monitor (CIV-0001 partial).
//!
//! Lightweight checks callable each tick or after replay load: the replay log's
//! hash-chain tip must match its tick markers, and post-tick invariants must hold.

use crate::engine::Simulation;
use crate::invariants::{check_tick_invariants, InvariantError};
use crate::replay::ReplayError;

/// Integrity violation detected during online monitoring.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegrityError {
    /// Stored hash-chain root does not match tick events in the replay log.
    HashChainMismatch,
    /// Post-tick invariant check failed.
    Invariant(InvariantError),
}

impl std::fmt::Display for IntegrityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HashChainMismatch => write!(f, "replay hash chain mismatch"),
            Self::Invariant(err) => write!(f, "{err:?}"),
        }
    }
}

impl std::error::Error for IntegrityError {}

/// Check hash-chain consistency and tick invariants on a live [`Simulation`].
///
/// - **Hash chain:** when [`ReplayLog::running_hash`](crate::replay::ReplayLog::running_hash)
///   is present, it must equal [`ReplayLog::recompute_running_hash`](crate::replay::ReplayLog::recompute_running_hash).
///   Legacy logs without a stored root skip this step.
/// - **Invariants:** delegates to [`check_tick_invariants`].
pub fn check_integrity(sim: &Simulation) -> Result<(), IntegrityError> {
    sim.replay_log()
        .verify_hash_chain()
        .map_err(|err| match err {
            ReplayError::HashChainMismatch => IntegrityError::HashChainMismatch,
            _ => IntegrityError::HashChainMismatch,
        })?;
    check_tick_invariants(sim).map_err(IntegrityError::Invariant)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::invariants::InvariantError;
    use crate::replay::ReplayLog;
    use civ_economy::{LedgerEntry, LedgerInvariantError, ACCOUNT_CONSUMPTION};
    use tempfile::NamedTempFile;

    #[test]
    fn check_integrity_accepts_fresh_simulation() {
        let sim = Simulation::with_seed(1);
        check_integrity(&sim).expect("initial state");
    }

    #[test]
    fn check_integrity_accepts_after_tick() {
        let mut sim = Simulation::with_seed(42);
        sim.tick();
        check_integrity(&sim).expect("hash chain and invariants after one tick");
        assert!(sim.hash_chain_root().is_some());
    }

    #[test]
    fn check_integrity_across_many_ticks() {
        let mut sim = Simulation::with_seed(104);
        for _ in 0..200 {
            sim.tick();
            check_integrity(&sim).expect("integrity holds each tick");
        }
    }

    #[test]
    fn check_integrity_rejects_tampered_hash_chain() {
        let mut sim = Simulation::with_seed(7);
        sim.tick();
        sim.replay_log_mut().running_hash = Some([0xAA; crate::hash_chain::HASH_LEN]);
        let err = check_integrity(&sim).unwrap_err();
        assert_eq!(err, IntegrityError::HashChainMismatch);
    }

    #[test]
    fn check_integrity_rejects_invariant_violation() {
        let mut sim = Simulation::with_seed(3);
        sim.tick();
        sim.economy_state.tick = 1;
        sim.economy_state.ledger.push(LedgerEntry {
            tick: 0,
            debit: 1,
            credit: 1,
            account: ACCOUNT_CONSUMPTION,
        });
        let err = check_integrity(&sim).unwrap_err();
        assert_eq!(
            err,
            IntegrityError::Invariant(InvariantError::EconomyLedger(
                LedgerInvariantError::LedgerTooLarge {
                    len: sim.economy_state.ledger.len(),
                    tick: 1,
                    max_len: 2,
                }
            ))
        );
    }

    #[test]
    fn check_integrity_after_replay_load() {
        let mut sim = Simulation::with_seed(9);
        for _ in 0..5 {
            sim.tick();
        }

        let file = NamedTempFile::new().unwrap();
        sim.save_replay(file.path()).unwrap();
        let loaded = Simulation::load_replay_from_file(file.path()).unwrap();
        check_integrity(&loaded).expect("integrity after replay load");
        assert_eq!(loaded.state.tick, sim.state.tick);
        assert_eq!(loaded.hash_chain_root(), sim.hash_chain_root());
    }

    #[test]
    fn replay_log_without_stored_hash_skips_chain_check() {
        let mut log = ReplayLog {
            seed: 1,
            ..ReplayLog::default()
        };
        log.record_tick(1);
        log.running_hash = None;

        let mut sim = Simulation::with_seed(1);
        sim.replay_log_mut().events = log.events;
        sim.replay_log_mut().running_hash = None;
        sim.state.tick = 1;
        check_integrity(&sim).expect("legacy log without stored root");
    }
}
