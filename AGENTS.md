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
