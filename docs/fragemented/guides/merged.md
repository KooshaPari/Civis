# Merged Fragmented Markdown

## Source: guides/COPILOT_L3_AGENTS.md

# Copilot L3 Agents Guide for CivLab

**Purpose:** Define patterns and best practices for dispatching L3 copilot agents to implement CivLab features
**Level:** L3 = autonomous agent, write code, run tests, make commits (no human in loop)
**Safety:** Scoped permissions, isolated worktrees, explicit constraints
**Monitoring:** Check git logs and copilot session logs

---

## What is L3?

L3 = Autonomous Agent Level 3:
- **Reads code:** ✅ Full codebase access
- **Writes code:** ✅ Can create/edit files
- **Runs commands:** ✅ Can run cargo, git, tests
- **Makes commits:** ✅ Can `git commit` (signed or unsigned)
- **No human loop:** ✅ Completes task without asking
- **Bounded scope:** ✅ Explicit constraints (--deny-tool rules)

L3 agents are used for:
- Writing failing tests (test-first phase)
- Implementing features (code phase)
- Refactoring (cleanup phase)
- Running quality gates (validation phase)

L3 agents are NOT used for:
- Pushing to remote (that's L4+)
- Making PRs (human creates PR from worktree changes)
- Deleting files (too risky)
- Modifying specs (specs handled separately)

---

## Basic Dispatch Pattern

### Simplest Form

```bash
copilot -p "Implement FR-CIV-ECON-001: Market struct" \
  --yolo --model gpt-5-mini \
  --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &
```

### With Permissions (Recommended)

```bash
copilot -p "Implement FR-CIV-ECON-001: Market struct" \
  --yolo --model gpt-5-mini \
  --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ \
  --allow-tool 'write' \
  --allow-tool 'shell(cargo:*)' \
  --allow-tool 'shell(git:*)' \
  --deny-tool 'shell(git push)' \
  --deny-tool 'shell(rm)' \
  &
```

### With Full Constraints (Most Explicit)

```bash
copilot -p "Implement FR-CIV-ECON-001" \
  --yolo \
  --model gpt-5-mini \
  --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ \
  --allow-tool 'read' \
  --allow-tool 'write' \
  --allow-tool 'shell(cargo)' \
  --allow-tool 'shell(git)' \
  --deny-tool 'shell(*)' \
  --deny-tool 'shell(rm)' \
  --deny-tool 'shell(git push)' \
  --deny-tool 'shell(git reset)' \
  --timeout 600 \
  &
```

### Batch Dispatch (Multiple Agents)

```bash
#!/bin/bash
# scripts/dispatch-phase1.sh

set -e

CIV_DIR="/Users/kooshapari/temp-PRODVERCEL/485/kush/civ"
MODEL="gpt-5-mini"

# Array of tasks for Phase 1
declare -a TASKS=(
  "Write failing test for FR-CIV-ECON-001-MARKET in crates/economy/tests/fr_econ_market.rs. Test must verify Market struct exists and price_update() works. Commit: 'test(economy): FR-CIV-ECON-001 failing test'"
  "Implement FR-CIV-ECON-001-MARKET: Market price tracking in crates/economy/src/market.rs. Must pass all tests in crates/economy/tests/fr_econ_market.rs. Commit: 'feat(economy): FR-CIV-ECON-001 market'"
  "Write failing test for FR-CIV-ECON-002-JOULE in crates/economy/tests/fr_econ_joule.rs. Test Joule allocation conserves energy. Commit: 'test(economy): FR-CIV-ECON-002 failing test'"
  "Implement FR-CIV-ECON-002-JOULE: Joule allocator in crates/economy/src/joule.rs. Must pass all tests. Commit: 'feat(economy): FR-CIV-ECON-002 joule'"
)

# Dispatch each task
for i in "${!TASKS[@]}"; do
  task="${TASKS[$i]}"
  echo "[$(date)] Dispatching task $((i+1))/${#TASKS[@]}: ${task:0:50}..."

  copilot -p "$task" \
    --yolo --model "$MODEL" \
    --add-dir "$CIV_DIR" \
    --allow-tool 'write' \
    --allow-tool 'shell(cargo:*)' \
    --allow-tool 'shell(git:*)' \
    --deny-tool 'shell(git push)' \
    &
done

# Wait for all background tasks to complete
wait
echo "[$(date)] All tasks dispatched and completed"
```

**Run:**
```bash
chmod +x scripts/dispatch-phase1.sh
./scripts/dispatch-phase1.sh
```

---

## Task Prompt Template

Use this template for consistent, high-quality prompts:

```bash
copilot -p "
TASK: {Task Name}

DESCRIPTION:
{1-2 sentence description of what to implement}

REQUIREMENTS:
- File(s) to create/modify: {file list}
- Function/struct signatures: {API}
- Invariants to maintain: {invariants}
- Test file(s): {test files to make pass}
- Commit message: '{conventional commit message}'

CONSTRAINTS:
- Do NOT modify {files/modules to avoid}
- Must pass: cargo test --package {crate}
- Must pass: cargo clippy -- -D warnings
- Maximum function length: 40 lines
- Do NOT add .unwrap() without clear safety justification

ACCEPTANCE CRITERIA:
✓ Test(s) pass: cargo test --test {test_file}
✓ Clippy clean: cargo clippy --package {crate} -- -D warnings
✓ Correct commit message format
✓ No new warnings or errors

CONTEXT:
See docs/specs/CIV-{spec_id}.md for detailed spec
Relate to PLAN.md task {task_id}
Reference {other related files}
" \
  --yolo --model gpt-5-mini \
  --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ \
  --allow-tool 'write' \
  --allow-tool 'shell(cargo:*)' \
  --allow-tool 'shell(git:*)' \
  --deny-tool 'shell(git push)' \
  &
```

---

## Phase 0 Dispatch Example

**Phase 0: Foundation** — Core tick loop

### Task P0.1: Write Failing Test

```bash
copilot -p "
TASK: P0.1 — Core Tick Loop Failing Test

DESCRIPTION:
Write a failing test for the core simulation tick loop.

REQUIREMENTS:
- Create file: crates/engine/tests/fr_core_tick_loop.rs
- Test function name: test_simulation_tick_increments_turn
- Test must verify:
  * Simulation::new() creates a simulation with turn = 0
  * sim.tick() increments the turn counter
  * sim.current_turn() returns the updated turn number
- Test MUST FAIL right now (Simulation struct doesn't exist yet)
- Do NOT implement Simulation yet, only write the test

CONSTRAINTS:
- Do NOT implement the Simulation struct
- Do NOT create crates/engine/src/simulation.rs
- Do NOT modify any existing source files
- Only create the test file

ACCEPTANCE CRITERIA:
✓ File crates/engine/tests/fr_core_tick_loop.rs exists
✓ Test fails: 'error: cannot find struct Simulation'
✓ Commit: 'test(engine): FR-CIV-0001-TICK failing test'

CONTEXT:
See docs/specs/CIV-0001-core-simulation-loop.md
This is the foundation test for Phase 0
Relates to PLAN.md task P0.1
" \
  --yolo --model gpt-5-mini \
  --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &
```

### Task P0.2: Implement Tick Loop

```bash
copilot -p "
TASK: P0.2 — Core Tick Loop Implementation

DESCRIPTION:
Implement the Simulation struct and its tick() method.

REQUIREMENTS:
- Create file: crates/engine/src/simulation.rs
- Implement struct:
  struct Simulation {
    turn: u64,
  }
- Implement methods:
  * pub fn new() -> Self { Self { turn: 0 } }
  * pub fn tick(&mut self) -> Result<()> { self.turn += 1; Ok(()) }
  * pub fn current_turn(&self) -> u64 { self.turn }
- Export from crates/engine/src/lib.rs
- Test must pass: cargo test --package civ-engine test_simulation_tick_increments_turn

CONSTRAINTS:
- Do NOT add any RNG logic yet (that's P0.6)
- Do NOT add any policy/economy logic
- Keep implementation under 20 lines
- Fail on any tick() errors (don't use unwrap but do propagate via Result)

ACCEPTANCE CRITERIA:
✓ Test passes: cargo test --package civ-engine test_simulation_tick_increments_turn
✓ Clippy clean: cargo clippy --package civ-engine -- -D warnings
✓ Code under 20 lines
✓ Commit: 'feat(engine): FR-CIV-0001-TICK core tick loop'

CONTEXT:
Test already exists in crates/engine/tests/fr_core_tick_loop.rs
Make it pass by implementing Simulation struct
See docs/specs/CIV-0001-core-simulation-loop.md
Relates to PLAN.md task P0.2
" \
  --yolo --model gpt-5-mini \
  --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &
```

---

## Monitoring Agents

### Check Agent Status

```bash
# List all running copilot sessions
copilot session list

# Output:
# ID          Task                              Status      Started   Duration
# c0d1e2f3    Implement FR-CIV-ECON-001         RUNNING     14:32     2m 15s
# a1b2c3d4    Write test for FR-CIV-ECON-002    COMPLETED   14:25     1m 48s
```

### Monitor Specific Session

```bash
# Watch logs in real-time
copilot session watch c0d1e2f3

# View full log
copilot session log c0d1e2f3

# Tail last 50 lines
copilot session log c0d1e2f3 --tail 50
```

### Check Git Commits

```bash
cd /Users/kooshapari/temp-PRODVERCEL/485/kush/civ-wt-phase1-economy

# See commits made by agents
git log --oneline | head -20

# See commit details
git show 1a2b3c4

# See what changed
git log --stat | head -30
```

### Verify Tests Passed

```bash
cd /Users/kooshapari/temp-PRODVERCEL/485/kush/civ-wt-phase1-economy

# Run same tests agent ran
cargo test --package civ-economy

# Run specific test
cargo test --package civ-economy --test fr_econ_market test_market_price_increase
```

---

## Agent Permissions & Safety

### Recommended Permissions Set

```bash
copilot -p "Task..." \
  --allow-tool 'read' \
  --allow-tool 'write' \
  --allow-tool 'shell(cargo)' \
  --allow-tool 'shell(git)' \
  --deny-tool 'shell(git push)' \
  --deny-tool 'shell(git reset)' \
  --deny-tool 'shell(rm)' \
  --deny-tool 'shell(rm -rf)' \
  &
```

### Allowed Tools

| Tool | Purpose | Safe? |
|------|---------|-------|
| `read` | Read files | ✅ Yes |
| `write` | Create/edit files | ✅ Yes (if scope limited) |
| `shell(cargo)` | Run cargo commands | ✅ Yes |
| `shell(git)` | Run git commands | ✅ Yes (except push) |
| `shell(git commit)` | Make commits | ✅ Yes (no push) |
| `shell(git add)` | Stage files | ✅ Yes |

### Denied Tools

| Tool | Reason |
|------|--------|
| `shell(git push)` | Pushes to remote (only for PRs) |
| `shell(git reset)` | Loses work |
| `shell(git clean)` | Deletes untracked files |
| `shell(rm)` | Deletes files (risky) |
| `shell(docker)` | Could affect system state |
| `shell(sudo)` | Privilege escalation |

---

## Handling Agent Failures

### Agent Times Out

```
error: agent exceeded timeout (600s)
```

**Solution:**
```bash
# Check what agent was doing
copilot session log {session_id} --tail 100

# Look for where it got stuck (usually cargo build)

# Manually clean and restart
cd ../civ-wt-phase1-economy
cargo clean
git status  # see uncommitted changes

# Redispatch with simpler task
copilot -p "Resume: implement FR-CIV-ECON-001. cargo build is slow, so just run cargo check instead. Then: cargo test --package civ-economy. Commit." ...
```

### Agent Produces Compilation Error

```
error[E0433]: cannot find function `foo` in this scope
```

**Solution:**
```bash
# Check git log
cd ../civ-wt-phase1-economy
git log --oneline -5

# See the broken commit
git show 1a2b3c4

# Redispatch agent to fix
copilot -p "Fix compilation error in crates/economy/src/market.rs. Function foo_bar is called but not defined. Either implement it or remove the call. Run: cargo check --package civ-economy. Commit: 'fix(economy): undefined function'" ...
```

### Agent Makes Wrong Commit

```bash
# Check what was committed
cd ../civ-wt-phase1-economy
git log --oneline -1
git show HEAD

# Reset and redispatch
git reset --soft HEAD~1  # undo commit, keep changes
git status

# Redispatch with clearer instructions
copilot -p "Fix commit message and re-commit. Current message is wrong. Should be 'feat(economy): FR-CIV-ECON-001 market'. Commit with correct message." ...
```

### Test Fails After Implementation

```bash
$ cargo test --package civ-economy test_market_price_increase

---- test_market_price_increase stdout ----
assertion failed: new_price < old_price
```

**Solution:**
```bash
# Review test expectations
cd ../civ-wt-phase1-economy
cat crates/economy/tests/fr_econ_market.rs | grep -A 10 "test_market_price_increase"

# Redispatch agent to fix the implementation
copilot -p "Test test_market_price_increase is failing. Assertion expects new_price < old_price when supply is high and demand is low. Review the price calculation logic in crates/economy/src/market.rs and fix it. Run: cargo test --package civ-economy test_market_price_increase" ...
```

---

## Batch Dispatch Workflow

### Full Phase Dispatch Script

```bash
#!/bin/bash
# scripts/dispatch-phase1-full.sh
# Dispatch all tasks for Phase 1 (Economy Layer)

set -e

CIV_DIR="/Users/kooshapari/temp-PRODVERCEL/485/kush/civ"
MODEL="gpt-5-mini"

# Ensure we're in the right directory
if [[ ! -f "$CIV_DIR/Cargo.toml" ]]; then
  echo "ERROR: CIV_DIR not found at $CIV_DIR"
  exit 1
fi

# Enter phase worktree
PHASE_WT="$CIV_DIR/../civ-wt-phase1-economy"
if [[ ! -d "$PHASE_WT" ]]; then
  echo "Creating phase worktree..."
  cd "$CIV_DIR"
  git worktree add "$PHASE_WT" main
fi

cd "$PHASE_WT"
git checkout -b feat/civ-phase1-economy || true

# Phase 1 tasks (from PLAN.md)
declare -a TASKS=(
  "P1.1|test|P1.1: Write failing test for FR-CIV-ECON-001-MARKET. Create crates/economy/tests/fr_econ_market.rs. Test: market.record_transaction(), market.update_prices(), market.get_price(). Must fail. Commit: 'test(economy): FR-CIV-ECON-001 failing test'"

  "P1.2|impl|P1.2: Implement FR-CIV-ECON-001-MARKET. File: crates/economy/src/market.rs. Implement Market struct with new(), record_transaction(), update_prices(), get_price(). Pass: cargo test --package civ-economy. Commit: 'feat(economy): FR-CIV-ECON-001 market'"

  "P1.3|test|P1.3: Write failing test for FR-CIV-ECON-002-JOULE. Create crates/economy/tests/fr_econ_joule.rs. Test Joule allocation conserves energy. Must fail. Commit: 'test(economy): FR-CIV-ECON-002 failing test'"

  "P1.4|impl|P1.4: Implement FR-CIV-ECON-002-JOULE. File: crates/economy/src/joule.rs. Implement JouleAllocator::allocate(). Energy conservation. Commit: 'feat(economy): FR-CIV-ECON-002 joule'"
)

# Counter
COUNT=0
MAX_CONCURRENT=4

for task_entry in "${TASKS[@]}"; do
  IFS='|' read -r TASK_ID TYPE DESC <<< "$task_entry"

  echo "[$(date '+%Y-%m-%d %H:%M:%S')] Dispatching $TASK_ID ($TYPE): ${DESC:0:80}..."

  copilot -p "$DESC" \
    --yolo \
    --model "$MODEL" \
    --add-dir "$PHASE_WT" \
    --allow-tool 'write' \
    --allow-tool 'shell(cargo:*)' \
    --allow-tool 'shell(git:*)' \
    --deny-tool 'shell(git push)' \
    --timeout 900 \
    &

  ((COUNT++))

  # Limit concurrent agents
  if [[ $((COUNT % MAX_CONCURRENT)) -eq 0 ]]; then
    echo "[$(date)] Waiting for batch $((COUNT / MAX_CONCURRENT)) to complete..."
    wait
  fi
done

# Wait for remaining agents
wait

echo "[$(date)] All Phase 1 tasks dispatched and completed"

# Final quality gate
echo "[$(date)] Running final quality gate..."
cargo test --all
cargo clippy --all -- -D warnings
cargo fmt --all -- --check

echo "[$(date)] Phase 1 complete! Ready for merge."
```

**Run:**
```bash
chmod +x scripts/dispatch-phase1-full.sh
./scripts/dispatch-phase1-full.sh
```

---

## Session Management

### List Active Sessions

```bash
copilot session list --filter status=RUNNING
```

### Kill a Stuck Session

```bash
copilot session cancel {session_id}
```

### Dump Session Results

```bash
# Export session log to file
copilot session export {session_id} > /tmp/session-{session_id}.txt

# Share findings
cat /tmp/session-{session_id}.txt | grep -A 10 "error\|failed"
```

### Archive Completed Sessions

```bash
# Move old sessions out of active list
copilot session archive {session_id}

# View archived
copilot session list --archived
```

---

## Example: Complete Phase Task

### Setup Phase 1

```bash
cd /Users/kooshapari/temp-PRODVERCEL/485/kush/civ

# Wait for Phase 0 to be merged to main
# Then create worktree
git pull  # update main
git worktree add ../civ-wt-phase1-economy main
cd ../civ-wt-phase1-economy
git checkout -b feat/civ-phase1-economy
```

### Dispatch Test-First

```bash
copilot -p "
TASK: P1.1 - Market Test

Write failing test for FR-CIV-ECON-001-MARKET.

File: crates/economy/tests/fr_econ_market.rs

Test must verify:
1. Market::new() creates market
2. market.record_transaction(GoodID::Grain, 100.0)
3. market.update_prices()
4. market.get_price(GoodID::Grain) returns f32
5. Price changes based on supply/demand

Test MUST FAIL (Market doesn't exist yet).

Commit: 'test(economy): FR-CIV-ECON-001 failing test'
" \
  --yolo --model gpt-5-mini \
  --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ/../civ-wt-phase1-economy \
  --allow-tool 'write' \
  --allow-tool 'shell(cargo:*)' \
  --allow-tool 'shell(git:*)' \
  --deny-tool 'shell(git push)' \
  &
```

### Monitor

```bash
# In another terminal
cd ../civ-wt-phase1-economy

# Watch for test file to appear
watch -n 1 'ls -la crates/economy/tests/fr_econ_market.rs 2>/dev/null || echo "Waiting..."'

# When test appears
cargo test --package civ-economy test_market_tracks_grain_price 2>&1 | tail -20

# Check commits
git log --oneline -5
```

### Dispatch Implementation

Once test file exists and fails:

```bash
copilot -p "
TASK: P1.2 - Market Implementation

Implement FR-CIV-ECON-001-MARKET.

File: crates/economy/src/market.rs

Implement Market struct:
- new() -> Self
- record_transaction(good: GoodID, qty: f32)
- update_prices()
- get_price(good: GoodID) -> f32

Test file already exists: crates/economy/tests/fr_econ_market.rs
Make it pass.

Requirements:
- Prices inversely proportional to supply/demand
- Price range: [0.0, 1000.0]
- Must pass: cargo test --package civ-economy
- Must pass: cargo clippy -- -D warnings
- Export in src/lib.rs

Commit: 'feat(economy): FR-CIV-ECON-001 market'
" \
  --yolo --model gpt-5-mini \
  --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ/../civ-wt-phase1-economy \
  --allow-tool 'write' \
  --allow-tool 'shell(cargo:*)' \
  --allow-tool 'shell(git:*)' \
  --deny-tool 'shell(git push)' \
  &
```

### Verify

```bash
cd ../civ-wt-phase1-economy

# Wait for agent to finish
sleep 10

# Check test passes
cargo test --package civ-economy test_market_tracks_grain_price -v

# Check commits
git log --oneline -3
```

---

## Summary

**Key Points:**
- L3 agents work in isolated worktrees
- Use explicit prompt templates with constraints
- Batch dispatch for parallel work (limit to 4-8 concurrent)
- Monitor via copilot session logs + git logs
- Always use `--deny-tool 'shell(git push)'` (no remote pushes)
- Test-first: failing test → implementation
- Permissions: read, write, cargo, git (no push/reset/rm)

**Command Template:**
```bash
copilot -p "{TASK: ... REQUIREMENTS: ... ACCEPTANCE CRITERIA: ...}" \
  --yolo --model gpt-5-mini \
  --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ \
  --allow-tool 'write' \
  --allow-tool 'shell(cargo:*)' \
  --allow-tool 'shell(git:*)' \
  --deny-tool 'shell(git push)' \
  &
```



---

## Source: guides/GIT_WORKTREE_GUIDE.md

# Git Worktree Strategy for CivLab Parallel Development

**Purpose:** Enable parallel feature development with L3 copilot agents using isolated git worktrees
**Scope:** One worktree per sprint track (Phase), each with independent agent workers
**Isolation:** Changes in one worktree don't affect others until merge
**Merge Strategy:** PR-based review on main before integrating phase changes

---

## Why Worktrees?

Traditional branch-based development has issues with parallel L3 agents:
- Merge conflicts when multiple agents push to same branch
- Shared workspace state causes race conditions
- Each agent needs isolated `target/` directory, `Cargo.lock`

**Git worktrees solve this:**
- Each worktree has independent working directory
- Each worktree can have independent git index
- Agents never interfere with each other
- Parallel `cargo build` across worktrees (no lock contention)

---

## Worktree Architecture

```
civ/                              # Main worktree (never edit here during phases)
  .git/
  Cargo.toml
  crates/
  ...

civ-wt-phase0-foundation/         # Phase 0 worktree
  .git -> ../civ/.git/linked
  Cargo.toml
  crates/
  ...

civ-wt-phase1-economy/            # Phase 1 worktree
  .git -> ../civ/.git/linked
  Cargo.toml
  crates/
  ...

civ-wt-phase2-actors/             # Phase 2 worktree (can start before Phase 1 merges)
  .git -> ../civ/.git/linked
  Cargo.toml
  crates/
  ...
```

All worktrees share the same `.git/` directory but have independent working trees.

---

## Creating Worktrees

### Phase 0: Foundation

```bash
cd /Users/kooshapari/temp-PRODVERCEL/485/kush/civ

# Create worktree for Phase 0
git worktree add ../civ-wt-phase0-foundation main

# Enter worktree
cd ../civ-wt-phase0-foundation

# Verify
git log -1  # should show main's HEAD
git branch  # should show *main (detached-like state)
```

### Phase 1: Economy

```bash
cd /Users/kooshapari/temp-PRODVERCEL/485/kush/civ

# Wait for Phase 0 to be merged into main
# Then create Phase 1 worktree (based on updated main)
git worktree add ../civ-wt-phase1-economy main

cd ../civ-wt-phase1-economy
```

### Parallel Worktrees (e.g., Phase 3 during Phase 1-2)

```bash
cd /Users/kooshapari/temp-PRODVERCEL/485/kush/civ

# Can create before Phase 1 merges (Phase 3 is independent)
git worktree add ../civ-wt-phase3-client main

cd ../civ-wt-phase3-client
```

---

## Branch Naming Convention

Each worktree uses branch name following this pattern:

```
feat/civ-{PHASE}-{FEATURE}
```

Examples:
- `feat/civ-phase0-tick-loop`
- `feat/civ-phase1-market-economy`
- `feat/civ-phase2-institutions`
- `feat/civ-phase3-websocket`

### Setting Up Branch in Worktree

```bash
cd ../civ-wt-phase0-foundation

# Create feature branch (don't stay on main)
git checkout -b feat/civ-phase0-tick-loop

# Verify
git branch
# Output:
#   main
# * feat/civ-phase0-tick-loop
```

---

## Commit Message Convention

All commits MUST follow this format:

```
{type}({crate}): {FR-ID} {short description}

{optional detailed explanation}

Relates-to: {PLAN task ID if applicable}
Spec-ref: {CIV-#### spec section if applicable}
```

### Commit Types

| Type | Purpose | Example |
|------|---------|---------|
| `feat` | New feature | `feat(engine): FR-CIV-0001-TICK core tick loop` |
| `test` | Test (including failing tests) | `test(engine): FR-CIV-0001-TICK failing test` |
| `fix` | Bug fix | `fix(economy): FR-CIV-ECON-001 market price calc` |
| `refactor` | Code cleanup (no logic change) | `refactor(engine): FR-CIV-0001 simplify tick` |
| `docs` | Documentation | `docs: scenario YAML format spec` |
| `perf` | Performance improvement | `perf(economy): joule allocation O(n)` |

### Examples

```bash
# Test-first commit
git commit -m "test(economy): FR-CIV-ECON-001 failing test"

# Implementation commit
git commit -m "feat(economy): FR-CIV-ECON-001 market price tracking"

# Refactor commit
git commit -m "refactor(economy): FR-CIV-ECON-001 simplify price calculation"

# Multi-line with spec reference
git commit -m "feat(engine): FR-CIV-0001-TICK core tick loop

Implements deterministic turn increment per spec CIV-0001-core-simulation-loop.md.
Ensures all state changes are reproducible with same seed.

Spec-ref: CIV-0001-core-simulation-loop § 3.1
Relates-to: P0.2"
```

---

## Workflow: Single Phase Task

### Agent 1: Write Failing Test

```bash
cd ../civ-wt-phase0-foundation
git checkout feat/civ-phase0-tick-loop

# Dispatch L3 agent to write test
copilot -p "
Write failing test for FR-CIV-0001-TICK.
...
Commit: 'test(engine): FR-CIV-0001-TICK failing test'
" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ/.. &

# Or manually:
# Write test file crates/engine/tests/fr_core_tick_loop.rs
# git add tests/fr_core_tick_loop.rs
# git commit -m "test(engine): FR-CIV-0001-TICK failing test"
```

### Verify Test Fails

```bash
cd ../civ-wt-phase0-foundation
cargo test --package civ-engine test_simulation_tick_increments_turn

# Should output:
# test test_simulation_tick_increments_turn ... FAILED
```

### Agent 2: Implement Feature

```bash
cd ../civ-wt-phase0-foundation
git log -1  # verify test commit is there

# Dispatch L3 agent to implement
copilot -p "
Implement FR-CIV-0001-TICK.
Test already exists in crates/engine/tests/fr_core_tick_loop.rs
Make it pass.
...
Commit: 'feat(engine): FR-CIV-0001-TICK core tick loop'
" --yolo --model gpt-5-mini --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ/.. &
```

### Verify Test Passes

```bash
cd ../civ-wt-phase0-foundation
cargo test --package civ-engine test_simulation_tick_increments_turn

# Should output:
# test test_simulation_tick_increments_turn ... ok
```

### Check Commit History

```bash
cd ../civ-wt-phase0-foundation
git log --oneline feat/civ-phase0-tick-loop | head -5

# Output:
# 1a2b3c4 feat(engine): FR-CIV-0001-TICK core tick loop
# 5d6e7f8 test(engine): FR-CIV-0001-TICK failing test
# 9g0h1i2 main branch commit
```

---

## Workflow: Merge Phase Back to Main

Once a phase is complete (all tasks done, all tests pass):

### Step 1: Final Quality Gate in Worktree

```bash
cd ../civ-wt-phase0-foundation

# Run complete quality gate
cargo fmt --all
cargo test --all
cargo clippy --all -- -D warnings
task spec:validate

# All should pass
```

### Step 2: Create Pull Request

```bash
cd ../civ-wt-phase0-foundation

# Get list of commits since main
git log main..feat/civ-phase0-tick-loop --oneline

# Create PR description
cat > /tmp/pr-phase0.md << 'EOF'
## Phase 0: Foundation — Core Tick Loop

### Summary
Implements deterministic core simulation tick loop and determinism replay validation.

### Changes
- Core `Simulation::tick()` with turn increment
- RNG seeding contract for all randomness
- Deterministic replay test harness
- 100% test coverage for engine crate

### Tests
- `crates/engine/tests/fr_core_tick_loop.rs` — tick logic
- `crates/engine/tests/fr_determinism_replay.rs` — replay
- All passing: `cargo test --package civ-engine`

### Related Issues
Closes #[none yet]
Relates to PLAN.md Phase 0

### Checklist
- [x] All tests passing
- [x] clippy clean
- [x] 100% engine coverage
- [x] Determinism verified (100+ ticks)
EOF

# In GitHub UI: Create PR from feat/civ-phase0-tick-loop to main
# Paste PR description
# Request review
```

### Step 3: Merge via GitHub

```bash
# GitHub UI: Merge PR (use "Squash and merge" or "Rebase and merge")
# Delete branch on GitHub
```

### Step 4: Sync and Clean Up Locally

```bash
cd /Users/kooshapari/temp-PRODVERCEL/485/kush/civ

# Update main
git pull origin main

# Delete worktree
git worktree remove ../civ-wt-phase0-foundation

# Verify
git worktree list  # should not show phase0
```

---

## Conflict Resolution Policy

### Spec Files Go Through Main

If two agents modify `docs/specs/CIV-*.md`:

```
❌ WRONG: Merge in worktree, risk inconsistency
✅ RIGHT: Make spec changes in main ONLY
```

Process:
1. Any spec change goes as PR to main
2. Worktree agents pull updated main
3. Merge spec changes to all worktrees

### Implementation Files Can Merge in Parallel

If two agents modify different crates:

```
civ-wt-phase1-economy/
  changes to crates/economy/src/

civ-wt-phase2-actors/
  changes to crates/actors/src/

→ No conflict, both merge cleanly
```

### If Conflict Occurs

```bash
cd ../civ-wt-phase1-economy

# Pull latest main to sync
git fetch origin main
git rebase origin/main feat/civ-phase1-economy

# Resolve conflicts
# (Manual edit if needed)
git add crates/economy/src/conflicting_file.rs
git rebase --continue

# Force push to feature branch (safe in worktree)
git push origin feat/civ-phase1-economy -f
```

---

## Monitoring Worktrees

### List All Worktrees

```bash
cd /Users/kooshapari/temp-PRODVERCEL/485/kush/civ
git worktree list

# Output:
# /Users/kooshapari/temp-PRODVERCEL/485/kush/civ                 1a2b3c4 [main]
# /Users/kooshapari/temp-PRODVERCEL/485/kush/civ-wt-phase0-foundation 5d6e7f8 [feat/civ-phase0-tick-loop]
# /Users/kooshapari/temp-PRODVERCEL/485/kush/civ-wt-phase1-economy 9g0h1i2 [feat/civ-phase1-economy]
```

### Check Status of Worktree

```bash
cd ../civ-wt-phase1-economy

git status
# Should show clean or staged changes only

git log origin/main..HEAD --oneline
# Shows commits not yet on main
```

### Check Branch Status

```bash
cd ../civ-wt-phase1-economy

git branch -vv
# Shows tracking branch and local commits
```

---

## Best Practices

### 1. Keep Worktrees On Feature Branches

❌ **Wrong:**
```bash
cd ../civ-wt-phase1-economy
git checkout main  # Don't do this!
```

✅ **Right:**
```bash
cd ../civ-wt-phase1-economy
git checkout feat/civ-phase1-economy  # Stay on feature branch
```

**Why:** Changes to main in worktree won't be committed anywhere.

### 2. Sync main Periodically

```bash
cd ../civ-wt-phase1-economy

# Pull latest main (doesn't change working dir if on feature branch)
git fetch origin main

# Check if feature branch is behind
git log origin/main..HEAD  # commits ahead
git log HEAD..origin/main  # commits behind

# If behind, rebase
git rebase origin/main feat/civ-phase1-economy
```

### 3. Don't Manually Merge Between Worktrees

❌ **Wrong:**
```bash
cd ../civ-wt-phase1-economy
git merge ../civ-wt-phase2-actors  # Don't cherry-pick from other worktree
```

✅ **Right:**
```bash
# Let GitHub handle merges via PR
# Each phase merges independently to main
# Then next phase pulls updated main
```

### 4. Use Separate `target/` Directories

Each worktree has its own `target/` directory (Cargo handles this automatically). But for faster builds:

```bash
# Optionally share target directory (advanced)
cd ../civ-wt-phase1-economy
mkdir -p ~/.cargo/config.local.toml
cat > ~/.cargo/config.local.toml << 'EOF'
[build]
target-dir = "/tmp/civ-build"  # shared across worktrees
EOF
```

**Warning:** This can cause issues if two agents build simultaneously. Safer to keep separate targets.

### 5. Clean Up Before Merging

```bash
cd ../civ-wt-phase0-foundation

# Remove test artifacts
cargo clean

# Remove any uncommitted files
git clean -fd  # Remove untracked files
git clean -fdx # Also remove ignored files (be careful!)

# Verify clean state
git status
# On branch feat/civ-phase0-tick-loop
# nothing to commit, working tree clean
```

---

## Example: Three Parallel Phases

```bash
# Main repository
cd /Users/kooshapari/temp-PRODVERCEL/485/kush/civ

# Phase 0 worktree (sequential, blocks others)
git worktree add ../civ-wt-phase0-foundation main
cd ../civ-wt-phase0-foundation
git checkout -b feat/civ-phase0-tick-loop
# ... agents work in parallel
# ... merge to main

# Back on main
cd /Users/kooshapari/temp-PRODVERCEL/485/kush/civ
git pull

# Phase 1 worktree (depends on Phase 0)
git worktree add ../civ-wt-phase1-economy main
cd ../civ-wt-phase1-economy
git checkout -b feat/civ-phase1-economy
# ... 4-6 agents work on parallel tasks

# Phase 3 worktree (independent from Phase 1-2!)
cd /Users/kooshapari/temp-PRODVERCEL/485/kush/civ
git worktree add ../civ-wt-phase3-client main
cd ../civ-wt-phase3-client
git checkout -b feat/civ-phase3-client
# ... 4-6 agents work on parallel tasks

# Meanwhile, Phase 1 finishes and merges...
# Phase 3 doesn't care, it continues independently

# When Phase 1 merges to main:
cd /Users/kooshapari/temp-PRODVERCEL/485/kush/civ
git pull

# Phase 3 pulls updated main if needed
cd ../civ-wt-phase3-client
git fetch origin main
git rebase origin/main feat/civ-phase3-client
```

---

## Troubleshooting

### Worktree Lock File Error

```
fatal: '/Users/.../civ/.git/worktrees/civ-wt-phase1/.lock': Permission denied
```

**Solution:**
```bash
# Remove stale lock file
rm /Users/kooshapari/temp-PRODVERCEL/485/kush/civ/.git/worktrees/civ-wt-phase1/.lock

# Then prune
git worktree prune
```

### Can't Delete Worktree (Still in Use)

```bash
# Make sure you're not in the worktree directory
cd /Users/kooshapari/temp-PRODVERCEL/485/kush/civ  # NOT in the worktree!

# Then remove
git worktree remove ../civ-wt-phase1-economy
```

### Worktree Out of Sync

```bash
cd ../civ-wt-phase1-economy

# Fetch latest main
git fetch origin main

# Check what's different
git diff origin/main..HEAD -- crates/economy/

# If you want latest main plus your changes
git rebase origin/main HEAD

# Or hard reset to main (loses local changes)
git reset --hard origin/main
```

### Can't Commit from L3 Agent

```
error: object file is empty
error: object file .git/objects/XX/YYYYYY is empty
```

**Solution:**
```bash
cd ../civ-wt-phase1-economy

# Recover git objects
git fsck --full --lost-found
git reflog expire --expire-unreachable=all --all
git gc --prune=all

# If still broken, clone again
```

---

## Summary

**Key Points:**
- One worktree per phase/sprint
- Feature branch naming: `feat/civ-{PHASE}-{FEATURE}`
- Commit type + crate + FR-ID: `feat(economy): FR-CIV-ECON-001 market`
- Each agent works isolated (no shared state)
- Merge through GitHub PR only
- Clean up worktree before merging
- Phase 3+ can start before Phase 0-2 finish

**Workflow:**
```
Main
  ├─ P0 worktree → feat/civ-phase0-tick-loop → PR → merge
  │   (Update main)
  ├─ P1 worktree → feat/civ-phase1-economy → PR → merge
  │   (Parallel: P3 starts)
  └─ P3 worktree → feat/civ-phase3-client → PR → merge
```



---

## Source: guides/TEST_FIRST_GUIDE.md

# Test-First Engineering Guide for CivLab

**Purpose:** Establish and enforce test-first development (TDD) for all CivLab implementation
**Scope:** All crates (engine, economy, actors, social, policy, metrics, server)
**Mandate:** Every functional requirement (FR) gets a failing test BEFORE implementation
**Coverage Target:** 100% for engine + economy crates, 80%+ for others

---

## Core TDD Cycle

For every feature requirement (FR):

```
1. Write Failing Test
   └─> File: crates/{crate}/tests/fr_{id}.rs
   └─> Test must FAIL (code doesn't exist yet)
   └─> Commit: 'test({crate}): {FR-ID} failing test'

2. Implement Feature
   └─> File: crates/{crate}/src/{module}.rs
   └─> Implement until test passes
   └─> Commit: 'feat({crate}): {FR-ID} {description}'

3. Refactor (Optional)
   └─> Improve code quality, reduce duplication
   └─> All tests still pass
   └─> Commit: 'refactor({crate}): {FR-ID} {improvement}'
```

**Each FR = 1 failing test → 1 implementation → optionally 1 refactor**

---

## Test File Naming & Organization

### Directory Structure

```
crates/{crate}/
  src/
    lib.rs          # Module exports
    module.rs       # Implementation
  tests/
    fr_{id}.rs      # Test file for FR-ID
    common/mod.rs   # Shared test fixtures (if needed)
```

### Naming Convention

| Test Type | File | Example |
|-----------|------|---------|
| Single FR test | `tests/fr_{fr_id}.rs` | `tests/fr_core_tick_loop.rs` |
| Multi-requirement test | `tests/fr_{feature}.rs` | `tests/fr_market_operations.rs` |
| Property test | `tests/fr_{feature}_properties.rs` | `tests/fr_economy_properties.rs` |
| Integration test | `tests/integration_{crates}.rs` | `tests/integration_engine_economy.rs` |
| Determinism/replay | `tests/fr_{feature}_replay.rs` | `tests/fr_determinism_replay.rs` |

### Module-Level Tests

For small, unit-level tests, keep them in the same file as the implementation:

```rust
// crates/engine/src/simulation.rs

pub fn tick_turn_counter(turn: &mut u64) -> Result<()> {
    *turn += 1;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_increments() {
        let mut turn = 0;
        tick_turn_counter(&mut turn).unwrap();
        assert_eq!(turn, 1);
    }
}
```

### Integration Tests (Preferred for FR tests)

Keep tests in `tests/` directory to test module boundaries:

```rust
// crates/engine/tests/fr_core_tick_loop.rs

#[test]
fn test_simulation_tick_increments_turn() {
    let mut sim = Simulation::new();
    let initial_turn = sim.current_turn();
    sim.tick().expect("tick should succeed");
    assert_eq!(sim.current_turn(), initial_turn + 1);
}
```

---

## Test Types Required

Every FR must have AT LEAST ONE test from the following categories. Most FRs need multiple:

### 1. Unit Tests (Pure Functions)

Test a single function with simple inputs/outputs.

**When to use:** Pure logic, no state mutation, no external dependencies

**Example:**

```rust
// crates/economy/tests/fr_econ_market.rs

#[test]
fn test_market_price_increase() {
    let mut market = Market::new();
    market.record_transaction(GoodID::Grain, 100.0); // high supply
    market.record_transaction(GoodID::Grain, 10.0);  // low demand

    let old_price = market.get_price(GoodID::Grain);
    market.update_prices();
    let new_price = market.get_price(GoodID::Grain);

    assert!(new_price < old_price); // low demand → price down
}
```

### 2. Integration Tests (Crate Boundary)

Test interaction between modules within a crate or across crate boundaries.

**When to use:** Module interactions, public API, setup/teardown

**Example:**

```rust
// crates/engine/tests/integration_engine_economy.rs

#[test]
fn test_simulation_integrates_economy() {
    let mut sim = Simulation::new();

    // Set up economy state
    sim.economy.add_goods(vec![Grain, Wood]);

    // Tick should call economy::tick()
    sim.tick().expect("tick succeeds");

    // Verify economy was ticked
    assert_eq!(sim.turn(), 1);
}
```

### 3. Scenario Tests (Full Sim Run)

Load a scenario YAML, run N ticks, assert metric snapshots.

**When to use:** Full system behavior, emergent properties, reproducibility

**Example:**

```rust
// crates/engine/tests/fr_scenario_loader.rs

#[test]
fn test_scenario_runs_100_ticks() {
    let scenario = Scenario::load_yaml("docs/scenarios/test_basic.yaml")
        .expect("scenario loads");
    let mut sim = scenario.into_simulation();

    for _ in 0..100 {
        sim.tick().expect("tick succeeds");
    }

    let snapshot = sim.snapshot();
    assert!(snapshot.population > 0); // population survived
    assert_eq!(snapshot.tick, 100);
}
```

### 4. Replay Tests (Determinism)

Record simulation state, replay with same seed, verify byte-for-byte equality.

**When to use:** Determinism verification, replay validation, save/restore

**Example:**

```rust
// crates/engine/tests/fr_determinism_replay.rs

#[test]
fn test_deterministic_replay_100_ticks() {
    let seed = 42u64;

    // First run
    let mut sim1 = Simulation::with_seed(seed);
    let states1 = record_states(&mut sim1, 100);

    // Second run (same seed)
    let mut sim2 = Simulation::with_seed(seed);
    let states2 = record_states(&mut sim2, 100);

    // Verify state equality
    assert_eq!(states1, states2, "replays must be deterministic");
}

fn record_states(sim: &mut Simulation, ticks: u64) -> Vec<SimulationState> {
    let mut states = Vec::new();
    for _ in 0..ticks {
        states.push(sim.state().clone());
        sim.tick().expect("tick");
    }
    states
}
```

### 5. Property-Based Tests (Invariants)

Use `proptest` to verify invariants hold across random input ranges.

**When to use:** Algorithms, resource allocation, conservation laws

**Example:**

```rust
// crates/economy/tests/fr_econ_properties.rs

use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_joule_allocation_never_exceeds_budget(
        available_joules in 1000u64..1_000_000u64,
        actor_count in 1..100usize,
    ) {
        let mut allocator = JouleAllocator::new();
        let allocation = allocator.allocate(available_joules, actor_count);

        let total: u64 = allocation.iter().sum();
        prop_assert!(total <= available_joules,
            "allocation {} exceeds budget {}", total, available_joules);
    }

    #[test]
    fn prop_market_prices_stay_bounded(
        transactions in prop::collection::vec((any::<u32>(), any::<f32>()), 1..1000),
    ) {
        let mut market = Market::new();
        for (good_id, qty) in transactions {
            market.record_transaction(GoodID(good_id as u32), qty.abs());
        }
        market.update_prices();

        for (_, price) in market.prices() {
            prop_assert!(price >= 0.0 && price <= 1000.0,
                "price {} out of bounds", price);
        }
    }
}
```

### 6. Snapshot/Regression Tests (State Capture)

Capture full system state, commit as golden file, verify future runs match.

**When to use:** Complex emergent behavior, historical validation

**Example:**

```rust
// crates/engine/tests/fr_state_snapshot.rs

#[test]
fn test_state_matches_golden_snapshot() {
    let mut sim = Scenario::load_yaml("docs/scenarios/test_small.yaml")
        .unwrap()
        .into_simulation();

    for _ in 0..50 {
        sim.tick().unwrap();
    }

    let snapshot = sim.snapshot();
    insta::assert_debug_snapshot!(snapshot);
}
```

(Uses `insta` crate for golden file management)

---

## Cargo Test Organization

### Run All Tests

```bash
cargo test --all
```

### Run Tests by Crate

```bash
cargo test --package civ-engine
cargo test --package civ-economy
cargo test --package civ-actors
```

### Run Tests by Category

```bash
# Unit tests only (in-module)
cargo test --lib

# Integration tests only
cargo test --test '*'

# Single test file
cargo test --test fr_core_tick_loop

# Single test function
cargo test --test fr_core_tick_loop test_simulation_tick_increments_turn

# Determinism tests (single-threaded for consistency)
cargo test -- --test-threads=1
```

### Run With Logging

```bash
RUST_LOG=debug cargo test -- --nocapture
```

### Test Coverage

```bash
# Using tarpaulin
cargo tarpaulin --out Html --exclude-files tests/ --timeout 300

# Using llvm-cov
cargo llvm-cov --html
```

---

## Writing Your First Failing Test

Step-by-step example for `FR-CIV-ECON-001-MARKET`:

### Step 1: Create Test File

```bash
touch crates/economy/tests/fr_econ_market.rs
```

### Step 2: Write Test That MUST FAIL

```rust
// crates/economy/tests/fr_econ_market.rs

use civ_economy::market::{Market, GoodID, Price};

#[test]
fn test_market_tracks_grain_price() {
    let mut market = Market::new();

    // Record transactions
    market.record_transaction(GoodID::Grain, 100.0); // supply
    market.record_transaction(GoodID::Grain, 10.0);  // demand

    // Update prices based on supply/demand
    market.update_prices();

    // Verify price decreased (low demand)
    let price = market.get_price(GoodID::Grain);
    assert!(price < 50.0, "grain price should decrease with low demand");
}
```

### Step 3: Verify Test Fails

```bash
cd crates/economy
cargo test test_market_tracks_grain_price

# Output should be:
# error[E0433]: cannot find `Market` in this scope
#    --> tests/fr_econ_market.rs:3:29
```

### Step 4: Commit Failing Test

```bash
git add tests/fr_econ_market.rs
git commit -m "test(economy): FR-CIV-ECON-001 failing test"
```

### Step 5: Implement Until Test Passes

```rust
// crates/economy/src/market.rs

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GoodID {
    Grain,
    Wood,
    // ...
}

pub type Price = f32;

pub struct Market {
    prices: HashMap<GoodID, Price>,
    supply: HashMap<GoodID, f32>,
    demand: HashMap<GoodID, f32>,
}

impl Market {
    pub fn new() -> Self {
        Self {
            prices: HashMap::new(),
            supply: HashMap::new(),
            demand: HashMap::new(),
        }
    }

    pub fn record_transaction(&mut self, good: GoodID, quantity: f32) {
        // Simple heuristic: positive = supply, negative = demand
        if quantity > 0.0 {
            *self.supply.entry(good).or_insert(0.0) += quantity;
        } else {
            *self.demand.entry(good).or_insert(0.0) += quantity.abs();
        }
    }

    pub fn update_prices(&mut self) {
        for (good, _) in self.prices.iter_mut() {
            let supply = self.supply.get(good).copied().unwrap_or(0.0);
            let demand = self.demand.get(good).copied().unwrap_or(1.0); // avoid div by 0

            // Price inversely proportional to supply/demand ratio
            let ratio = supply / (demand + 0.1);
            let new_price = 50.0 / (1.0 + ratio);

            self.prices.insert(*good, new_price);
        }
    }

    pub fn get_price(&self, good: GoodID) -> Price {
        self.prices.get(&good).copied().unwrap_or(50.0)
    }
}
```

### Step 6: Verify Test Passes

```bash
cargo test test_market_tracks_grain_price

# Output should be:
# test test_market_tracks_grain_price ... ok
```

### Step 7: Commit Implementation

```bash
git add crates/economy/src/market.rs
git commit -m "feat(economy): FR-CIV-ECON-001 market implementation"
```

---

## Copilot L3 Agent Test-First Pattern

For delegating to L3 copilot agents, use this prompt template:

### Phase 1: Write Failing Test

```bash
copilot -p "
Implement FR-CIV-ECON-001: Market price tracking.

Step 1 (THIS TASK): Write a failing test.

Requirements:
- Create file: crates/economy/tests/fr_econ_market.rs
- Test name: test_market_tracks_grain_price
- Test must verify:
  * Market::new() creates empty market
  * market.record_transaction(GoodID::Grain, 100.0) records supply
  * market.record_transaction(GoodID::Grain, 10.0) records demand
  * market.update_prices() calculates prices
  * market.get_price(GoodID::Grain) returns price < 50.0 (low demand)
- Test MUST FAIL right now (Market doesn't exist)
- Do NOT implement Market yet
- Commit message: 'test(economy): FR-CIV-ECON-001 failing test'

Only write the test file. Do not implement the feature.
" \
  --yolo --model gpt-5-mini \
  --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &
```

### Phase 2: Implement Feature

```bash
copilot -p "
Implement FR-CIV-ECON-001: Market price tracking.

Step 2 (THIS TASK): Implement the feature.

Requirements:
- File: crates/economy/src/market.rs
- Implement Market struct with:
  * new() -> Self
  * record_transaction(good: GoodID, quantity: f32)
  * update_prices()
  * get_price(good: GoodID) -> f32
- Invariant: price is always in range [0.0, 1000.0]
- Test must pass: cargo test --package civ-economy test_market_tracks_grain_price
- Must pass clippy: cargo clippy --package civ-economy -- -D warnings
- Commit message: 'feat(economy): FR-CIV-ECON-001 market'

The test already exists in crates/economy/tests/fr_econ_market.rs
Make it pass.
" \
  --yolo --model gpt-5-mini \
  --add-dir /Users/kooshapari/temp-PRODVERCEL/485/kush/civ &
```

---

## Coverage Requirements & Verification

### Coverage Targets by Crate

| Crate | Target | Measurement |
|-------|--------|-------------|
| engine | 100% | Line + branch coverage |
| economy | 100% | Line + branch coverage |
| actors | 80% | Line coverage |
| social | 80% | Line coverage |
| policy | 80% | Line coverage |
| metrics | 80% | Line coverage |
| server | 70% | Line coverage (external deps harder to test) |
| io | 70% | Line coverage (I/O harder to test) |

### Measure Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin \
  --out Html \
  --output-dir coverage/ \
  --exclude-files tests/ \
  --timeout 300

# View report
open coverage/index.html
```

### Identify Coverage Gaps

```bash
# List uncovered lines
cargo tarpaulin \
  --out Stdout \
  --exclude-files tests/ \
  | grep "MISSED"
```

### Add Tests for Gaps

For each uncovered line:

```rust
// crates/engine/tests/fr_coverage_gaps.rs

#[test]
fn test_error_path_invalid_tick() {
    // Test the error case that wasn't covered
    let sim = Simulation::new();
    let result = sim.invalid_operation();
    assert!(result.is_err());
}
```

---

## CI/CD Integration

### Pre-Commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

set -e

cargo fmt --all
cargo clippy --all -- -D warnings
cargo test --all

# Must pass traceability
task spec:validate
```

### GitHub Actions Example

```yaml
name: tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all --verbose
      - run: cargo clippy --all -- -D warnings
      - run: cargo tarpaulin --out Xml --exclude-files tests/
      - uses: codecov/codecov-action@v3
```

---

## Anti-Patterns

### ❌ DO NOT

1. **Write tests after implementation**
   - Bad: Implement feature, then add tests (false confidence)
   - Good: Write failing test first, then implement

2. **Skip unit tests because you have integration tests**
   - Bad: Only scenario tests
   - Good: Unit + integration + scenario tests

3. **Ignore coverage gaps**
   - Bad: "That error path will never happen"
   - Good: Test all branches, even error paths

4. **Use `#[ignore]` for broken tests**
   - Bad: Mark test as ignored
   - Good: Fix the test or remove it

5. **Comment out assertions to "fix" failures**
   - Bad: `// assert!(condition);` // TODO: fix this
   - Good: Fix the code to make assertion pass

6. **Test implementation details instead of behavior**
   - Bad: `assert_eq!(sim.actors.len(), 10);` // checking internal vec
   - Good: `assert!(sim.snapshot().population > 0);` // checking observable behavior

7. **Write non-deterministic tests**
   - Bad: `assert!(rng.gen::<f32>() < 0.5);` // flaky
   - Good: Seed RNG, verify behavior deterministically

---

## Example: Complete FR Implementation with Tests

Full example from test to code:

### Test File: `crates/economy/tests/fr_econ_joule.rs`

```rust
use civ_economy::joule::JouleAllocator;

#[test]
fn test_joule_allocation_respects_budget() {
    let available = 1000.0;
    let actor_count = 10;

    let allocator = JouleAllocator::new();
    let allocation = allocator.allocate(available, actor_count);

    let total: f32 = allocation.iter().sum();
    assert!(total <= available,
        "allocation {} exceeded budget {}", total, available);
}

#[test]
fn test_joule_fair_distribution() {
    let available = 1000.0;
    let actor_count = 10;

    let allocator = JouleAllocator::new();
    let allocation = allocator.allocate(available, actor_count);

    let avg = available / actor_count as f32;
    let tolerance = avg * 0.1; // 10% variance

    for joule in allocation.iter() {
        assert!((joule - avg).abs() < tolerance,
            "joule {} not within 10% of avg {}", joule, avg);
    }
}
```

### Implementation: `crates/economy/src/joule.rs`

```rust
pub struct JouleAllocator;

impl JouleAllocator {
    pub fn new() -> Self {
        Self
    }

    pub fn allocate(&self, available: f32, actor_count: usize) -> Vec<f32> {
        if actor_count == 0 {
            return Vec::new();
        }

        let per_actor = available / actor_count as f32;
        vec![per_actor; actor_count]
    }
}
```

### Export: `crates/economy/src/lib.rs`

```rust
pub mod joule;

pub use joule::JouleAllocator;
```

### Run Tests

```bash
cargo test --package civ-economy test_joule

# test test_joule_allocation_respects_budget ... ok
# test test_joule_fair_distribution ... ok
```

---

## Summary

**The Rule:**
- Every FR gets a failing test BEFORE code
- Every test file in `crates/{crate}/tests/fr_{id}.rs`
- Multiple test types per FR (unit, integration, scenario, property-based)
- 100% coverage on engine, 80%+ everywhere else
- Commit test first, then implement

**Copilot L3 Workflow:**
1. Dispatch agent to write failing test
2. Verify test fails
3. Dispatch agent to implement
4. Verify test passes



---

## Source: guides/anti-patterns.md

# Anti-Pattern Detection Hooks

Six hooks that detect and prevent common code anti-patterns. Each runs on `PreToolUse:Write/Edit` events and provides actionable fix suggestions.

## Hook Summary

| Hook | Pattern Detected | Severity | Languages |
|------|-----------------|----------|-----------|
| suppress-custom-retry.sh | Custom retry loops when tenacity available | WARNING | Python |
| suppress-v2-files.sh | `_v2`, `_new`, `_old` file naming | ERROR | All |
| suppress-hardcoded-strings.sh | Hardcoded provider/model/URL strings | WARNING | Python, TS, Go |
| suppress-print-statements.sh | print()/console.log when structured logger available | WARNING | Python, TS, Go |
| suppress-isolated-classes.sh | God classes (>15 methods or >300 lines) | WARNING | Python, TS |
| suppress-direct-http.sh | Direct HTTP calls without client abstraction | WARNING | Python, TS, Go |

## Hook Details

### suppress-custom-retry.sh

**What it detects**: Hand-rolled retry/backoff loops in Python when `tenacity` is declared in project dependencies.

**Patterns caught**:
- `for attempt in range(N)` with `sleep` + `try/except`
- `while True` retry loops with attempt counters
- Manual exponential backoff (`2 ** attempt`)

**Fix**:
```python
# Before (anti-pattern)
for attempt in range(5):
    try:
        result = httpx.get(url)
        break
    except Exception:
        time.sleep(2 ** attempt)

# After (correct)
from tenacity import retry, stop_after_attempt, wait_exponential

@retry(stop=stop_after_attempt(5), wait=wait_exponential())
def fetch(url: str) -> httpx.Response:
    return httpx.get(url, timeout=10)
```

---

### suppress-v2-files.sh

**What it detects**: Files with `_v2`, `_new`, `_old`, `_backup`, `_copy`, `_temp` suffixes.

**Why blocked (ERROR)**: v2 files lead to:
- Import confusion (which version to use?)
- Stale code copies that diverge
- No clear migration path

**Fix options**:
1. Modify the original file directly
2. Use feature flags for behavioral changes
3. Use interface versioning (APIv2 endpoint, not handler_v2.py)
4. If migrating, rename the original and update all imports in one commit

---

### suppress-hardcoded-strings.sh

**What it detects**: Hardcoded provider names, model identifiers, and API URLs in source code.

**Patterns caught**:
- LLM model names: `"gpt-4"`, `"claude-3"`, `"gemini-1.5"`
- API URLs: `"https://api.openai.com/..."`
- Provider identifiers: `"aws"`, `"openai"`, `"anthropic"` as string literals

**Excludes**: Test files, imports, comments.

**Fix**:
```python
# Before
response = client.chat("gpt-4", messages)

# After
from config import settings
response = client.chat(settings.default_model, messages)
```

---

### suppress-print-statements.sh

**What it detects**: Unstructured logging (print, console.log, fmt.Println) when a structured logging library is in project dependencies.

**Dependency triggers**:
- Python: `structlog` in deps -> blocks `print()` and `logging.getLogger()`
- TypeScript: `pino`/`winston` in deps -> blocks `console.log()`
- Go: `zerolog`/`zap` in deps -> blocks `fmt.Println()`

**Fix**:
```python
# Before
print(f"Processing user {user_id}")

# After
import structlog
log = structlog.get_logger()
log.info("processing_user", user_id=user_id)
```

---

### suppress-isolated-classes.sh

**What it detects**: Classes that are too large ("God classes").

**Thresholds**:
- More than 15 methods
- More than 300 lines

**Fix**:
1. **SRP decomposition**: Extract cohesive method groups into separate classes
2. **Composition**: Break into composed components instead of one monolith
3. **Strategy pattern**: Extract behavioral variations into strategy classes
4. **DTO extraction**: Move data-only methods into separate dataclasses

---

### suppress-direct-http.sh

**What it detects**: Direct HTTP client calls (requests.get, fetch(), http.Get) in business logic.

**Why**: Direct calls scatter URL construction, authentication, retry logic, timeout handling, and error mapping across the codebase.

**Excludes**: Files in `clients/`, `adapters/`, `infrastructure/`, `transport/` directories. Files named `*client*`, `*http*`, `*fetcher*`.

**Fix**:
```python
# Before (scattered in business logic)
response = requests.get(f"{BASE_URL}/users/{user_id}", headers=auth_headers)

# After (centralized client)
class UserClient:
    def __init__(self, base_url: str, auth: Auth):
        self.client = httpx.AsyncClient(base_url=base_url, auth=auth)

    async def get_user(self, user_id: str) -> User:
        response = await self.client.get(f"/users/{user_id}")
        response.raise_for_status()
        return User.model_validate(response.json())
```

---

## CIV-Specific Anti-Patterns

### suppress-floating-point-simulation.md

**What it detects**: Use of floating-point arithmetic for simulation state.

**Why blocked**: Floating-point rounding errors accumulate across simulation ticks, making determinism impossible. CIV requires bit-exact reproducibility.

**Fix**:
```rust
// Before (ANTI-PATTERN)
pub struct Resource {
    amount: f64,
}

// After (CORRECT)
pub struct Resource {
    amount: i64,  // Fixed-point: represents smallest unit
}

// Helper functions for scaling
const RESOURCE_SCALE: i64 = 1_000_000; // 6 decimal places
fn to_scaled(f: f64) -> i64 { (f * RESOURCE_SCALE as f64) as i64 }
fn from_scaled(i: i64) -> f64 { i as f64 / RESOURCE_SCALE as f64 }
```

---

### suppress-hashmap-usage.sh

**What it detects**: Use of HashMap when deterministic iteration order is required.

**Why**: HashMap iteration order is undefined. Any code that traverses the map for simulation events must use BTreeMap for consistent ordering across runs.

**Fix**:
```rust
// Before (ANTI-PATTERN)
use std::collections::HashMap;
let mut events: HashMap<String, Event> = HashMap::new();

// After (CORRECT)
use std::collections::BTreeMap;
let mut events: BTreeMap<String, Event> = BTreeMap::new();
```

---

### suppress-allocation-regime-fallbacks.sh

**What it detects**: Code that falls back to defaults when an allocation regime is not configured.

**Why blocked**: Fallbacks hide configuration errors. CIV simulation must fail loudly if regime is missing.

**Fix**:
```rust
// Before (ANTI-PATTERN)
let regime = allocation_config
    .get("resource_regime")
    .unwrap_or_else(|| default_regime());  // Fallback!

// After (CORRECT)
let regime = allocation_config
    .get("resource_regime")
    .expect("FATAL: resource_regime not configured");
```

---

### suppress-sim-state-reads-during-tick.sh

**What it detects**: Reading live simulation state during a tick instead of using the snapshotted state from tick start.

**Why**: Live reads can cause race conditions and non-deterministic behavior. Each tick must operate on an immutable snapshot.

**Fix**:
```rust
// Before (ANTI-PATTERN)
fn tick(sim: &mut Simulation) {
    for citizen in &mut sim.citizens {  // LIVE mutations!
        citizen.hunger = sim.get_current_food() - citizen.eaten;
    }
}

// After (CORRECT)
fn tick(sim_state: &SimulationState, changes: &mut Changes) {
    // sim_state is immutable snapshot from tick start
    for citizen_id in &sim_state.citizens {
        let food = sim_state.get_food(citizen_id);
        changes.update_hunger(citizen_id, food);
    }
}
```

---

## Integration

### Claude Code Hooks (settings.json)

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Write|Edit",
        "hooks": [
          "hooks/suppress-custom-retry.sh $FILE_PATH",
          "hooks/suppress-v2-files.sh $FILE_PATH",
          "hooks/suppress-hardcoded-strings.sh $FILE_PATH",
          "hooks/suppress-print-statements.sh $FILE_PATH",
          "hooks/suppress-isolated-classes.sh $FILE_PATH",
          "hooks/suppress-direct-http.sh $FILE_PATH"
        ]
      }
    ]
  }
}
```

### Pre-commit (local hooks)

```yaml
- repo: local
  hooks:
    - id: no-v2-files
      name: Block v2/new/old file creation
      entry: hooks/suppress-v2-files.sh
      language: script
      stages: [pre-commit]
```

## Testing Hooks

Each hook can be tested standalone:

```bash
# Test v2 file detection (should print error and exit 1)
echo "test" > /tmp/handler_v2.py
./hooks/suppress-v2-files.sh /tmp/handler_v2.py
echo "Exit code: $?"

# Test custom retry detection (should print warning)
cat > /tmp/retry_test.py << 'EOF'
for attempt in range(5):
    try:
        result = httpx.get(url)
        break
    except Exception:
        time.sleep(2 ** attempt)
EOF
./hooks/suppress-custom-retry.sh /tmp/retry_test.py
```


---
