//! civlab-sdk — guest API for CivLab WASM mods.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Policy-phase hook (no-op until host wires full capability API).
#[must_use]
pub fn policy_tick() -> i32 {
    0
}

/// Military-phase hook (no-op until host wires full capability API).
#[must_use]
pub fn military_tick(tick: u64) -> i32 {
    let _ = tick;
    0
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn civlab_policy_tick() -> i32 {
    policy_tick()
}

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
        assert_eq!(policy_tick(), 0);
    }

    #[test]
    fn military_tick_is_noop() {
        assert_eq!(military_tick(0), 0);
    }
}
