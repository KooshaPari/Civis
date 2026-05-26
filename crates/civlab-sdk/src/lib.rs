//! civlab-sdk — guest API for CivLab WASM mods.

#![cfg_attr(not(target_arch = "wasm32"), forbid(unsafe_code))]
#![warn(missing_docs)]

/// Capability surface version echoed by host and guest (FR-CIV-TACTICS-044).
pub const CAPABILITY_API_VERSION: &str = "0.1.0";

/// Policy-phase hook (no-op until host wires full capability API).
#[must_use]
pub fn policy_tick(tick: u64) -> i32 {
    let _ = tick;
    0
}

/// Economic-phase hook (no-op until host wires full capability API).
#[must_use]
pub fn economy_tick(tick: u64) -> i32 {
    let _ = tick;
    0
}

/// Military-phase hook (no-op until host wires full capability API).
#[must_use]
pub fn military_tick(tick: u64) -> i32 {
    let _ = tick;
    0
}

/// WASM export for the policy phase (`civlab_policy_tick`).
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn civlab_policy_tick(tick: i64) -> i32 {
    policy_tick(tick as u64)
}

/// WASM export for the economy phase (`civlab_economy_tick`).
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn civlab_economy_tick(tick: i64) -> i32 {
    economy_tick(tick as u64)
}

/// WASM export for the military phase (`civlab_military_tick`).
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn civlab_military_tick(tick: i64) -> i32 {
    military_tick(tick as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_tick_is_noop() {
        assert_eq!(policy_tick(0), 0);
    }

    #[test]
    fn economy_tick_is_noop() {
        assert_eq!(economy_tick(0), 0);
    }

    #[test]
    fn military_tick_is_noop() {
        assert_eq!(military_tick(0), 0);
    }

    #[test]
    fn capability_version_is_non_empty() {
        assert!(!CAPABILITY_API_VERSION.is_empty());
    }
}
