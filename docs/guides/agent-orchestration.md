# Agent Orchestration

Use parallel agents for bounded discovery and implementation work, then converge
through a single integrating agent that owns validation and final commits.

For Civis, keep orchestration tied to the active spec IDs, the workstream, and
the repository quality gates. Child work should report exact files changed,
tests run, and any blockers that require parent-agent judgment.
