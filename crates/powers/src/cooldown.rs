//! Per-power cooldown state for god-power usage.
//!
//! FR-CIV-POWER-COOLDOWN: a power becomes unavailable immediately after
//! use and recharges over time until it can be used again.

/// Snapshot of a power's cooldown state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PowerCooldownState {
    /// The total cooldown duration in ticks.
    pub cooldown_ticks: u64,
    /// The remaining ticks before the power becomes available.
    pub remaining_ticks: u64,
}

/// Cooldown tracker for a single power.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PowerCooldown {
    cooldown_ticks: u64,
    remaining_ticks: u64,
}

impl PowerCooldown {
    /// Create a cooldown tracker with the given recharge duration.
    #[must_use]
    pub const fn new(cooldown_ticks: u64) -> Self {
        Self {
            cooldown_ticks,
            remaining_ticks: 0,
        }
    }

    /// Returns `true` when the power can be used right now.
    #[must_use]
    pub const fn is_available(self) -> bool {
        self.remaining_ticks == 0
    }

    /// Mark the power as used, starting a fresh cooldown window.
    pub fn use_power(&mut self) {
        self.remaining_ticks = self.cooldown_ticks;
    }

    /// Advance time by `ticks`, reducing the remaining cooldown.
    pub fn tick(&mut self, ticks: u64) {
        self.remaining_ticks = self.remaining_ticks.saturating_sub(ticks);
    }

    /// Return a serializable snapshot of the cooldown state.
    #[must_use]
    pub const fn state(self) -> PowerCooldownState {
        PowerCooldownState {
            cooldown_ticks: self.cooldown_ticks,
            remaining_ticks: self.remaining_ticks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PowerCooldown;

    #[test]
    fn power_unavailable_right_after_use_and_available_after_cooldown() {
        let mut cooldown = PowerCooldown::new(5);

        assert!(cooldown.is_available());

        cooldown.use_power();
        assert!(!cooldown.is_available());

        cooldown.tick(4);
        assert!(!cooldown.is_available());

        cooldown.tick(1);
        assert!(cooldown.is_available());
    }
}
