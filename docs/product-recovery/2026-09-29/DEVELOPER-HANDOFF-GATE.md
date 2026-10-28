# Civis developer-agent handoff gate

Date2026-09-30.

## READY NOW — bounded experimental work only

A developer agent may:
- run/extend recovery persistence oracles;
- evolve the test-only semantic state manifest;
- mechanically enumerate state owners/components;
- prototype v6 component encoding/migration in tests without changing default save format.

It may NOT yet:
- flip CIVSAVE_FORMAT_VERSION;
- silently default missing durable state;
- delete legacy support;
- claim save/load fixed;
- merge spec PR.

## Preconditions for production v6 handoff

1. Existing reproduced reds remain bound to baseline.
2. Semantic manifest prototype passes policy input, control policy kind, research, market and mod-compatibility controls.
3. Authoritative-state manifest classifies all future-affecting Simulation/ECS owners.
4. v5->v6 missing-state policy is explicit; no invented values hidden as faithful restore.
5. Mod artifact identity strategy resolved beyond id/version where possible.
6. Metadata downgrade classifier and atomic commit/reconciliation strategy experimentally defined.
7. Local/server save UI authority and mounted save path traced.
8. Catalog authority blockers do not contaminate save requirements.

## Proposed implementation WPs after gate

C-SAVE6-01: semantic_state component schema + version.
C-SAVE6-02: policy input/control policy/research/market adapters.
C-SAVE6-03: active mod-set identity + guest-memory compatibility.
C-SAVE6-04: outer current-vs-legacy classifier.
C-SAVE6-05: atomic archive/slot commit + DB/filesystem reconciliation.
C-SAVE6-06: v5 adapter/migration receipts preserving source bytes.
C-SAVE6-07: mounted local/server UI save/load authority.
C-SAVE6-08: state-denominator expansion and regression suite.
C-SAVE6-09: user journey save/restart/recovery evidence.

No implementation agent may use the synthetic requirement ranges or unresolved namespace collisions as completion denominators.
