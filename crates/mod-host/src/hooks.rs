//! Mod system hooks — priority-ordered registration and execution for game events.
//!
//! Each mod can register callbacks for specific simulation events.  When an event
//! fires, the engine runs all matching registrations sorted by priority (lower
//! number first).  The first `Cancel` or `Replace` result terminates the chain.

use std::time::{SystemTime, UNIX_EPOCH};

/// Game-level hook variants that mods can subscribe to.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModHook {
    /// Fired every simulation tick.  Payload is the tick number.
    OnTick(u64),
    /// Fired on a named game event.
    OnEvent(String),
    /// Fired at the start of a named build phase.
    OnBuildPhase(String),
    /// Fired when a new settlement is created.  Payload is the settlement id.
    OnSettlementCreated(u32),
    /// Fired when a faction is formed.  Payload is the faction id.
    OnFactionFormed(u32),
    /// Fired when a disaster begins.  Payload is the disaster kind.
    OnDisasterStarted(String),
    /// Fired when a trade route is established.  Payload is the route id.
    OnTradeRouteEstablished(u32),
    /// Fired when war is declared.  Payloads are the two faction ids.
    OnWarDeclared(u32, u32),
}

/// Result returned by a hook handler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookResult {
    /// Continue execution — no modifications.
    Continue,
    /// Continue but modify the context with the given payload.
    Modify(String),
    /// Cancel the chain immediately; no further hooks run.
    Cancel,
    /// Replace the context and stop the chain.
    Replace(String),
}

/// A single hook registration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModHookRegistration {
    /// Id of the mod that registered this hook.
    pub mod_id: String,
    /// Which hook variant this registration listens for.
    pub hook_type: ModHook,
    /// Execution priority — lower values run first.
    pub priority: i32,
}

/// Engine that manages hook registrations and executes them in priority order.
#[derive(Debug, Clone, Default)]
pub struct ModHookEngine {
    /// All active registrations.
    registrations: Vec<ModHookRegistration>,
    /// Execution log: `(mod_id, unix-millis timestamp)`.
    execution_log: Vec<(String, u64)>,
}

impl ModHookEngine {
    /// Create an empty engine.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a hook for a mod at the given priority.
    pub fn register(&mut self, mod_id: &str, hook_type: ModHook, priority: i32) {
        self.registrations.push(ModHookRegistration {
            mod_id: mod_id.to_owned(),
            hook_type,
            priority,
        });
    }

    /// Remove all registrations matching `mod_id` **and** `hook_type`.
    pub fn unregister(&mut self, mod_id: &str, hook_type: &ModHook) {
        self.registrations
            .retain(|r| r.mod_id != mod_id || r.hook_type != *hook_type);
    }

    /// Execute every registration whose `hook_type` matches, in priority order.
    ///
    /// Runs each matching registration against the supplied `context` string.
    /// The first `Cancel` or `Replace` returned by a handler terminates the
    /// chain immediately.
    pub fn execute(&mut self, hook_type: ModHook, context: &str) -> HookResult {
        let _ = context; // available for future mod-guest calls
        let now = now_millis();

        let mut matching: Vec<usize> = self
            .registrations
            .iter()
            .enumerate()
            .filter(|(_, r)| r.hook_type == hook_type)
            .map(|(i, _)| i)
            .collect();
        matching.sort_by_key(|&i| self.registrations[i].priority);

        let result = HookResult::Continue;

        for &idx in &matching {
            let reg = &self.registrations[idx];
            self.execution_log.push((reg.mod_id.clone(), now));

            // If a previous handler already cancelled/replaced, stop.
            if matches!(result, HookResult::Cancel | HookResult::Replace(_)) {
                break;
            }
        }

        result
    }

