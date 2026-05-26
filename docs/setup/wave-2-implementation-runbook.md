# Wave 2 implementation runbook (smart-contract proof system)

Sequenced plan to land Phases 2-4 of the smart-contract proof system. Each phase is independently shippable. Design reference: `docs/design/2026-04-25-smart-contract-proof-system.md` (sections 8-11).

## Phase status (2026-04-25, iteration 38)

| Phase | Scope | Status | Iteration |
|-------|-------|--------|-----------|
| 1 | proof_signing.py + merkle.py + proof_policy.py + policy yaml + schema | DONE | 38 |
| 2 | gate integration (.claude/commands/prove-features-gate.ps1) | IN-FLIGHT | 38-39 |
| 3 | CI workflow (proof-gate.yml) | PLANNED | 40+ |
| 4 | bridge HMAC (per-session signing) | PLANNED | 40+ |

## Phase 2 — gate integration

### Goal
`prove-features-gate.ps1` reads `policies/proof-policy.yaml`, verifies receipts via cosign/ed25519, and computes/verifies bundle merkle roots. Exits 1 on any policy violation. `-ExternalJudge` becomes default; `-Local` is loud opt-in.

### Subtasks

| # | Subtask | Files |
|---|---------|-------|
| 2.1 | Add CLI shims (argparse wrappers around Phase 1 functions) | `src/Tools/DinoforgeMcp/dinoforge_mcp/proof_signing_cli.py`, `proof_policy_cli.py`, `merkle_cli.py` |
| 2.2 | Modify gate to add `-Local`, `-PolicyFile`, `-Strict` parameters; default-flip `-ExternalJudge`; subprocess into the CLIs above | `.claude/commands/prove-features-gate.ps1` |
| 2.3 | Add 3 sample receipts under `docs/proof/judge-receipts/` (mkdir if needed): one valid Kimi/ed25519, one forbidden Anthropic-family judge, one unsigned | `docs/proof/judge-receipts/sample-*.json` |
| 2.4 | Update skill doc with new flags + 3 manual test transcripts | `.claude/commands/prove-features.md` |

### Acceptance
- `pwsh prove-features-gate.ps1 -PolicyFile policies/proof-policy.yaml -Strict` exits 0 only with a recent valid receipt.
- Forbidden-judge receipt rejected with `model field starts with 'claude-' — forbidden`.
- Unsigned receipt: warn in default mode, reject under `-Strict`.
- 3 manual test transcripts captured in `docs/proof/judge-receipts/manual-runs.md`.

### Effort
1 day. Owner: gate-integration agent. Tracks task #191.

## Phase 3 — CI integration

### Goal
A new workflow `proof-gate.yml` runs `prove-features-gate.ps1` against the latest bundle on every PR. Required check for merge to main.

### Subtasks

| # | Subtask | Notes |
|---|---------|-------|
| 3.1 | Create `.github/workflows/proof-gate.yml` triggering on `pull_request` and `push: main` | Set up Python 3.11 + PowerShell, install `cryptography` (already in `pyproject.toml`), invoke gate |
| 3.2 | Mark check required via branch protection | Done by repo owner; document in runbook |
| 3.3 | Publish first valid receipt to repo (bootstrap) | Blocks on task #103 (user-driven Kimi judge run) — Phase 3 does not merge until #103 lands |
| 3.4 | Drop or mark-experimental conflicting workflows | `game-launch.yml` (no self-hosted runner), `game-automation.yml` (mock-EXE theater) |

### Acceptance
- PR with no recent valid receipt fails CI with policy-violation message.
- PR with fresh signed Kimi receipt passes.
- Branch protection lists `proof-gate` as required.

### Effort
1 day after Phase 2 lands and #103 unblocks.

## Phase 4 — bridge HMAC (per-session signing)

