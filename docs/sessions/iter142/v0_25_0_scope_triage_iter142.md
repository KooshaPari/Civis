# v0.25.0 Scope Triage (Iter-142 Retrospective)

**Release Target**: v0.25.0-dev → v0.25.0 (tag)  
**Date**: 2026-05-18  
**Status**: TAG-APPROVED (iter-133 FULL GREEN baseline holding)

---

## (A) MUST Land Before Tag

| Task | Effort | Rationale |
|------|--------|-----------|
| **#523** EconomyContentLoader test regression (9 tests expect `InvalidDataException`, prod throws `ArgumentException`) | 0.5h | Pattern #95/#210 IValidatable drift; fix in flight (iter-128 agent reference: a7eb4ac4f96342a56). Blocks SDK pre-release validation. |
| **#524** PreToolUse hooks fire-behavior verification under real harness | 1h | Hooks wired to settings.json (block-git-stash.ps1, guard-git-worktree.ps1) but stdin-pipe behavior unverified. Safety critical (iter-142 governance hardening). Smoke test inconclusive. |
| **Merge fix/handle-connect-iter142 → main** (51 commits, 282-file intersection) | 2h | GameClient.cs, JsonRpcMessage.cs (3-phase explicit merge). Blocks game recovery + HandleConnect deployment. HIGH conflict density but resolvable. |

**Subtotal**: 3.5h estimated effort

---

## (B) NICE to Land (Quality Polish)

| Task | Rationale |
|------|-----------|
| **#269** P3 Pattern #96 Roslyn analyzer | Late-stage design change detector; non-critical for release. Defer post-tag if cycle time is tight. |
| **#515** P2 benchmarks.yml + asset-pipeline.yml path mismatch | 6 broken CI refs (Pattern #86 false-completion examples). Improves CI observability but not a blocker. Queued for v0.26.0 sweep. |

---

## (C) Defer to v0.26.0

| Task | Rationale |
|------|-----------|
| **#101** P0 Star Wars asset 0/36 units render | Blocked on headless infra path (#188, #425 research dispatched). Not a v0.25.0 release blocker; visual parity feature. |
| **#103** P3 Kimi MOONSHOT_API_KEY runbook E2E | External blocker (Kimi auth). Judge-receipt gating. Scheduled post-tag. |
| **#505** P2 Pattern #231 static-init audits (11 HIGH, promote DF1028) | Audit complete; HIGH instances 11; promote DF1028 analyzer for v0.26.0 sweep (iter-141 wave queued). |
| **#507** P0 + **#510** P1 + **#512** P0 Branch consolidation coordination | Stash recovery (#510) + branch-naming strategy (#507/#512) are post-merge housekeeping tasks. Defer after main integration. |

---

## (D) Single Critical-Path Blocker

**→ MERGE fix/handle-connect-iter142 → main**  

This merge gates:
- Game recovery (user manual launch → HandleConnect live)
- v0.25.0 tag push (v0.25.0-dev → v0.25.0)
- PR CI validation (safety/iter140 snapshot + iter-142 audit docs live)

**Conflict Strategy**:
1. **GameClient.cs**: 3-way merge (OnFatalError handler placement).
2. **JsonRpcMessage.cs**: Properties vs. public field migration; take main (already migrated in iter-131).
3. **VERSION**: Accept fix branch (0.25.0-dev is same as main).

**ETA**: ~1h via phase-3 explicit merge (test after each phase).

---

## (E) Estimated Effort Summary

**MUST-land effort**: 3.5 hours  
- EconomyContentLoader regression fix: 0.5h
- PreToolUse hook smoke test: 1h
- Merge conflict resolution (3-phase): 2h

**Release-ready trigger**: All (A) + merge complete → git tag v0.25.0 + release.yml auto-fire.

---

**Go/No-Go Decision**: **GO** for tag once merge phase-3 completes and #523/#524 verified passing.
