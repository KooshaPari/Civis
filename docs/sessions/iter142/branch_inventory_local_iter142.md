# Local Branch Inventory (Iter-142)

Date: 2026-05-18

## (a) Local Branches Summary

**Total**: 3 local branches
- `fix/handle-connect-iter142` (HEAD: ced0dccf)
- `main` (HEAD: 17f88a14)
- `safety/iter140-snapshot-2026-05-18` (HEAD: f699154e) **← CURRENT**

## (b) Commit Status vs origin/main

| Branch | Behind | Ahead | Status |
|--------|--------|-------|--------|
| `fix/handle-connect-iter142` | 51 | 2 | 2 new commits (HandleConnect feature + safety) |
| `main` | 51 | 1 | 1 new commit (changelog only, diverged) |
| `safety/iter140-snapshot-2026-05-18` | 51 | 2 | 2 commits (safety snapshot + HandleConnect) |

**Key Insight**: All branches are 51 commits BEHIND origin/main, indicating origin/main has been force-pushed or the remote has accumulated stale commits. This is a red flag for consolidation.

## (c) Top 5 Commits by Branch

### `fix/handle-connect-iter142` (2 ahead)
1. `ced0dccf` fix(bridge): implement HandleConnect for GameClient handshake
2. `17f88a14` chore(changelog): Iter-108 wave summary—13 Tier 1 Roslyn analyzers, ...
3. `f222cd32` chore(nuget): Bump Bridge packages to v0.24.0 for NuGet publishing
4. `14615443` Merge branch 'main' of https://github.com/KooshaPari/Dino
5. `fef46be0` feat(docs): Expand journey collection from 4 to 21+ comprehensive demonstrations

### `main` (1 ahead, diverged)
1. `17f88a14` chore(changelog): Iter-108 wave summary—13 Tier 1 Roslyn analyzers, ...
2. `f222cd32` chore(nuget): Bump Bridge packages to v0.24.0 for NuGet publishing
3. `14615443` Merge branch 'main' of https://github.com/KooshaPari/Dino
4. `fef46be0` feat(docs): Expand journey collection from 4 to 21+ comprehensive demonstrations
5. `95e54139` chore(workflows): dedupe release-drafter via phenoShared reusable (#153)

### `safety/iter140-snapshot-2026-05-18` (2 ahead, CURRENT)
1. `f699154e` chore: safety snapshot of iter-140 session work
2. `17f88a14` chore(changelog): Iter-108 wave summary—13 Tier 1 Roslyn analyzers, ...
3. `f222cd32` chore(nuget): Bump Bridge packages to v0.24.0 for NuGet publishing
4. `14615443` Merge branch 'main' of https://github.com/KooshaPari/Dino
5. `fef46be0` feat(docs): Expand journey collection from 4 to 21+ comprehensive demonstrations

## (d) Remote vs Local Status

**Pushed to Remote**:
- `main` (origin/main exists)
- `safety/iter140-snapshot-2026-05-18` (origin/safety/iter140-snapshot-2026-05-18 exists)

**Local-Only**:
- `fix/handle-connect-iter142` (NO corresponding origin branch)

**Note**: `fix/deps-npm-2026-04-27` exists on remote but NOT locally.

## (e) Primary Work Container Identification

**Winner**: `fix/handle-connect-iter142`

**Rationale**:
- Contains the feature commit `ced0dccf` (HandleConnect implementation) not on any other branch
- Represents the newest, most forward-looking work
- 2 commits ahead of main (one feature, one changelog alignment)
- Not yet pushed — consolidation PR will be the first push

**Consolidation Recommendation**:
- **Base**: `origin/main` (pull latest 51 commits, resolve divergence)
- **Feature Branch**: `fix/handle-connect-iter142`
- **Target SHA**: `ced0dccf` (top commit, HandleConnect feature)
- **Approach**: Rebase onto origin/main, then open PR for review before merge

**Cleanup After Merge**:
- Delete `main` (diverged copy, has no unique work)
- Delete `safety/iter140-snapshot-2026-05-18` (safety snapshot, redundant after main merge)
- Keep `fix/handle-connect-iter142` until PR merged, then delete

**Current Divergence Issue**: All local branches are 51 commits behind origin/main, suggesting either:
1. Remote was force-pushed (e.g., rebase of main)
2. Local fetch is stale (run `git fetch --all`)

**Recommendation**: Run `git fetch --all` to refresh, then re-check the "behind" count to confirm consolidation approach.
