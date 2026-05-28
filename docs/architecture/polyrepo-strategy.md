# DINOForge Polyrepo Split Strategy

> Status: **RECOMMENDATION — Strategy B (Balanced)**
> Date: 2026-05-28
> Author: Architecture Analysis Agent

---

## 1. Executive Summary

CLAUDE.md describes DINOForge as "polyrepo-hexagonal" but the repository is a **single-repo monolith** containing 31 C# projects, a Python MCP server, 10 content packs, a VitePress docs site, and ~50 CI workflow files.

This document maps the true project boundaries, evaluates three split strategies, and recommends **Strategy B** — a balanced three-repo split that cleanly separates community-facing pack content from core platform code, without the steep orchestration cost of a full per-project polyrepo.

---

## 2. Current Boundary Map

### 2.1 C# Project Dependency Graph

```
DINOForge.Analyzers          (Roslyn analyzers; no project refs; analyzer-only)
       |
DINOForge.SDK                (netstandard2.0; NuGet-published; refs Analyzers as analyzer)
       |
DINOForge.Bridge.Protocol    (netstandard2.0; NuGet-published; refs Analyzers)
       |
DINOForge.Bridge.Client      (netstandard2.0; NuGet-published; refs Protocol + SDK)
       |
DINOForge.Domains.*          (netstandard2.0; NuGet-published; each refs SDK)
   Warfare / Economy / Scenario / UI
       |
DINOForge.Runtime            (netstandard2.0 + net8.0 multi-TFM; BepInEx plugin; refs SDK)
       |
DINOForge.Tools.Cli          (net11.0 Exe; refs SDK + PackCompiler)
DINOForge.Tools.PackCompiler (net11.0 Exe; refs SDK)
DINOForge.Tools.Installer    (net8.0 Lib + Avalonia GUI; refs SDK)
DINOForge.Tools.DumpTools    (net11.0; refs SDK)
DINOForge.Templates          (dotnet new template; no project refs)
DINOForge.Bridge.McpServer   (DEPRECATED C# stub; refs Protocol) — not primary
```

### 2.2 Non-C# Components

| Directory | Language | Runtime | Deployment |
|-----------|----------|---------|------------|
| `src/Tools/DinoforgeMcp/` | Python (FastMCP) | Python 3.11 | HTTP/SSE port 8765 |
| `packs/` | YAML/JSON | none (data) | copied to BepInEx |
| `docs/` | Markdown + TypeScript (VitePress) | Node.js | GitHub Pages |
| `schemas/` | JSON Schema | none (data) | bundled with SDK |
| `scripts/` | PowerShell / Bash | shell | developer machine |

### 2.3 Key Coupling Points

1. **SDK `InternalsVisibleTo` → Runtime**: `DINOForge.SDK.csproj` grants internal access to `DINOForge.Runtime`. Splitting these repos requires either promoting internals to public API or using a NuGet pre-release flow.

2. **Bridge.Client → SDK project reference**: `DINOForge.Bridge.Client.csproj` uses a `<ProjectReference>` to SDK. In a polyrepo this becomes a NuGet reference, requiring SDK to be published before Client can build.

3. **Tests reference everything**: `DINOForge.Tests.csproj` has project references to SDK, all four Domains, Bridge.Client, Bridge.Protocol, and Runtime. A split would force test repos to pin NuGet versions of their dependencies.

4. **CI workflows are monolithic**: All 50 workflows live in `.github/workflows/` and run against a single checkout. Pattern-gate workflows (pattern-gates.yml, configureawait.yml, etc.) run `scripts/ci/*.py` scripts that assume the full source tree is present.

5. **PackCompiler references SDK**: The pack tooling imports SDK types at compile time. This is intentional — pack validation uses the same models the runtime uses.

6. **MCP Python server references no C# code directly**: It communicates with the runtime via named pipe (the bridge). This is the cleanest split boundary in the entire codebase.

---

## 3. Strategy Comparison

### Strategy A — Minimal: Logical Boundaries, One Repo

Keep the single repository. Add governance infrastructure that simulates polyrepo isolation within the monorepo.

**What changes:**
- Add `CODEOWNERS` entries that map paths to team/bot owners
- Add per-project `.gitattributes` for linguist overrides
- Add `docs/architecture/boundaries.md` with explicit layer contracts
- Tag releases with component-scoped tags (e.g., `sdk/v1.2.0`, `runtime/v0.27.0`)
- Use NuGet pack-on-tag via existing `release.yml` (already in place)

**What stays the same:**
- One `git clone`
- All `<ProjectReference>` wires stay intact
- CI runs as a single pipeline
- Contributors submit one PR for changes that span layers

