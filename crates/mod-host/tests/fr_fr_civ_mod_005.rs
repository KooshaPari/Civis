//! Tests for FR-CIV-MOD-005 — Hook registration and execution
//!
//! Epic: FR-CIV-MOD
//! Verifies that ModHookEngine registers and executes hooks.

#[cfg(test)]
mod fr_fr_civ_mod_005 {
    /// FR-CIV-MOD-005: ModHookEngine starts empty.
    #[test]
    fn hook_engine_starts_empty() {
        use civ_mod_host::hooks::{ModHookEngine, ModHook};
        let engine = ModHookEngine::new();
        let regs = engine.get_registrations(&ModHook::OnTick(0));
        assert!(regs.is_empty());
    }

    /// FR-CIV-MOD-005: Registering a hook makes it retrievable.
    #[test]
    fn register_makes_hook_retrievable() {
        use civ_mod_host::hooks::{ModHookEngine, ModHook};
        let mut engine = ModHookEngine::new();
        engine.register("test-mod", ModHook::OnTick(0), 0);
        let regs = engine.get_registrations(&ModHook::OnTick(0));
        assert_eq!(regs.len(), 1);
        assert_eq!(regs[0].mod_id, "test-mod");
    }

    /// FR-CIV-MOD-005: Executing with no hooks returns Continue.
    #[test]
    fn execute_empty_returns_continue() {
        use civ_mod_host::hooks::{ModHookEngine, ModHook, HookResult};
        let mut engine = ModHookEngine::new();
        let result = engine.execute(ModHook::OnTick(0), "{}");
        assert!(matches!(result, HookResult::Continue));
    }
}
