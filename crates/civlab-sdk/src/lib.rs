//! civlab-sdk — guest API for CivLab WASM mods.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Policy-phase hook (no-op until host wires full capability API).
#[must_use]
pub fn policy_tick() -> i32 {
    0
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn civlab_policy_tick() -> i32 {
    policy_tick()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_tick_is_noop() {
        assert_eq!(policy_tick(), 0);
    }
}