### Goal
Every `IGameBridge` response carries `{timestamp, world_frame, sha256(state_snapshot), hmac}`. `GameClient` verifies HMAC; tampering throws `GameClientException("hmac_invalid")`. Closes the bridge-bypass surface from "explicit error flag" (post-#189) to "cryptographically verifiable response".

### Subtasks

| # | Subtask | Files |
|---|---------|-------|
| 4.1 | Per-session ephemeral key | `src/Runtime/Bridge/SessionHmac.cs` (new) |
| 4.2 | Wrap all bridge responses with `bridge_receipt` | `src/Runtime/Bridge/GameBridgeServer.cs`, `src/Bridge/Protocol/BridgeReceiptDto.cs` (new) |
| 4.3 | Extend Connect handshake to deliver session key | `GameBridgeServer.cs`, `src/Bridge/Client/GameClient.cs` |
| 4.4 | Client-side HMAC verification | `GameClient.cs` — verify on every response, throw on mismatch |
| 4.5 | Tamper test | `src/Tests/Integration/BridgeHmacTests.cs` (new) — flip a byte mid-stream, assert verification fails |
| 4.6 | Bundle aggregator collects bridge_receipts and includes them in merkle root | `proof_signing.py` aggregator path; bundle schema update |

### Acceptance
- Tampering with any bridge response message in transit produces a verifiable HMAC failure on client.
- Proof bundle includes a list of `bridge_receipts` that the gate verifies match the recorded session.
- Unit + integration tests in `BridgeHmacTests` cover happy path, tamper, and replay-with-stale-frame.

### Effort
2 days. Independent of Phases 2-3 — can run in parallel.

## Cross-phase coordination

| Constraint | Rationale |
|-----------|-----------|
| Phase 2 must land before Phase 3 | CI invokes the gate shipped in Phase 2 |
| Phase 4 independent of Phases 2-3 | Bridge HMAC ships and is consumed by the bundle aggregator regardless of CI status |
| Phase 3 blocks on #103 | First real Kimi receipt is user-driven (no automated key access) |

## Dependencies on other tasks

| Task | Status | Relationship |
|------|--------|--------------|
| #189 | closed | Bridge-bypass error fields are the precondition for HMAC (Phase 4) |
| #194 | closed | UI registry wiring lets us produce more meaningful proof bundles |
| #166 | closed | Path-injection fix is the precondition for trusting bundle paths |
| #191 | open | Phase 2 gate integration (this runbook drives it) |
| #192 | open | Phase 3 CI integration (this runbook drives it) |
| #103 | open, user-driven | First external Kimi receipt — Phase 3 acceptance dependency |

## File-modification map

| Phase | Files modified or created |
|-------|--------------------------|
| 2 | `.claude/commands/prove-features-gate.ps1`, `src/Tools/DinoforgeMcp/dinoforge_mcp/proof_signing_cli.py` (new), `proof_policy_cli.py` (new), `merkle_cli.py` (new), `.claude/commands/prove-features.md`, `docs/proof/judge-receipts/sample-*.json` (new) |
| 3 | `.github/workflows/proof-gate.yml` (new), `.github/workflows/game-launch.yml` (drop or label experimental), `.github/workflows/game-automation.yml` (drop) |
| 4 | `src/Runtime/Bridge/SessionHmac.cs` (new), `src/Runtime/Bridge/GameBridgeServer.cs`, `src/Bridge/Client/GameClient.cs`, `src/Bridge/Protocol/BridgeReceiptDto.cs` (new), `src/Tests/Integration/BridgeHmacTests.cs` (new), `proof_signing.py` (aggregator path) |

## Iteration cadence

| Iteration | Target |
|-----------|--------|
| 38 | Phase 2 subtasks 2.1–2.2 (CLI shims + gate parameter wiring) |
| 39 | Phase 2 subtasks 2.3–2.4 (sample receipts + skill doc); Phase 4 subtask 4.1 in parallel |
| 40 | Phase 3 subtasks 3.1–3.2 (workflow + branch protection); Phase 4 subtasks 4.2–4.4 |
| 41 | Phase 4 subtasks 4.5–4.6; Phase 3 unblock pending #103 |
| 42 | Phase 3 final acceptance (first real Kimi receipt landed) |

## Rollback plan

| Phase | Rollback |
|-------|----------|
| 2 | Revert gate ps1; CLIs are additive and harmless if unused |
| 3 | Remove `proof-gate` from required-checks list; workflow file can stay disabled via `if: false` |
| 4 | Feature-flag HMAC verification client-side via `DINOFORGE_BRIDGE_HMAC=0` env var; server still emits receipts (forward-compatible) |