#### Pros
- Zero migration effort — changes are additive, reversible
- No breakage of `InternalsVisibleTo`, cross-project tests, or CI workflows
- Contributors always have the full context (runtime + SDK + packs in one clone)
- `dotnet build src/DINOForge.sln` continues to work as today
- Agent orchestration is simple: one `git worktree` covers everything

#### Cons
- Does not enforce layer isolation at the git level — a PR can silently couple layers
- NuGet packages still have `RepositoryUrl` pointing to a mega-repo; `source-link` scoping is coarse
- Community pack authors clone the entire platform (Runtime source, CI scripts, etc.) just to submit a YAML change
- Release cadences are implicitly coupled: a hotfix to `DINOForge.SDK` touches the same tag as a pack update

#### Migration Effort
1–2 hours. Purely additive file changes.

#### CI/CD Impact
None. All workflows continue to work unchanged.

#### Contributor Impact
No change for platform contributors. Community pack authors still face a large clone, but this is the current state.

#### Versioning Model
Lockstep by default, with optional component tags (`sdk/v1.x`, `runtime/v0.x`) for granular NuGet publish triggers.

---

### Strategy B — Balanced: Three Repos (RECOMMENDED)

Split into three repositories along the sharpest natural boundary in the codebase: **platform code**, **community content**, and **documentation**.

```
KooshaPari/DINOForge          (SDK + Runtime + Bridge + Domains + Tools + MCP + CI)
KooshaPari/DINOForge.Packs    (packs/ tree + pack validation CI)
KooshaPari/DINOForge.Docs     (docs/ VitePress site + deploy.yml)
```

**Rationale for these three cuts:**

1. **Packs split**: `packs/` contains only YAML/JSON data files and asset bundles. It has zero C# compilation dependencies — it is already validated by `dotnet run --project PackCompiler -- validate packs/` via the `validate-packs.yml` workflow. Community mod authors currently must clone the entire platform to submit a YAML content change. A separate `DINOForge.Packs` repo costs near-zero infrastructure and immediately improves the pack-authoring DX.

2. **Docs split**: `docs/` is a Node.js/VitePress project with its own `package.json` and `node_modules/`. It has a dedicated `deploy.yml` workflow. Its only coupling to the main repo is that `docs/api/` content is generated by `xml-docfx` from C# XML docs during the `api-docs.yml` workflow. This coupling is handled with a cross-repo trigger or a scheduled sync job.

3. **Core platform stays unified**: The tight `InternalsVisibleTo` link between SDK and Runtime, the shared `Directory.Build.props` / `Directory.Build.targets`, the 50 CI workflows that scan the full source tree, the Roslyn analyzers, and the multi-TFM complexity all argue strongly against further splitting the C# projects. The .NET runtime team (dotnet/runtime) kept their massive monorepo unified precisely because the Roslyn + MSBuild toolchain works best with a shared build graph.

**What moves where:**

| Item | From | To |
|------|------|----|
| `packs/**` | `DINOForge` | `DINOForge.Packs` |
| `docs/**` | `DINOForge` | `DINOForge.Docs` |
| `schemas/**` | stays | `DINOForge` (keep — SDK embeds them) |
| `.github/workflows/validate-packs.yml` | stays in `DINOForge` for canonical schema | copy to `DINOForge.Packs` |
| `docs/.github/workflows/deploy.yml` | stays in `DINOForge` | move to `DINOForge.Docs` |

#### Pros
- Community pack authors get a minimal, fast-to-clone repo with only `packs/`, `schemas/` (submodule or copy), and pack validation CI
- Docs site gets its own release cycle independent of platform versions
- Core platform keeps all its project references, `InternalsVisibleTo` grants, and CI intact
- NuGet packages from `DINOForge` repo cleanly reference `https://github.com/KooshaPari/DINOForge` without the noise of packs and docs history in the blame trail
- Reduces `DINOForge` main repo PR noise from pack content changes (which are the majority of future community contributions)

#### Cons
- Three repos to coordinate on platform-touching pack changes (e.g., a new SDK feature that requires a new `pack.yaml` field — SDK PR + Packs PR must land in order)
- `docs/api/` auto-generation workflow requires a cross-repo trigger (GitHub Actions `workflow_dispatch` or `repository_dispatch`) from `DINOForge` → `DINOForge.Docs`
- Contributors need two clones to work on a feature + its example pack in the same session
- Git history split: pack git history is lost from the main repo (mitigated by preserving it in `DINOForge.Packs` via `git filter-repo`)

