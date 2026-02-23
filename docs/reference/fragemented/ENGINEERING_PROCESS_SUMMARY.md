# CivLab Engineering Process & Infrastructure — Complete Specification

**Date:** 2026-02-21
**Status:** Complete
**Total Documentation:** 2,462 lines across 5 files
**Architecture:** Modular Rust workspace with 8 crates, determinism-first, test-first TDD

---

## Files Created & Updated

### 1. `/Users/kooshapari/temp-PRODVERCEL/485/kush/civ/PLAN.md` (276 lines)

**Purpose:** Complete phased work breakdown structure (WBS) with DAG dependencies and critical path analysis

**Contents:**
- Phase diagram showing sequential (0→1→2→4→5→6) and parallel (3) execution
- 6 phases spanning ~28-35 days wall-clock time
- 48 atomic tasks (P0.1 → P6.8) with:
  - Task ID, description, dependencies
  - L3 copilot dispatch commands (fully templated)
  - Acceptance criteria (all measurable)
  - Git worktree setup instructions

**Key Phases:**
- **Phase 0:** Core tick loop & determinism (5 days, critical path blocker)
- **Phase 1:** Economy + Joule allocators (5-7 days, depends on P0)
- **Phase 2:** Actors + institutions + timeseries (6-8 days, depends on P1)
- **Phase 3:** WebSocket server + Bevy client (5-7 days, **parallel** with P1-2)
- **Phase 4:** War + diplomacy + shadow networks (7-9 days, depends on P2)
- **Phase 5:** Scenario runner + metrics export + replay format (5-7 days, depends on P4)
- **Phase 6:** Coverage, performance, hardening (7-10 days, depends on P5)

**Critical Path:** 0 → 1 → 2 → 4 → 5 → 6 (~28-35 days)
**Parallelizable:** Phase 3 independent, can run alongside 1-2

**Target Coverage:** 100% engine, 80%+ others
**Target Performance:** Benchmarks with criterion
**Git Strategy:** One worktree per phase

---

### 2. `/Users/kooshapari/temp-PRODVERCEL/485/kush/civ/docs/guides/TEST_FIRST_GUIDE.md` (765 lines)

**Purpose:** Comprehensive test-first (TDD) mandate and implementation guide

**Contents:**

**Core Cycle:**
1. Write failing test (must fail)
2. Implement feature (until test passes)
3. Refactor (optional, maintain tests)

**Test Organization:**
- Naming: `crates/{crate}/tests/fr_{id}.rs`
- Categories:
  - Unit tests (pure functions)
  - Integration tests (crate boundaries)
  - Scenario tests (full sim runs)
  - Replay tests (determinism)
  - Property-based tests (invariants via proptest)
  - Snapshot tests (golden file regression)

**Coverage Targets:**
- Engine: 100% (line + branch)
- Economy: 100% (line + branch)
- Others: 80%+ (line coverage)

**Cargo Organization:**
```bash
cargo test --all                          # All
cargo test --package civ-economy          # By crate
cargo test --test fr_core_tick_loop       # Single file
cargo test -- --test-threads=1            # Determinism
```

**Property-Based Testing Examples:**
- Joule allocation never exceeds budget
- Market prices stay bounded
- No negative balances

**Copilot L3 Pattern:**
1. Phase 1: Dispatch agent to write failing test
2. Verify test fails
3. Phase 2: Dispatch agent to implement
4. Verify test passes
5. Commit both atomically

**Complete Example:** Full FR-CIV-ECON-001 from test to implementation

---

### 3. `/Users/kooshapari/temp-PRODVERCEL/485/kush/civ/docs/guides/GIT_WORKTREE_GUIDE.md` (637 lines)

**Purpose:** Isolated parallel development strategy using git worktrees

**Key Concepts:**

**Worktree Architecture:**
```
civ/                              # Main (never edit during phases)
civ-wt-phase0-foundation/         # Independent working directory
civ-wt-phase1-economy/            # Can run parallel to phase0 merge
civ-wt-phase3-client/             # Independent from 1-2
```

**Creation:**
```bash
git worktree add ../civ-wt-phase{N}-{name} main
cd ../civ-wt-phase{N}-{name}
git checkout -b feat/civ-phase{N}-{feature}
```

**Branch Naming:** `feat/civ-{PHASE}-{FEATURE}`
- Example: `feat/civ-phase0-tick-loop`

**Commit Convention:** `{type}({crate}): {FR-ID} {description}`
- Types: `feat`, `test`, `fix`, `refactor`, `docs`, `perf`
- Example: `test(engine): FR-CIV-0001-TICK failing test`

**Merge Workflow:**
1. Task complete in worktree
2. Quality gate passes
3. Create PR on GitHub
4. Merge (squash/rebase)
5. Delete worktree locally
6. Next phase pulls updated main

**Conflict Resolution:**
- Spec files: go through main only
- Implementation: can merge parallel if different crates
- If conflict: rebase on origin/main

