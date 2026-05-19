# CI Manifest-Gate / Smart-Contract Reality Audit

**Date**: 2026-04-25
**Auditor**: Infra audit subagent (iteration 37, parallel batch of 5)
**Scope**: Pre-commit hooks, pre-push hooks, GitHub Actions workflows,
`prove-features-gate.ps1`, judge receipts, manifest signing, merkle bundles,
policy files
**Verdict**: THEATER

---

## Executive summary

DINOForge advertises a "smart-contract-like" verification system: every
feature must be backed by a signed manifest receipt with a merkle root over
the bundle of artifacts (screenshot + log-tail + judge output) and an external
(non-Anthropic) judge model. The advertisement is ahead of reality on every
axis:

- **0 of 24** GitHub Actions workflows launch the real game.
- `prove-features-gate.ps1` exists at `.claude/commands/`, contains real
  signature/judge-allowlist logic, but is **never invoked** from any workflow
  or hook. It is a dead script.
- No manifest signing key (sigstore or local ed25519) is committed.
- No merkle-root computation is wired into the proof bundle layout.
- No `policy.yaml` declaring required claims, allowed judge models, or max
  receipt age exists.
- The `docs/proof/judge-receipts/` directory **does not exist on disk**.
- Workflows that look like real-game gates (`game-launch.yml`,
  `ui-automation.yml`) target a self-hosted runner that is not provisioned.

The cumulative effect is a verification system that looks complete from CLAUDE.md
and looks complete from the README, but executes nothing. Every claim of
"VLM-confirmed" or "all green" is therefore self-judged or compile-only.

---

## Findings table

| Component | Claim | Reality | Status |
|-----------|-------|---------|--------|
| `ci.yml` catch-all gate | Runs all test categories | True — runs ~2,500 tests, no `--filter` | WORKS (compile/unit only) |
| Specialized workflows (`fuzz`, `policy-gate`, etc.) | Filter by category | True — `Category=X` traits | WORKS (compile/unit only) |
| `game-launch.yml` | Launches real game on self-hosted runner | Self-hosted runner not provisioned | THEATER |
| `ui-automation.yml` | Drives in-game UI on self-hosted runner | Self-hosted runner not provisioned | THEATER |
| `prove-features-gate.ps1` | Rejects Claude-judged receipts | Real logic; never invoked from CI | ORPHAN |
| Bundle merkle root | Manifest hashing of artifacts | Not implemented | VAPORWARE |
| Receipt signing | sigstore cosign or ed25519 | No key committed, no signing step | VAPORWARE |
| `policy.yaml` | Declares required claims, judge models | Does not exist | VAPORWARE |
| Hash-chain linking | Each receipt references prior | Not implemented | VAPORWARE |
| `docs/proof/judge-receipts/` | Directory of signed receipts | Directory does not exist on disk | VAPORWARE |
| Pre-commit hook | Blocks commits without receipt | No such hook | VAPORWARE |
| Pre-push hook | Blocks push without receipt | No such hook | VAPORWARE |

---

## Concrete gaps

1. **Zero workflows launch the real game.** All 24 workflows are
   compile/lint/unit-test only. `game-launch.yml` and `ui-automation.yml`
   reference `runs-on: self-hosted` but no self-hosted runner exists in the
   GitHub org. The workflows therefore queue forever or are skipped — they do
   not gate anything.

2. **`prove-features-gate.ps1` is never invoked.** The script contains real
   logic (rejects judge model strings starting with `claude-` or `codex-`),
   but no workflow calls it, no pre-commit hook runs it, and no skill
   integrates it. It is dead code.

3. **No signature trust root.** No public key is committed under `keys/` or
   `.github/`. No sigstore identity is documented. No ed25519 keypair has been
   generated. There is therefore nothing to verify against.

4. **No merkle-root scheme.** The proof bundle layout is "screenshot + log + judge
   output as loose files in a timestamped dir". There is no manifest file
   listing the artifacts, no canonical hash order, no merkle tree, no root
   to sign.

5. **No `policy.yaml`.** A "smart contract" needs a policy: which features
   require receipts, which judge models are accepted, how stale a receipt may
   be, what claim shape is mandatory. None of this is captured anywhere.

6. **`docs/proof/judge-receipts/` directory missing.** Multiple docs reference
   this path. It does not exist on disk. There has therefore never been an
   external judge receipt persisted, period.

7. **No hash-chain linking between receipts.** A meaningful smart-contract
   chain would link receipt N to receipt N-1 by hash. There is no such
   linkage.

8. **No bridge HMAC.** The bridge accepts unauthenticated JSON-RPC over the
   loopback socket. If a CI gate is going to read bridge state, the bridge
   needs to authenticate the caller. Not implemented.

---

## Tasks opened from this audit

- **#191 P0 INFRA** — Smart-contract proof system spec. Design landed in
  `docs/design/2026-04-25-smart-contract-proof-system.md` (391 lines).
  Phase 1 implementation (signing tooling) is open and must be chunked into
  3 dispatches.
- **#192 P2 INFRA** — Wire `prove-features-gate.ps1` into CI. Drop or
  conditionally-skip `game-launch.yml` until a self-hosted runner is
  provisioned. Depends on #191 phases.

User-facing dependency: provide `MOONSHOT_API_KEY` for first external Kimi
receipt, AND decide between sigstore (cosign keyless, requires GitHub
identity) and local ed25519 keypair for signing.

---

## See also

- `docs/sessions/2026-04-25-infra-pivot-plan.md` — Wave 2 (#191) and Wave 3 (#192) own this audit's tasks
- `docs/design/2026-04-25-smart-contract-proof-system.md` — Wave 2 spec
- `docs/sessions/2026-04-25-bridge-bypass-audit.md` — bridge HMAC gap is shared with that audit
- `docs/sessions/2026-04-25-steamless-multi-instance-audit.md` — without working sandbox, no game-launch CI is meaningful
- `docs/sessions/2026-04-25-sandbox-isolation-audit.md` — same dependency
- `docs/TRUTH_TABLE.md` — every "❌ STUB" / "🟡 PARTIAL" row will move once Waves 2-3 land

---

## Verdict

The CI manifest-gate / smart-contract layer is **theater**. The pieces are
named correctly (gate script, judge allowlist, judge receipts directory) but
none are wired together and the directory itself is missing. Until Wave 2
lands the spec implementation and Wave 3 wires it into CI, every "green CI"
claim must be qualified to "compile + unit + parameterized tests pass; no
real-game gate, no signed receipt, no policy enforcement".
