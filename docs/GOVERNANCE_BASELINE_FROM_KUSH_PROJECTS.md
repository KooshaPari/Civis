# Governance Baseline Adopted From Sibling Kush Projects

Projects scanned: `crun`, `task2`, `zen-mcp-server`, `thegent`.

## Reusable Governance Practices

1. Spec-first change flow (`proposal -> tasks -> validation -> archive`) before implementation.
2. Deterministic quality gates (`lint`, `type-check`, `unit/integration`) as required completion criteria.
3. Explicit workstream ownership and sub-agent delegation for large efforts.
4. Reproducible toolchain pinning (`uv.lock` / lockfiles) and task runners (`Taskfile.yml`/`make`).
5. Security and operations hygiene in docs (permissions, least-privilege, environment/config contracts).
6. Strong anti-drift rules: avoid compatibility shims/silent failure paths; fail loud and document decisions.

## Civ Baseline Policy

1. Every major feature starts as a spec PR in `docs/`.
2. Every spec includes invariants, interfaces, and acceptance checks.
3. Every implementation change references spec IDs and theorem/assumption dependencies.
4. Every release candidate runs full deterministic quality checks and replay consistency checks.

## Source Snapshot Paths

- `docs/upstream-governance/crun/`
- `docs/upstream-governance/task2/`
- `docs/upstream-governance/zen-mcp-server/`
- `docs/upstream-governance/thegent/`