**Anti-Patterns:**
- Don't stay on main in worktree
- Don't merge between worktrees
- Don't push from worktree (only via PR)

**Troubleshooting:**
- Lock file error → `rm .git/worktrees/.lock && git worktree prune`
- Out of sync → `git fetch origin main && git rebase origin/main`

---

### 4. `/Users/kooshapari/temp-PRODVERCEL/485/kush/civ/docs/guides/COPILOT_L3_AGENTS.md` (705 lines)

**Purpose:** Complete guide to dispatching and monitoring L3 copilot agents

**L3 Definition:**
- Autonomous agent (no human in loop)
- Reads code, writes files, runs cargo/git, makes commits
- Bounded scope with explicit permissions
- ~50+ agents across all phases

**Basic Dispatch:**
```bash
copilot -p "Task description" \
  --yolo --model gpt-5-mini \
  --add-dir /path/to/civ \
  &
```

**With Permissions (Recommended):**
```bash
copilot -p "..." \
  --allow-tool 'write' \
  --allow-tool 'shell(cargo:*)' \
  --allow-tool 'shell(git:*)' \
  --deny-tool 'shell(git push)' \
  &
```

**Task Prompt Template:**
```
TASK: {Name}
DESCRIPTION: {Summary}
REQUIREMENTS: {Bulleted list}
CONSTRAINTS: {What NOT to do}
ACCEPTANCE CRITERIA: {Measurable outcomes}
CONTEXT: {Links to specs/docs}
```

**Batch Dispatch Script:**
```bash
#!/bin/bash
for task in "${TASKS[@]}"; do
  copilot -p "$task" --yolo --model gpt-5-mini ... &
done
wait
```

**Monitoring:**
```bash
copilot session list                    # All sessions
copilot session watch {id}              # Real-time logs
copilot session log {id} --tail 50      # Last 50 lines
git log --oneline                       # Check commits
cargo test --package {crate}            # Verify tests
```

**Allowed Tools:**
- `read`, `write`
- `shell(cargo)`, `shell(git)`
- `shell(git commit)`, `shell(git add)`

**Denied Tools:**
- `shell(git push)` — only via PR
- `shell(git reset)` — loses work
- `shell(rm)` — dangerous
- `shell(sudo)` — privilege escalation

**Failure Handling:**
- Timeout: check logs, clean build, redispatch
- Compilation error: fix and redispatch
- Test failure: review test, fix implementation
- Wrong commit: reset --soft, redispatch

**Phase 0 Example:**
1. Dispatch P0.1: Write failing test for tick loop
2. Verify test fails
3. Dispatch P0.2: Implement tick loop
4. Verify test passes
5. Continue to P0.3 (determinism replay)

**Full Phase Dispatch:**
```bash
scripts/dispatch-phase1-full.sh
# Dispatches 4-8 agents in parallel
# Limits to 4 concurrent to avoid system load
```

---

### 5. `/Users/kooshapari/temp-PRODVERCEL/485/kush/civ/process-compose.yaml` (79 lines, updated)

**Purpose:** Local development infrastructure orchestration

**Services Added/Updated:**

**sim-server:**
- Runs `cargo run -p civ-server --release`
- Listens on 0.0.0.0:8080
- Restarts on failure (max 5 restarts)
- Readiness probe via netstat + HTTP health check
- Logs to `.process-compose/logs/sim-server.log`
- Timeout: 10s

**metrics-collector:**
- Polls `/metrics` endpoint every 10 seconds
- Saves metrics snapshots to `/tmp/civ-metrics-{timestamp}.txt`
- Depends on sim-server being healthy
- Restart policy: 3 max, 5s backoff
- Logs to `.process-compose/logs/metrics-collector.log`

**replay-validator:**
- Runs deterministic replay tests (`fr_determinism_replay.rs`)
- Waits for sim-server to stabilize
- Single-threaded (`--test-threads=1`) for consistency
- Runs validation every 60 seconds
- Logs to `.process-compose/logs/replay-validator.log`
- Restart policy: 2 max, 10s backoff

**Environment Variables:**
```
RUST_LOG=info
CIV_SERVER_HOST=0.0.0.0
CIV_SERVER_PORT=8080
CIV_METRICS_PORT=9090
```

**Usage:**
```bash
task infra:up       # Start all services
task infra:down     # Stop all services

# Manual
process-compose -f process-compose.yaml up -d
process-compose -f process-compose.yaml logs -f sim-server
process-compose -f process-compose.yaml down
```

**Integration with PLAN.md:**
- Services start automatically during Phase 3+ (server integration)
- Metrics collector enables Phase 5 (research API)
- Replay validator ensures Phase 0 (determinism) throughout execution

---

## Summary: Complete Engineering Stack

### Methodology
✅ Test-First (TDD) — 100% engine, 80%+ other coverage
✅ Spec-Driven — All work references CIV-#### specs
✅ Determinism-First — Replay validation via process-compose
✅ L3 Agent Workforce — ~50+ copilot agents, worktree-isolated
✅ Parallel Phases — Phase 3 independent, others sequential with clear deps