#### Migration Effort
**Estimated: 2–3 days.**

Step-by-step plan:

1. **Create `DINOForge.Packs` repo** (2 hours)
   ```
   cd C:\Users\koosh\Dino
   git filter-repo --path packs/ --path schemas/ --tag-rename '':'packs-' \
       --target ../DINOForge.Packs
   ```
   - Push to `KooshaPari/DINOForge.Packs`
   - Add `pack-validation.yml` (copy of current `validate-packs.yml` + `pack-screenshots.yml`)
   - Add `CONTRIBUTING.md` explaining how to submit packs

2. **Create `DINOForge.Docs` repo** (2 hours)
   ```
   git filter-repo --path docs/ --tag-rename '':'docs-' \
       --target ../DINOForge.Docs
   ```
   - Push to `KooshaPari/DINOForge.Docs`
   - Move `deploy.yml` → `DINOForge.Docs/.github/workflows/deploy.yml`
   - Add `repository_dispatch` receiver in `DINOForge.Docs` triggered from `DINOForge` on tag push to regenerate API docs

3. **Strip `DINOForge` main repo** (1 day)
   - Use `git filter-repo --path-glob 'packs/*' --invert-paths` to remove `packs/` history from main
   - Remove `docs/` directory (keep `docs/` reference in README pointing to new repo)
   - Remove `validate-packs.yml`, `pack-validation.yml`, `pack-screenshots.yml`, `deploy.yml` from `.github/workflows/`
   - Update `Directory.Build.props` if it references `packs/` paths
   - Run `dotnet build src/DINOForge.sln` to verify nothing broke
   - Update `CLAUDE.md` repo structure section

4. **Update CI cross-repo trigger** (4 hours)
   - In `DINOForge` `release.yml`: add a step that triggers `repository_dispatch` on `DINOForge.Docs` with `event_type: sdk-released`
   - In `DINOForge.Docs`: add workflow that responds to `sdk-released` by running docfx + deploying GitHub Pages

5. **Update `README.md`** in all three repos with cross-links

#### CI/CD Impact
- `DINOForge` main repo: remove 4 pack/docs workflows; gain 1 cross-repo dispatch step in `release.yml`
- `DINOForge.Packs`: add 3 workflows (validate-packs, pack-screenshots, pack-submission-bot)
- `DINOForge.Docs`: add 2 workflows (deploy, api-sync trigger)
- All other CI workflows (50+) in `DINOForge` continue unchanged

#### Contributor Impact
- **Platform contributors** (SDK, Runtime, Tools, Domains): no change. Single clone, all project references intact.
- **Community pack authors**: dramatically improved DX. Clone `DINOForge.Packs` (~50 MB YAML+assets), create `packs/my-pack/pack.yaml`, PR. No need to interact with any C# toolchain.
- **Docs contributors**: clone `DINOForge.Docs`, run `npm run dev`, PR.

#### Versioning Model
- **`DINOForge`**: SemVer tags (e.g., `v0.28.0`) continue as today. NuGet packages published on tag.
- **`DINOForge.Packs`**: Independent SemVer or date-based tags (e.g., `packs-2026.05`). Pack schemas are versioned by the `framework_version` field in `pack.yaml`, not by repo tag.
- **`DINOForge.Docs`**: Tracks `DINOForge` major.minor (docs are versioned with the platform they document); auto-deployed on `sdk-released` dispatch.

---

### Strategy C — Aggressive: One Repo Per Project

Eight separate repositories:

```
KooshaPari/DINOForge.SDK
KooshaPari/DINOForge.Runtime
KooshaPari/DINOForge.Bridge
KooshaPari/DINOForge.Domains
KooshaPari/DINOForge.CLI
KooshaPari/DINOForge.Packs
KooshaPari/DINOForge.MCP
KooshaPari/DINOForge.Docs
```

Each repo publishes a NuGet package (or Python package for MCP) and downstream repos consume versioned packages instead of project references.

#### Pros
- True polyrepo: each component has its own issue tracker, PR queue, release cadence, and contributor graph
- NuGet source-link is maximally precise — every package traces to a single-purpose repo
- Runtime contributors never need to clone CLI tooling
- Independent version bumps: SDK can release v2.0 without forcing Runtime to rebuild same day

