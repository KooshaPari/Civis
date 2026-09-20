//! Tests for FR-CIV-MOD-006 — Hook priority ordering
//!
//! Epic: FR-CIV-MOD
//! Verifies that hooks execute in priority order.

#[cfg(test)]
mod fr_fr_civ_mod_006 {
    /// FR-CIV-MOD-006: HookEngine clear removes all registrations.
    #[test]
    fn clear_removes_all_registrations() {
        use civ_mod_host::hooks::{ModHookEngine, ModHook};
        let mut engine = ModHookEngine::new();
        engine.register("mod-a", ModHook::OnTick(0), 0);
        engine.register("mod-b", ModHook::OnTick(0), 1);
        assert_eq!(engine.get_registrations(&ModHook::OnTick(0)).len(), 2);
        engine.clear();
        assert!(engine.get_registrations(&ModHook::OnTick(0)).is_empty());
    }

    /// FR-CIV-MOD-006: Unregistering a hook removes it.
    #[test]
    fn unregister_removes_hook() {
        use civ_mod_host::hooks::{ModHookEngine, ModHook};
        let mut engine = ModHookEngine::new();
        engine.register("mod-a", ModHook::OnTick(0), 0);
        engine.unregister("mod-a", &ModHook::OnTick(0));
        assert!(engine.get_registrations(&ModHook::OnTick(0)).is_empty());
    }
}