### Execution Model
✅ 6 phases over ~28-35 days
✅ Phase 0 blocks critical path; others have clear dependencies
✅ 48 atomic tasks (P0.1 → P6.8)
✅ Each task has failing test → implementation → refactor cycle
✅ Every task dispatches to L3 copilot agent with explicit constraints

### Infrastructure
✅ Git worktrees (one per phase, no interference)
✅ Process-compose services (server, metrics, replay validation)
✅ Cargo workspace (8 crates, tach.toml boundaries)
✅ CI quality gates (fmt, clippy, test, coverage, spec:validate)

### Documentation
✅ PLAN.md: 276 lines, phase WBS with full dispatch commands
✅ TEST_FIRST_GUIDE.md: 765 lines, TDD mandate + examples
✅ GIT_WORKTREE_GUIDE.md: 637 lines, isolation strategy
✅ COPILOT_L3_AGENTS.md: 705 lines, agent dispatch patterns
✅ process-compose.yaml: 79 lines, 3 services (server, metrics, replay)

### Total Lines Written: 2,462

---

## How to Use These Documents

### For Planning (PLAN.md)
1. Read "Critical Path Summary" to understand dependencies
2. Use "Phase Diagram" to visualize parallel execution
3. Copy dispatch commands from each phase table
4. Create git worktree per phase
5. Batch-dispatch L3 agents using commands

### For Development (TEST_FIRST_GUIDE.md)
1. Write failing test per FR (in `tests/fr_{id}.rs`)
2. Implement until test passes
3. Run coverage: `cargo tarpaulin --out Html`
4. Commit with conventional message
5. Verify no suppressions, no TODOs

### For Parallel Work (GIT_WORKTREE_GUIDE.md)
1. Create worktree: `git worktree add ../civ-wt-phase{N} main`
2. Create feature branch: `git checkout -b feat/civ-phase{N}-{name}`
3. Commit with format: `{type}({crate}): {FR-ID} {desc}`
4. When phase complete: `task quality` → PR → merge
5. Delete worktree and pull updated main

### For Agent Dispatch (COPILOT_L3_AGENTS.md)
1. Use task prompt template from section "Task Prompt Template"
2. Copy dispatch command and customize:
   - Add `--add-dir {worktree_path}`
   - Set `--model gpt-5-mini` for cost
   - Deny `shell(git push)`, `shell(rm)`
3. Dispatch with `&` to background
4. Monitor via `copilot session list` and `git log`
5. Handle failures per section "Handling Agent Failures"

### For Infrastructure (process-compose.yaml)
1. Run `task infra:up` to start services
2. Monitor logs: `process-compose logs -f sim-server`
3. Metrics available at `http://localhost:9090/metrics`
4. Replay validator runs automatically (checks determinism)
5. Run `task infra:down` to stop

---

## Success Criteria

- ✅ All phases execute in sequence (or parallel where allowed)
- ✅ All 48 tasks have passing tests + green commits
- ✅ 100% coverage on engine, 80%+ on others
- ✅ No new lint suppressions without inline justification
- ✅ All commits follow conventional message format
- ✅ Deterministic replay tests pass (single-threaded)
- ✅ WebSocket server accepts client connections
- ✅ Scenario YAML format defined and tested
- ✅ Metrics export (CSV) working
- ✅ Benchmarks published and tracked
- ✅ All docs built and deployable

---

## Files Reference

| File | Location | Purpose | Lines |
|------|----------|---------|-------|
| PLAN.md | `/civ/PLAN.md` | Phases 0-6 with DAG and dispatch commands | 276 |
| TEST_FIRST_GUIDE.md | `/civ/docs/guides/TEST_FIRST_GUIDE.md` | TDD mandate + test patterns | 765 |
| GIT_WORKTREE_GUIDE.md | `/civ/docs/guides/GIT_WORKTREE_GUIDE.md` | Isolation + parallel worktree strategy | 637 |
| COPILOT_L3_AGENTS.md | `/civ/docs/guides/COPILOT_L3_AGENTS.md` | L3 agent dispatch + monitoring | 705 |
| process-compose.yaml | `/civ/process-compose.yaml` | Infrastructure services (3 added) | 79 |
| **TOTAL** | — | — | **2,462** |

---

## Next Steps

1. **Review PLAN.md** and verify dependencies align with your crate architecture
2. **Prepare Phase 0 worktree:**
   ```bash
   cd /Users/kooshapari/temp-PRODVERCEL/485/kush/civ
   git worktree add ../civ-wt-phase0-foundation main
   cd ../civ-wt-phase0-foundation
   git checkout -b feat/civ-phase0-tick-loop
   ```
3. **Dispatch Phase 0 agents** using commands from PLAN.md § Phase 0
4. **Monitor via copilot sessions** and git logs
5. **On Phase 0 completion:** Merge to main, create Phase 1 worktree, repeat

---

**Document Version:** 1.0
**Last Updated:** 2026-02-21 06:15 UTC
**Status:** Ready for execution