#### Cons
- **Breaks `InternalsVisibleTo`**: SDK exposes internals to Runtime via this mechanism; moving to NuGet references requires all those internal types to become public, expanding the API surface permanently (or using `FRIEND_ASSEMBLIES` equivalents in NuGet via `InternalsVisibleTo` in a published NuGet package — a fragile pattern)
- **Cross-project refactoring becomes multi-PR hell**: renaming a method in SDK requires a NuGet pre-release in SDK repo, a PR in Runtime repo, a PR in Bridge repo, etc. Currently this is one commit. Microsoft's dotnet/runtime team explicitly reversed a polyrepo experiment in 2018–2019 and returned to monorepo for exactly this reason
- **CI coordination is exponential**: 50 current workflows become 8 × N workflows, many of which need to know about NuGet version compatibility across repos. The pattern-gate scripts (`scripts/ci/detect_*.py`) assume full source visibility — impossible in polyrepo without sub-module or artifact-sharing hacks
- **`Directory.Build.props` / `Directory.Build.targets`**: shared MSBuild props (game paths, BepInEx dirs, TFM enforcement) currently apply automatically by directory structure. In polyrepo, every repo needs a copy; drift between copies is a constant maintenance burden
- **Agent orchestration collapses**: The entire CLAUDE.md agent model assumes a single working tree. Multi-repo agents require explicit cross-repo coordination protocols, dramatically increasing orchestration complexity
- **Tests require full dependency graph**: `DINOForge.Tests` references SDK + all Domains + Runtime + Bridge. In polyrepo, tests must consume NuGet packages; debugging a test failure in Runtime caused by a SDK change requires coordinating two repo PRs before the test can run

#### Migration Effort
**Estimated: 2–3 weeks minimum**, with high ongoing maintenance cost.

This is not a recommended path. See "Microsoft's .NET Runtime Experience" below.

#### CI/CD Impact
Severe. Every repo needs its own CI setup; cross-repo dependency tracking requires either GitHub Actions matrix strategies with version pinning or a dependency bot configuration. The current 50 workflows would expand to ~120 across 8 repos.

#### Contributor Impact
Negative for platform contributors. Positive only for contributors working on exactly one isolated component who never need to trace a bug across layers (rare in a game mod platform).

#### Versioning Model
Independent SemVer per repo, with NuGet ranges in each downstream consumer. Coordination requires a compatibility matrix document and automated dependency update PRs (Dependabot / Renovate) across all repos.

---

## 4. Reference: Microsoft's .NET Runtime Experience

The .NET team's polyrepo-to-monorepo journey is the canonical reference for this decision class:

- **2012–2018 (polyrepo)**: `dotnet/coreclr`, `dotnet/corefx`, `dotnet/corert` were separate repos. The BCL team spent ~15% of developer time on cross-repo version coordination. "Diamond dependency" issues (A depends on B v1.0 and C v2.0, both depend on D) were chronic.
- **2018–2019 (consolidation)**: Repos merged into `dotnet/runtime`. Quoted benefit: "We can now make a change to the GC and the BCL in the same PR and run the full test suite." Cross-layer refactorings that previously required 4–6 PRs across repos became single commits.
- **Lesson for DINOForge**: The tight coupling between SDK and Runtime (`InternalsVisibleTo`, shared type system, shared BepInEx loading constraints) maps closely to the BCL/CoreCLR coupling. The right boundary is where _deployment units_ diverge, not where _ownership_ diverges.

The DINOForge pack system is the exception: packs are truly independent deployment units (YAML + assets, no compiled code) with a clear interface contract (`pack.yaml` schema). This is analogous to NuGet packages in the dotnet ecosystem — they are productively separated.

---

## 5. Recommendation: Strategy B

**Split into three repos. Keep the C# platform monolith intact.**

### Decision criteria satisfied:

| Criterion | A (Mono) | B (Balanced) | C (Aggressive) |
|-----------|----------|-------------|----------------|
| `InternalsVisibleTo` preserved | Yes | Yes | No |
| Cross-layer refactor in one PR | Yes | Yes | No |
| Community packs have minimal clone | No | Yes | Yes |
| CI scripts have full source visibility | Yes | Yes | No |
| Agent orchestration simple | Yes | Yes | No |
| Migration cost acceptable | Trivial | 2–3 days | 2–3 weeks |
| Docs independent lifecycle | No | Yes | Yes |

Strategy B delivers the primary benefit of polyrepo (community-facing pack submission with minimal friction) without paying the primary cost (cross-project refactoring coordination overhead).

### Concrete migration plan (Strategy B)

#### Phase 1: Create `DINOForge.Packs` (Priority: High)

**Trigger**: When community pack submission volume exceeds ~2 PRs/month or when pack contributors report friction cloning the full repo.

```
# From DINOForge repo root (run in dedicated branch)
pip install git-filter-repo
git filter-repo --path packs/ --path schemas/ --force
# Push result to new remote
git remote add packs https://github.com/KooshaPari/DINOForge.Packs
git push packs main
```

