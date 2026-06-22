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

