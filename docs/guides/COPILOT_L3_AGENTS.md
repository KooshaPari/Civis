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