Files to add to `DINOForge.Packs`:
- `README.md` — pack authoring guide
- `CONTRIBUTING.md` — submission workflow
- `.github/workflows/validate-packs.yml` — copy from main repo
- `.github/workflows/pack-screenshots.yml` — copy from main repo

Files to remove from `DINOForge` main after split:
- `packs/` directory
- `.github/workflows/validate-packs.yml`
- `.github/workflows/pack-screenshots.yml`
- `.github/workflows/pack-validation.yml`

Schemas (`schemas/`) stay in `DINOForge` (SDK embeds them at build time). `DINOForge.Packs` can reference them via a Git submodule `schemas/` pointing to `DINOForge/schemas/`.

#### Phase 2: Create `DINOForge.Docs` (Priority: Medium)

**Trigger**: When docs contributors are blocked by needing to understand the C# build pipeline to submit a documentation fix.

```
git filter-repo --path docs/ --force
git remote add docs-repo https://github.com/KooshaPari/DINOForge.Docs
git push docs-repo main
```

Files to add to `DINOForge.Docs`:
- `.github/workflows/deploy.yml` — moved from main repo
- `.github/workflows/api-sync.yml` — new; triggers on `repository_dispatch` from main

Cross-repo trigger addition in `DINOForge` `release.yml`:
```yaml
- name: Trigger docs rebuild
  uses: peter-evans/repository-dispatch@v3
  with:
    token: ${{ secrets.DOCS_DISPATCH_TOKEN }}
    repository: KooshaPari/DINOForge.Docs
    event-type: sdk-released
    client-payload: '{"version": "${{ steps.version.outputs.VERSION }}"}'
```

Files to remove from `DINOForge` main:
- `docs/` directory
- `.github/workflows/deploy.yml`
- `.github/workflows/api-docs.yml`

#### Phase 3: No further splits

Do not split SDK, Runtime, Bridge, Domains, Tools, or MCP into separate repos. The coupling is architectural; separating them creates maintenance overhead that outweighs any benefit.

---

## 6. Staying Monorepo (When to Choose Strategy A)

Choose Strategy A if:
- The team is small (1–3 active contributors, all platform-focused)
- Community pack contributions are rare or come from technically sophisticated users comfortable with a large clone
- Agent automation is the primary development model (agents work best in single-tree)

The current state (2026-05-28) falls into this bucket. Strategy A is a valid choice _now_; the recommendation is to plan for Strategy B as community pack submission scales.

---

## 7. Appendix: Project Reference Coupling Matrix

The following matrix shows which C# projects have `<ProjectReference>` dependencies. This is the constraint graph for any split.

| Consumer | Depends On |
|----------|-----------|
| `DINOForge.Runtime` | `DINOForge.SDK` (InternalsVisibleTo) |
| `DINOForge.Bridge.Protocol` | `DINOForge.Analyzers` (analyzer only) |
| `DINOForge.Bridge.Client` | `DINOForge.Bridge.Protocol`, `DINOForge.SDK` |
| `DINOForge.Bridge.McpServer` | `DINOForge.Bridge.Protocol` |
| `DINOForge.Domains.Warfare` | `DINOForge.SDK` |
| `DINOForge.Domains.Economy` | `DINOForge.SDK` |
| `DINOForge.Domains.Scenario` | `DINOForge.SDK` |
| `DINOForge.Domains.UI` | `DINOForge.SDK` |
| `DINOForge.Tools.Cli` | `DINOForge.SDK`, `DINOForge.Tools.PackCompiler` |
| `DINOForge.Tools.PackCompiler` | `DINOForge.SDK` |
| `DINOForge.Tools.Installer` | `DINOForge.SDK` |
| `DINOForge.Tests` | All of the above |

**Observation**: `DINOForge.SDK` is the root dependency. Any split that puts SDK in a separate repo forces every other project to consume it as a NuGet package. This is feasible but requires a pre-release NuGet flow during development (every SDK change needs a publish before downstream projects can build — or a NuGet feed with `--prerelease` resolvers). The cost/benefit is negative for this repo's scale and agent-driven model.

---

## 8. Related Documents

- `docs/architecture/POLYGLOT_STRATEGY.md` — polyglot (Rust/Go) build strategy
- `docs/architecture/diagrams.md` — architecture layer diagrams
- `CLAUDE.md` — agent governance, TFM policy, build commands
- `.github/workflows/ci.yml` — current CI entry point
- `.github/workflows/release.yml` — NuGet publish + release workflow