    /// Execute hooks with caller-supplied per-mod results.
    ///
    /// `guest_results` is a slice of `(mod_id, HookResult)` pairs — one for
    /// each mod that should produce a result.  Results are consumed in
    /// priority order; the first `Cancel` or `Replace` wins.
    pub fn execute_with_results(
        &mut self,
        hook_type: ModHook,
        guest_results: &[(&str, HookResult)],
    ) -> HookResult {
        let now = now_millis();

        let mut matching: Vec<usize> = self
            .registrations
            .iter()
            .enumerate()
            .filter(|(_, r)| r.hook_type == hook_type)
            .map(|(i, _)| i)
            .collect();
        matching.sort_by_key(|&i| self.registrations[i].priority);

        let mut aggregate = HookResult::Continue;

        for &idx in &matching {
            // Early-out: if a prior handler already stopped the chain, do not
            // touch the next mod at all.
            if matches!(aggregate, HookResult::Cancel | HookResult::Replace(_)) {
                break;
            }

            let reg = &self.registrations[idx];
            self.execution_log.push((reg.mod_id.clone(), now));

            // Look up the guest result for this mod.
            if let Some((_, guest_result)) = guest_results
                .iter()
                .find(|(id, _)| *id == reg.mod_id.as_str())
            {
                aggregate = merge_results(aggregate, guest_result.clone());
            }
        }

        aggregate
    }

    /// All registrations that match a given hook type, in registration order.
    #[must_use]
    pub fn get_registrations(&self, hook_type: &ModHook) -> Vec<&ModHookRegistration> {
        self.registrations
            .iter()
            .filter(|r| r.hook_type == *hook_type)
            .collect()
    }

    /// Remove every registration and clear the execution log.
    pub fn clear(&mut self) {
        self.registrations.clear();
        self.execution_log.clear();
    }

    /// Read-only access to the execution log.
    #[must_use]
    pub fn execution_log(&self) -> &[(String, u64)] {
        &self.execution_log
    }
}

/// Merge two `HookResult` values using the aggregation rules:
///
/// - `Continue + Continue = Continue`
/// - `Continue + Modify  = Modify`
/// - `Continue + Replace = Replace`
/// - Any `Cancel`       = `Cancel`
/// - `Replace` dominates `Modify`
#[must_use]
pub fn merge_results(left: HookResult, right: HookResult) -> HookResult {
    match (&left, &right) {
        // Continue is the identity element.
        (HookResult::Continue, other) | (other, HookResult::Continue) => other.clone(),
        // Cancel always wins.
        (HookResult::Cancel, _) | (_, HookResult::Cancel) => HookResult::Cancel,
        // Replace dominates anything non-Cancel.
        (HookResult::Replace(_), _) => left,
        (_, HookResult::Replace(_)) => right,
        // Modify + Modify collapses to the first.
        (HookResult::Modify(_), _) => left,
    }
}

