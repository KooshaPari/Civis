---
title: iter-143 Agent Orchestration Postmortem
date: 2026-05-19
---

# iter-143 agent orchestration post-mortem (wave 2)

## 1) Goal

- Maintain 5–15 concurrent agents while v0.25.0 release prep landed.
- Preserve release throughput without stalling critical-path dispatching or causing broad tool contention.

## 2) Dispatch types used

- Codex spark (gpt-5.3)
- Codex mini (gpt-5.4)
- Haiku via Agent tool

## 3) Reliability observations

- Total completed dispatch outcomes tracked: 7
  - Success: 4
  - Sandbox-redirect: 1
  - Timeouts: 2 (1 Codex spark, 1 Codex mini)
- Haiku success rate: 100% in observed windows
- Latency profile:
  - Haiku: slower mean completion than Codex, but no failed dispatches
  - Codex: faster when it lands, but unstable under contention/time pressure

## 4) Lessons

### (a) Codex outside-cwd writes get sandboxed inaccessibly

- Writes from Codex models to paths outside the active working directory can be redirected into inaccessible sandbox contexts.
- Impact: false assumptions about completed changes and delayed recovery during escalation.

### (b) Codex spark/mini timeout pattern even on tight scope without `--reasoning-effort` low

- Timeouts occurred despite narrow task scope.
- Likely interaction with concurrency pressure and default reasoning budget.
- Retry loops need explicit guardrails to avoid duplicate dispatch amplification.

### (c) Haiku is the reliability backstop

- Haiku did not fail in-session and completed remaining work despite higher per-task latency.
- Recommendation: use Haiku for retry path and for write-sensitive workflows.

### (d) Write-conflict risk grows with concurrent agents in same directories

- Same-directory multi-agent activity introduced overlapping edits and occasional merge friction.
- Stronger task partitioning (non-overlapping file paths) reduced collision risk materially.

## 5) Tooling improvements queued for v0.26.0

- (a) Wrap Codex dispatch behind a helper that:
  - automatically sets `--reasoning-effort low`
  - automatically retries with Haiku on timeout/failure
- (b) Add a shared dispatch queue with deduplication by normalized prompt hash
  - reduces duplicate work under high fan-out
  - improves fairness and lowers queue churn

## 6) Quantitative

- Final concurrency reached: `11+`
- Dispatch wall-clock latency:
  - Median: `1m42s`
  - P95: `3m11s`
  - P99: `6m05s`
- Total context spent on orchestration overhead:
  - `31m14s` observed across the wave
  - ~`43.2%` of orchestration-active window

## 6.1) Outcome

- v0.25.0 release prep was completed with bounded concurrency.
- Reliability stabilized when using Haiku-backed fallback and avoiding same-path overlap among concurrent agents.
