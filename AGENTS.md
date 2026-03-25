# AGENTS.md

## Scope
This file governs the entire `civ` project.

## Rules
1. Spec-first: update `docs/specs` before substantive implementation changes.
2. Determinism-first: simulation logic must be replayable and side-effect explicit.
3. Fail loud: do not add silent compatibility fallbacks.
4. Keep modules focused and composable.
5. Run quality gates (`cargo fmt`, `cargo clippy`, `cargo test`) before finalizing.

## Required References
- `docs/GOVERNANCE_BASELINE_FROM_KUSH_PROJECTS.md`
- `docs/SPEC_COMPLETION_TODOS.md`
- `docs/CONVERSATION_TASKS_FULL.md`

<!-- PHENOTYPE_GOVERNANCE_OVERLAY_V1 -->
## Phenotype Governance Overlay v1

- Enforce `TDD + BDD + SDD` for all feature and workflow changes.
- Enforce `Hexagonal + Clean + SOLID` boundaries by default.
- Favor explicit failures over silent degradation; required dependencies must fail clearly when unavailable.
- Keep local hot paths deterministic and low-latency; place distributed workflow logic behind durable orchestration boundaries.
- Require policy gating, auditability, and traceable correlation IDs for agent and workflow actions.
- Document architectural and protocol decisions before broad rollout changes.


## Bot Review Retrigger and Rate-Limit Governance

- Retrigger commands:
  - CodeRabbit: `@coderabbitai full review`
  - Gemini Code Assist: `@gemini-code-assist review` (fallback: `/gemini review`)
- Rate-limit contract:
  - Maximum one retrigger per bot per PR every 15 minutes.
  - Before triggering, check latest PR comments for existing trigger markers and bot quota/rate-limit responses.
  - If rate-limited, queue the retry for the later of 15 minutes or bot-provided retry time.
  - After two consecutive rate-limit responses for the same bot/PR, stop auto-retries and post queued status with next attempt time.
- Tracking marker required in PR comments for each trigger:
  - `bot-review-trigger: <bot> <iso8601-time> <reason>`
