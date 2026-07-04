# Contributing to Civis

## Branch naming

| Prefix | Use for |
|--------|---------|
| `feat/` | New features |
| `fix/` | Bug fixes |
| `test/` | Test additions or coverage |
| `perf/` | Performance work |
| `chore/` | Tooling, deps, CI, config |
| `docs/` | Documentation only |

## Commit format

```
type(scope): short description (FR-CIV-XXX-NNN)
```

Every commit implementing a functional requirement must reference its FR ID. Example:

```
feat(client): tech tree panel — T key, TECH_STUBS catalogue (FR-CIV-CLIENT-007)
```

## PR requirements

1. **Add the `local-first-ci` label** — required for the governance gate to treat third-party checks (SonarCloud, Kilo) as optional. Create it once with:
   ```
   gh label create "local-first-ci" --color "#0E8A16" \
     --description "Third-party CI optional; local-first attestation governs"
   ```

2. **Governance gate must pass** — `pr-governance-gate` is the merge blocker. Never merge while governance is red.

3. **Every PR must include tests** — new `pub fn` items require at least one test. Coverage target: ≥ 75% of `pub fn` items per crate.

4. **No stale `.ci/quality-manifest.json`** — if your branch has one with an old SHA, delete it:
   ```
   rm .ci/quality-manifest.json
   git commit -m "chore(ci): remove stale quality manifest"
   ```
   The verify script exits 0 when the file is absent.

## Local dev

### Cargo target directory

Always build to the E: drive — C: fills fast with Rust build artifacts:

```toml
# .cargo/config.toml (already set)
[build]
target-dir = "E:/civis-target"
```

Never override this to a C: path.

### Never stash

`git stash` loses context and creates invisible state. Instead:

- Commit to a WIP branch: `git commit -m "wip: in-progress changes"`
- Or keep working dirty on the current branch

### Worktrees

Parallel feature work goes in dedicated worktrees, not dirty checkouts of the same tree:

```
git worktree add .worktrees/my-topic feat/my-topic
```

## Test coverage

- Target: ≥ 75% of `pub fn` items covered per crate
- Run: `cargo test --workspace`
- Measure: `cargo llvm-cov --workspace` (ensure `CARGO_TARGET_DIR=E:/civis-target`)
- Every emergence-coupling PR must have a corresponding `test/nN-*` coverage PR

## CI checks

| Check | Required | Notes |
|-------|----------|-------|
| `pr-governance-gate` | Yes | Merge blocker |
| `quality-manifest (cloud verify)` | Yes | Delete manifest if stale |
| `Rust Module Graph` | No | Bypassed via `local-first-ci` label |
| `SonarCloud` | No | Bypassed via `local-first-ci` label |
| `Kilo Code Review` | No | Bypassed via `local-first-ci` label |