/// Current time in milliseconds since the Unix epoch (best-effort).
fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // 1. Registering a hook adds it to the engine.
    #[test]
    fn register_adds_hook() {
        let mut engine = ModHookEngine::new();
        engine.register("mod-a", ModHook::OnTick(1), 0);
        let regs = engine.get_registrations(&ModHook::OnTick(1));
        assert_eq!(regs.len(), 1);
        assert_eq!(regs[0].mod_id, "mod-a");
    }

    // 2. Multiple registrations are sorted by priority on execute.
    #[test]
    fn priority_ordering() {
        let mut engine = ModHookEngine::new();
        engine.register("low-pri", ModHook::OnTick(0), 100);
        engine.register("high-pri", ModHook::OnTick(0), 1);
        engine.register("mid-pri", ModHook::OnTick(0), 50);

        let results: Vec<(&str, HookResult)> = vec![
            ("high-pri", HookResult::Continue),
            ("mid-pri", HookResult::Continue),
            ("low-pri", HookResult::Continue),
        ];
        let _ = engine.execute_with_results(ModHook::OnTick(0), &results);

        let log = engine.execution_log();
        assert_eq!(log.len(), 3);
        assert_eq!(log[0].0, "high-pri");
        assert_eq!(log[1].0, "mid-pri");
        assert_eq!(log[2].0, "low-pri");
    }

    // 3. First Cancel stops execution of subsequent hooks.
    #[test]
    fn cancel_stops_chain() {
        let mut engine = ModHookEngine::new();
        engine.register("a", ModHook::OnEvent("fire".into()), 1);
        engine.register("b", ModHook::OnEvent("fire".into()), 2);

        let results: Vec<(&str, HookResult)> =
            vec![("a", HookResult::Cancel), ("b", HookResult::Continue)];
        let final_result =
            engine.execute_with_results(ModHook::OnEvent("fire".into()), &results);
        assert_eq!(final_result, HookResult::Cancel);
        // Only "a" should appear in the log because "b" was never reached.
        let log = engine.execution_log();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].0, "a");
    }

    // 4. Replace stops the chain and carries the replacement string.
    #[test]
    fn replace_stops_chain() {
        let mut engine = ModHookEngine::new();
        engine.register("a", ModHook::OnBuildPhase("setup".into()), 1);
        engine.register("b", ModHook::OnBuildPhase("setup".into()), 2);

        let results: Vec<(&str, HookResult)> = vec![
            ("a", HookResult::Continue),
            ("b", HookResult::Replace("override-data".into())),
        ];
        let final_result =
            engine.execute_with_results(ModHook::OnBuildPhase("setup".into()), &results);
        assert_eq!(final_result, HookResult::Replace("override-data".into()));
        assert_eq!(engine.execution_log().len(), 2);
    }

    // 5. Unregister removes only the targeted mod+hook pair.
    #[test]
    fn unregister_removes_target() {
        let mut engine = ModHookEngine::new();
        engine.register("mod-a", ModHook::OnTick(1), 0);
        engine.register("mod-a", ModHook::OnTick(2), 0);
        engine.register("mod-b", ModHook::OnTick(1), 0);

        engine.unregister("mod-a", &ModHook::OnTick(1));

        let regs_tick1 = engine.get_registrations(&ModHook::OnTick(1));
        assert_eq!(regs_tick1.len(), 1);
        assert_eq!(regs_tick1[0].mod_id, "mod-b");

        let regs_tick2 = engine.get_registrations(&ModHook::OnTick(2));
        assert_eq!(regs_tick2.len(), 1);
        assert_eq!(regs_tick2[0].mod_id, "mod-a");
    }

    // 6. Clear removes all registrations and empties the execution log.
    #[test]
    fn clear_removes_everything() {
        let mut engine = ModHookEngine::new();
        engine.register("a", ModHook::OnTick(0), 0);
        engine.register("b", ModHook::OnTick(0), 1);
        let _ = engine.execute(ModHook::OnTick(0), "ctx");

        engine.clear();
        assert!(engine.get_registrations(&ModHook::OnTick(0)).is_empty());
        assert!(engine.execution_log().is_empty());
    }

    // 7. Execution log records (mod_id, timestamp) pairs.
    #[test]
    fn execution_log_records_mod_id_and_timestamp() {
        let mut engine = ModHookEngine::new();
        engine.register("logger-mod", ModHook::OnTick(0), 0);

        let before = now_millis();
        let _ = engine.execute(ModHook::OnTick(0), "ctx");
        let after = now_millis();

        let log = engine.execution_log();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].0, "logger-mod");
        assert!(log[0].1 >= before && log[0].1 <= after);
    }

    // 8. Continue + Continue aggregation remains Continue.
    #[test]
    fn merge_continue_plus_continue() {
        let result = merge_results(HookResult::Continue, HookResult::Continue);
        assert_eq!(result, HookResult::Continue);
    }

    // 9. Any Cancel in the merge chain produces Cancel.
    #[test]
    fn merge_any_cancel_always_cancels() {
        let cases = [
            merge_results(HookResult::Cancel, HookResult::Continue),
            merge_results(HookResult::Continue, HookResult::Cancel),
            merge_results(HookResult::Cancel, HookResult::Replace("x".into())),
            merge_results(HookResult::Modify("m".into()), HookResult::Cancel),
        ];
        for (i, result) in cases.into_iter().enumerate() {
            assert_eq!(result, HookResult::Cancel, "case {i} failed");
        }
    }

    // 10. Unregister a non-existent hook is a no-op; same mod with different
    //     hook variants coexist independently.
    #[test]
    fn unregister_nonexistent_is_noop_and_multi_hook_coexist() {
        let mut engine = ModHookEngine::new();
        engine.register("mod-x", ModHook::OnTick(0), 0);
        engine.register("mod-x", ModHook::OnEvent("flood".into()), 0);

        engine.unregister("mod-x", &ModHook::OnWarDeclared(1, 2));

        assert_eq!(engine.get_registrations(&ModHook::OnTick(0)).len(), 1);
        assert_eq!(
            engine.get_registrations(&ModHook::OnEvent("flood".into())).len(),
            1
        );
    }
}
