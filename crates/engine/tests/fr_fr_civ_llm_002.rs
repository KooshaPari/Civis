//! Tests for FR-CIV-LLM-002
//!
//! Epic: FR-CIV-LLM
//!
//! This test file verifies FR FR-CIV-LLM-002: Civ AI Decision structure.

#[cfg(test)]
mod fr_fr_civ_llm_002 {
    /// Verify FR-CIV-LLM-002: CivAiDecision has required fields.
    #[test]
    fn verify_fr_civ_llm_002_basic() {
        let decision = civ_engine::CivAiDecision {
            tick: 100,
            agent_id: 42,
            prompt: "What should we build?".to_string(),
            output: "Build a farm".to_string(),
        };
        assert_eq!(decision.tick, 100);
        assert_eq!(decision.agent_id, 42);
        assert_eq!(decision.prompt, "What should we build?");
        assert_eq!(decision.output, "Build a farm");
    }

    /// Verify CivAiDecision is Clone + PartialEq.
    #[test]
    fn civ_ai_decision_traits() {
        let a = civ_engine::CivAiDecision {
            tick: 1,
            agent_id: 1,
            prompt: "p".to_string(),
            output: "o".to_string(),
        };
        let b = a.clone();
        assert_eq!(a, b);
    }
}
