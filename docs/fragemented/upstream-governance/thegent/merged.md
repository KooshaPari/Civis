# Merged Fragmented Markdown

## Source: upstream-governance/thegent/AGENTS.md

# Global Claude Code Instructions

These rules apply to ALL projects. Project-level CLAUDE.md files supplement (and may override) these.

---

# 🔒 CRITICAL SECURITY RULES - NEVER VIOLATE

## ⛔ FORBIDDEN: Killing Agent or Terminal Processes

**ABSOLUTELY FORBIDDEN** - Agents MUST NEVER kill other agent processes or terminal processes.

### ❌ NEVER RUN THESE COMMANDS:
```bash
# FORBIDDEN - Killing cursor-agent (EXACT PATTERN YOU MUST NOT USE)
ps -ao pid,command | grep "cursor-agent" | grep -v grep | grep -v 40690 | awk '{print $1}' | xargs kill -9
ps | grep cursor-agent | xargs kill -9
pkill cursor-agent
killall cursor-agent

# FORBIDDEN - Killing any agent process
kill -9 <pid>  # where PID is cursor-agent, thegent, claude, codex, droid, etc.

# FORBIDDEN - Killing shell/terminal processes  
kill -9 <pid>  # where PID is bash, zsh, sh, ghostty, terminal, iterm, etc.
```

### ✅ CORRECT ALTERNATIVES:
```bash
# Safe cleanup of orphaned LSP/MCP processes
thegent mcp prune

# See what would be pruned (dry run)
thegent mcp prune --dry-run

# List active sessions
thegent ps

# Properly stop a session
thegent stop <session_id>
```

### 🛡️ PROTECTED PROCESSES:
The following processes are PROTECTED and MUST NEVER be killed:
- **Agent processes**: `cursor-agent`, `thegent`, `claude`, `codex`, `droid`, `opencode`, `copilot`, `gemini`
- **Shell processes**: `bash`, `zsh`, `sh`, `fish`, `tcsh`, `csh`
- **Terminal emulators**: `ghostty`, `terminal`, `iterm`, `alacritty`, `kitty`, `wezterm`, `warp`

### ⚠️ SECURITY ENFORCEMENT:
- All commands are validated before execution
- Commands attempting to kill protected processes will be BLOCKED
- Violations are logged and reported
- Rate limiting prevents abuse

**If you need to clean up processes, use the safe pruning tools provided by thegent, NOT manual kill commands.**

---

## ⛔ FORBIDDEN: Fallbacks, Legacy Compatibility, and Silent Failures

**ABSOLUTELY FORBIDDEN** - Agents MUST NEVER add fallbacks, legacy compatibility, or silent error handling.

### ❌ NEVER ADD:
- **Fallback code paths**: `try: new(); except: old()` or `try: fast(); except: slow()`
- **Legacy compatibility shims**: `if legacy_flag: old(); else: new()`
- **Backwards compatibility layers**: `def old(): warnings.warn(); return new()`
- **Silent error handling**: `try: thing(); except: pass` or `try: thing(); except: return default`
- **Error hiding**: `try: thing(); except: delete_from_db()` (hiding bugs)
- **"Just in case" code**: Code added "just in case" something fails
- **Import fallbacks**: `try: from X import Y; except: from Z import Y`
- **Migration systems for simple changes**: Don't create versioning/migration for simple edits

### ✅ CORRECT APPROACH:
- **Code should FAIL and STOP** on errors - fail fast, fail loudly
- **No fallbacks** unless explicitly requested (and even then, prefer fixing the root cause)
- **No legacy compatibility** - Zero user debt = zero backwards compatibility
- **No silent failures** - All errors must be visible and logged
- **Fix bugs, don't hide them** - If something fails, fix it, don't work around it
- **Verify parity BEFORE removal** - Always verify feature parity and migration completeness before removing code

### 🎯 "Aim Towards" Framing:
When removing code, frame it positively:
```
BAD: "Don't add fallbacks"
GOOD: "Now that we have fully transitioned to a new system and it has been 
confirmed to work as intended, let's clean out all backwards compatibility 
and fallbacks so we have a DRY, modular system with clear and clean separation 
of responsibilities. Once finished, we have a fresh system with no technical debt."
```

### ⚠️ AI AGENT PATTERN:
AI coding agents (Claude, Codex, ChatGPT) have a **systemic tendency** to add fallbacks and legacy compatibility even when explicitly told not to. This is a known pattern requiring:
- **Explicit rules** (like this section)
- **"Aim towards" framing** (positive direction, not just "don't do X")
- **Fail fast philosophy** (code should fail and stop)
- **Parity verification** (verify before removal)
- **CI checks** (automated detection of fallback patterns)

### 🛡️ ENFORCEMENT:
- All code is checked for fallback patterns
- Commits with fallbacks will be flagged
- Silent error handling is detected and blocked
- Legacy compatibility code is identified and removed

**Remember: Zero user debt = zero backwards compatibility. All changes are breaking changes by design. Code should fail fast and fail loudly, not silently work around problems.**

---

# Heavy Web Research Policy
- Use DuckDuckGo (`ddg_search`) for comprehensive web research when local knowledge is insufficient.
- Prefer `duckduckgo-search` library for programmatic access.
- Summarize findings for the user, providing links only for deep dives.

---

# Library-First Policy

**CRITICAL**: Prefer **library + thin wrapper** over full custom implementation. Apply from the start of development and throughout. Before writing code: "Is there a library?" — first question for generic problems (retry, cache, file watch, circuit breaker). See: `docs/research/LIBRARY_FIRST_AUDIT_AND_PLAN.md`, `docs/guides/anti-patterns.md`

**Proactive Governance**: Do not wait for the user to ask. When work touches a governance domain, check governance; if missing or outdated, add/update as part of the same task. If you see a gap, update it.

---

# Context Management Strategy

## The Manager Pattern

**CRITICAL**: Operate as a strategic manager, not a worker. Delegate to subagents.

### Keep in Main Context
- User intent and requirements
- Strategic decisions and trade-offs
- Summaries of completed work
- Critical architectural knowledge

### Delegate to Subagents
- File exploration (>3 files)
- Pattern searches across codebase
- Multi-file implementations
- Long command sequences
- Test execution

## When to Delegate Code (Decision Guide)

**Delegate** when:
- **Scope**: Changes span >3 files or multiple modules
- **Exploration**: Need to search patterns across the codebase
- **Context budget**: Task would add >2000 tokens of file content/output
- **Independence**: Work can be done in isolation with clear handoff
- **Long-running**: Test suites, builds, or multi-step sequences

**Handle directly** when:
- **Single-file**: One file, one concern, clear fix
- **Quick answer**: User needs info, not implementation
- **Config/tweak**: Small Taskfile, env, or script change
- **<3 files**: Limited scope, you can hold it in context

**Rule of thumb**: If you would need to read >3 files to implement correctly, delegate exploration first and get a summary. If the implementation touches >3 files, delegate to `general-purpose` or a task agent.

## Strategy Quick Reference

| Need | Tool/Provider | Example Prompt |
|------|---------------|----------------|
| Heavy Web Research | DuckDuckGo (`ddgr`) | "Search DDG for latest VitePress plugins" |
| Find code patterns | `Explore` | "Find all error handling patterns" |
| Design approach | `Plan` | "Design auth implementation strategy" |
| Run commands | `Bash` | "Run test suite and report failures" |
| Multi-step implementation | `general-purpose` | "Implement and test feature X" |
| Quick isolated fix | DO NOT delegate | Handle directly |

## DuckDuckGo Search Mandate
- Use `ddgr` (or equivalent DDG tool) for all heavy web research.
- Prefer DuckDuckGo over other search engines for privacy and agent-friendliness.
- Research tasks should prioritize finding up-to-date documentation and community-driven solutions.

### Parallel vs Sequential

**Parallel** (no dependencies): Launch 2-3 explore agents simultaneously for independent searches.

**Sequential** (dependent): explore -> receive summary -> plan based on findings -> implement approved plan.

## Subagent Swarm (async orchestration)

**If you have subagent/swarm capabilities:** Use them as an **async swarm**.

- **Call task agents async.** Fire tasks so that as each completes, you are reawoken to re-evaluate, spawn more agents, or do more work yourself.
- **Run a swarm.** Up to **50 concurrent task agents**. Scale up when work is well decomposed and independent.
- **Work in between.** While tasks run async, use your own context for planning, monitoring, or other work.
- **Reawaken on completion.** When idle, you will be reawoken as each agent completes. Use that to spawn more agents, do follow-up work, or consolidate results.

## Anti-Patterns

| Bad | Good |
|-----|------|
| Reading 10 files to "understand" | Delegate exploration, get summary |
| Editing files for multi-file changes | Delegate to `general-purpose` |
| Sequential explorations one-by-one | Batch parallel explores |
| Asking subagent for "all results" | Ask for "summary" or "key files" |
| **Workspace Cleanup**: Running `git restore .` or `git clean` to "reset" the environment | **Respect Work**: Leave modified files alone; assume they are active tasks from other agents. |
| Overwriting a "dirty" file with your version | Merge or work around existing changes. |

## Context Budget Rule

If task adds >2000 tokens of file content/output, **delegate it**.

---

# Optionality and Failure Behavior

**Require** dependencies where they belong; **require** clear, loud failures -- no silent or "graceful" degradation.

- **Force requirement where it belongs.** Do not make dependencies "optional" just to avoid failure. If a service or config is required for correctness, treat it as required and fail when missing.
- **Fail clearly, not silently.** Use explicit failures -- not reduced functionality, logging-only warnings, or hidden errors. Users must see *what* failed and that the process did not silently degrade.
- **Graceful in other ways.** Retries with visible feedback (e.g. "Waiting for X... (2/6)"); error messages that list each failing item; actionable messages and non-obscure stack traces. Do *not* use optionality or silent fallbacks as a substitute for fixing the real dependency.

---

# Planner Agents: No Code in Docs or Plans

**Planner agents** (PM, Analyst, Architect, etc.) must **never write code** in documentation and plans. Their job is to equip implementers. Write specs, acceptance criteria, architecture decisions, and clear handoffs. Prefer references, file paths, or brief pseudocode when necessary.

---

# Phased WBS and Plans with DAGs

When generating **plans**, **roadmaps**, or **implementation breakdowns**:

- **Phases:** Structure into ordered phases (Discovery, Design, Build, Test/Validate, Deploy/Handoff). Each phase contains deliverable-oriented work packages.
- **DAG:** Tasks have explicit **predecessors**; no cycles. List dependencies so execution order is unambiguous.
- **Output:** Phased WBS (hierarchy by phase) plus dependency list or DAG. Optionally: **Phase | Task ID | Description | Depends On** table.

---

# Timescales: Agent-Led, Aggressive Estimates

**Assume an agent-driven environment.** No user or external human intervention beyond prompts.

- **Forbidden in plans:** "Schedule external audit", "Stakeholder Presentation", "Team Kickoff", "Human checkpoint", "Get approval from X", or any step assigning work to a human.
- **Effort in agent terms only:** Agent actions (tool calls, subagent batches). Aggressive wall-clock -- err on the lower bound.
- **Rough mapping:**
  - Trivial change: 1-2 tool calls, <1 min
  - Small feature: 3-6 tool calls, 1-3 min
  - Cross-stack feature: 8-15 tool calls or 2-3 parallel subagents, 3-8 min
  - Major refactor: 15-30 tool calls or 3-5 parallel subagents, 8-20 min
  - Multi-phase initiative: decompose into agent batches; each batch 10-20 min max
- **Forbidden phrasing:** "This will take 2 days", "Schedule a review", "Assign owners", "Present to stakeholders". Use: "N tool calls", "N parallel subagents", "~M min wall clock".

---

# Documentation Organization

**CRITICAL**: All project documentation follows a strict organization structure.

### Root-Level Files (Keep in Root)
- `README.md` -- Main project documentation
- `CHANGELOG.md` -- Project changelog
- `AGENTS.md` -- AI agent instructions
- `CLAUDE.md` -- Claude-specific instructions
- `00_START_HERE.md` -- Getting started guide (if applicable)
- Spec docs: `PRD.md`, `ADR.md`, `FUNCTIONAL_REQUIREMENTS.md`, `PLAN.md`, `USER_JOURNEYS.md`

### Documentation Structure

All other `.md` files must be organized in `docs/` subdirectories:

```
docs/
  guides/              # Implementation guides and how-tos
    quick-start/       # Quick start guides
  reports/             # Completion reports, summaries, status reports
  research/            # Research summaries, indexes, analysis
  reference/           # Quick references, API references, trackers
  checklists/          # Implementation checklists, verification lists
  changes/             # Per-change proposal/design/task docs
    archive/           # Completed change docs
```

### File Organization Rules

1. **Quick Starts** -> `docs/guides/quick-start/` (`*QUICK_START*.md`, `*QUICKSTART*.md`)
2. **Quick References** -> `docs/reference/` (`*QUICK_REFERENCE*.md`, `*QUICK_REF*.md`)
3. **Implementation Guides** -> `docs/guides/` (`*GUIDE*.md`)
4. **Completion Reports** -> `docs/reports/` (`*COMPLETE*.md`, `*SUMMARY*.md`, `*REPORT*.md`, `PHASE_*.md`, `*TEST*.md`)
5. **Research Files** -> `docs/research/` (`*RESEARCH*.md`, `*INDEX*.md`)
6. **Checklists** -> `docs/checklists/` (`*CHECKLIST*.md`)
7. **Trackers** -> `docs/reference/` (`*TRACKER*.md`, `*STATUS*.md`, `*MAP*.md`)

### AI Agent Instructions

- **NEVER** create `.md` files in the project root (except allowed root-level files above)
- **ALWAYS** place new documentation in the appropriate `docs/` subdirectory
- **VERIFY** file location before creating documentation
- **MOVE** misplaced files to correct subdirectories if found

---

# Opinionated Quality Enforcement

- Enforce opinionated styling to a strict degree.
- **Quality Gate**: Always run `task quality` (full strict pipeline: max-lines, lint, core-boundary, deprecated-aliases, instruction-architecture, harness-contracts, runtime-contracts) before stopping work.
- Rather than disables or ignores, fix code properly.
- Use project linters, formatters, and type checkers. Never bypass them.

---

# Specification Documentation System

## Required Project Documentation

Every non-trivial project SHOULD maintain these spec docs (root level):

| File | Purpose |
|------|---------|
| `PRD.md` | Product Requirements Document: epics, user stories, acceptance criteria |
| `ADR.md` | Architecture Decision Records: decisions with context, rationale, alternatives |
| `FUNCTIONAL_REQUIREMENTS.md` | Functional Requirements: SHALL statements, traces to PRD |
| `PLAN.md` | Phased WBS with DAG dependencies |
| `USER_JOURNEYS.md` | User journeys with ASCII flow diagrams |

## Required Tracker Documentation

Projects with spec docs SHOULD maintain trackers in `docs/reference/`:

| File | Purpose |
|------|---------|
| `PRD_TRACKER.md` | Epic/story status, progress %, code locations |
| `ADR_STATUS.md` | ADR implementation status, code artifacts |
| `FR_TRACKER.md` | FR implementation status, test coverage |
| `PLAN_STATUS.md` | Phase/task completion status |
| `JOURNEY_VALIDATION.md` | Journey validation status, gaps |
| `CODE_ENTITY_MAP.md` | Forward and reverse mapping: code entities <-> requirements |

## Auto-Detection Behavior

**On session start:**
- If spec docs are missing, acknowledge it and offer to generate them
- Greenfield project: offer to scaffold all spec docs from project analysis
- Brownfield project: offer to analyze existing codebase and generate docs mapping to what exists
- Do NOT auto-generate without user confirmation -- offer, don't force

## VitePress Docsite Setup (Greenfield/Brownfield)

**MUST include docsite setup in any new project initialization:**

For greenfield projects:
- Copy VitePress template from `thegent/templates/vitepress-full/` to new project
- Run `pnpm install && pnpm docs:build` to verify setup
- Document in project CLAUDE.md

For brownfield projects (existing projects without docsites):
- Check if `docs-dist/index.html` exists -- if not, propose adding docsite
- Use same template from `thegent/templates/vitepress-full/`
- Run `pnpm install && pnpm docs:build` to verify

**Quick setup (30 seconds):**
```bash
cp -r thegent/templates/vitepress-full myproject/docs/.vitepress
# Rename .template files, edit config.ts placeholders
cd myproject && pnpm install && pnpm docs:build
open docs-dist/index.html
```

**Why:** All projects should have statically viewable docs that can be opened via `file://` in browser.

---

## Project Setup Checklist (Greenfield/Brownfield)

**MUST initialize these for ALL new projects:**

### 1. Docsite (VitePress)
- [ ] Copy `thegent/templates/vitepress-full/` to `docs/.vitepress/`
- [ ] Run `pnpm install && pnpm docs:build`
- [ ] Verify `docs-dist/index.html` opens in browser
- [ ] Add to CLAUDE.md

### 2. Taskfile (NOT Make)
- [ ] Create `Taskfile.yml` with standard tasks:
  - `lint` - Run all linters
  - `test` - Run tests
  - `quality` - Run quality gates
  - `docs:build` - Build docsite

### 3. Linters (Language-Specific)
| Stack | Linter | Formatter | Config Template |
|-------|--------|-----------|---------------|
| Python | ruff | ruff format | `thegent/templates/python/pyproject.template.toml` |
| TypeScript | oxlint | oxfmt/prettier | `thegent/templates/typescript/oxlint.config.json` |
| Go | golangci-lint | gofumpt | `thegent/templates/go/.golangci.yml` |
| Rust | clippy | rustfmt | `thegent/templates/rust/clippy.toml` |
| Ruby | rubocop | rubocop | `thegent/templates/ruby/.rubocop.yml` |
| Java | checkstyle + spotbugs | google-java-format | `thegent/templates/java/checkstyle.xml` |
| C/C++ | clang-tidy | clang-format | `thegent/templates/cpp/.clang-tidy` |
| PHP | phpstan + psalm | PHP CS Fixer | `thegent/templates/php/phpstan.neon` |
| Bash | shellcheck | shfmt | `thegent/templates/bash/.shellcheckrc` |

### 4. Project Scaffolding Tools (CLI/App Frameworks)
| Stack | CLI Framework | Web Framework | Config |
|-------|--------------|---------------|--------|
| Python | typer | FastAPI/starlette | `pyproject.toml` |
| TypeScript | commander.js | Express/Fastify/Hono | `package.json` |
| Rust | clap | axum/actix | `Cargo.toml` |
| Go | cobra/urfave/cli | gin/echo/fiber | `go.mod` |
| Ruby | thor | Rails/Hanami | `Gemfile` |
| Java | picocli | Spring Boot | `pom.xml`/`build.gradle` |
| C# | commandline | ASP.NET Core | `.csproj` |

### 5. Pre-commit Hooks
- [ ] Add `.pre-commit-config.yaml`
- [ ] Include: ruff-check, ruff-format, gitleaks, trailing-whitespace
- [ ] Run `pre-commit install`

### 6. Quality Gates
- [ ] Create `hooks/quality-gate.sh` with lint/test/coverage/security checks
- [ ] Run on pre-commit or Stop hook

### 7. Test Infrastructure (Per Language)
| Stack | Test Runner | Coverage | Test Config |
|-------|-------------|----------|-------------|
| Python | pytest + pytest-xdist | coverage.py | `pyproject.toml` [tool.pytest] |
| TypeScript | vitest | v8 | `vitest.config.ts` |
| Rust | cargo test | tarpaulin/grcov | `Cargo.toml` |
| Go | go test | gocov/coverprofile | `_test.go` files |
| Ruby | rspec | simplecov | `.rspec` |
| Java | JUnit 5 | JaCoCo | `pom.xml`/`build.gradle` |
| C++ | catch2/doctest | lcov | `CMakeLists.txt` |
| PHP | phpunit | phpunit-coverage | `phpunit.xml` |
| Bash | bats-core | - | `*.bats` files |

### 8. Full Traceability Setup
- [ ] Create `FUNCTIONAL_REQUIREMENTS.md` with FR-{CAT}-NNN IDs
- [ ] Create `docs/reference/FR_TRACKER.md` to track FR implementation status
- [ ] Create `docs/reference/CODE_ENTITY_MAP.md` mapping code <-> requirements
- [ ] Add FR ID tags to all test functions:
  - Python: `@pytest.mark.requirement("FR-XXX-NNN")`
  - TypeScript: `describe("FR-XXX-NNN: description", () => {...})`
  - Rust: `#[test] fn test_FR_XXX_NNN() {...}`
  - Add docstring: `Traces to: FR-XXX-NNN`
- [ ] Verify: `grep -r "FR-" tests/` shows all FRs have tests
- [ ] Run: `task quality` to verify spec verification

### 9. CLAUDE.md Project Instructions
Create project-specific CLAUDE.md with project info, library preferences, domain patterns.

---

## Quick Project Initialization

### Option 1: Copier (Recommended)
```bash
# Install copier if needed
pip install copier

# Initialize with all prompts
copier copy thegent/templates/initialize-project ./my-new-project

# Or with options specified
copier copy thegent/templates/initialize-project ./my-new-project \
  --project-name="my-project" \
  --project-description="A description" \
  --language="python" \
  --include-docs=true \
  --include-ci=true
```

### Option 2: Manual Template Selection
```bash
# Full setup for new project:
mkdir -p docs hooks
cp -r thegent/templates/vitepress-full/* docs/.vitepress/
mv docs/package.json.template docs/package.json
pnpm install && pnpm docs:build
open docs-dist/index.html
```

### Available Templates

| Template | Location | Purpose |
|----------|----------|---------|
| CLAUDE.md | `templates/claude/CLAUDE.md.template` | Project-specific agent instructions |
| Taskfile | `templates/{language}/Taskfile.{language}.yml` | Build automation |
| Quality | `templates/quality/` | 50+ lint/coverage configs for 25+ languages |
| VitePress | `templates/vitepress-full/` | Full docsite with versioning |
| Specs | `templates/specs/` | PRD, ADR, FR, PLAN templates |
| CI/CD | `templates/operational/ci/` | GitHub Actions workflows |
| Docker | `templates/operational/docker/` | Dockerfiles & compose |

**During work:**
- When making significant code changes (new modules, features, architecture changes), note which spec docs would need updating
- When completing a task, mentally check if trackers should be updated
- If you add new functions/modules, note they should be added to CODE_ENTITY_MAP.md

**On session end:**
- If there are unmapped code changes, acknowledge and update trackers if appropriate
- Treat session end as a documentation checkpoint

## Change Documentation (per-change, for significant changes)

For significant changes (new features, major refactors, architecture changes):
- Create `docs/changes/{change-name}/` with:
  - `proposal.md` -- What and why
  - `design.md` -- Technical approach, affected components
  - `tasks.md` -- Implementation checklist
- Archive completed changes to `docs/changes/archive/`
- NOT required for small fixes, typos, or minor adjustments

## Doc Format Standards

- **ID systems:** E{n}.{m}.{k} for epics/stories, FR-{CAT}-{NNN} for requirements, ADR-{NNN} for decisions, P{n}.{m} for plan tasks, UJ-{N} for journeys
- **Cross-reference** between docs (FR traces to PRD epics, code maps to FRs and ADRs)
- **ASCII diagrams** for flows and architecture (not images)
- **Tables** for tracking, matrices, and summaries
- Templates are available at `~/.claude/templates/` for consistent formatting (if present)

### Global Reference Docs for Code Generation

**Use these references when generating code:**

| Domain | Reference Path |
|--------|---------------|
| UI Design | `docs/reference/UI_DESIGN_PRINCIPLES_REFERENCE.md` |
| Architecture | `docs/reference/SOFTWARE_ARCHITECTURE_REFERENCE.md` |
| Design Patterns | `docs/reference/SOFTWARE_DESIGN_PATTERNS_REFERENCE.md` |
| Performance | `docs/reference/performance/PERFORMANCE_OPTIMIZATION.md` |
| Testing | `docs/reference/testing/TESTING_STRATEGIES.md` |
| Security | `docs/reference/security/SECURITY_BEST_PRACTICES.md` |
| **Full Index** | `docs/reference/INDEX.md` |

For hyperspecialization, agents can use `docs/reference/INDEX.md` to find domain-specific references mapped to their roles.

## Session State Continuity

- The hooks system (if configured) tracks file changes per session via `.claude/session-changes.log`
- On stop, changes are reconciled against trackers
- This provides session-to-session continuity for documentation maintenance

---

# Generalized Dev Environment Pattern

## Service Management

- **The user runs a dev TUI/dashboard in their own terminal.** This is their primary observation interface. **Never** start, stop, or restart the entire dev stack (`make dev`, `make dev-tui`, `make dev-down`) — only the user does that.
- **Use CLI introspection and per-service manipulation commands** to interact with the running stack without disrupting the user's TUI session. Process orchestrators (e.g. `process-compose`) expose a CLI/API that operates on the same running instance.
- **Assume services use hot reload** (file watchers, HMR, etc.). Save files and let watchers pick up changes — do not restart services just because you edited files.
- **When a service needs restarting** (e.g. config change, dependency update, crash), restart only that specific service via CLI, not the whole stack.
- **Read logs via CLI or log files** — never attach to or interfere with the user's TUI terminal.
- Before starting dev yourself, verify processes are not already up (check health endpoints, status commands, or log files) to avoid duplicate stacks.

## Package Manager

**Use the project's preferred package manager.** Detect from lockfiles:
- `bun.lockb` or `bun.lock` -> use `bun`
- `pnpm-lock.yaml` -> use `pnpm`
- `yarn.lock` -> use `yarn`
- `package-lock.json` -> use `npm`
- If unclear, check `package.json` `packageManager` field or project CLAUDE.md

## Native Over Docker

**Prefer native services over Docker** for local development. Run databases, caches, reverse proxies as native processes. Use Docker only when native install is not feasible or explicitly required.

## OSS and Free First

**Strictly prefer local, OSS, and free tools** over paid SaaS. If a feature requires external services, prefer OSS/self-hosted or free options first. Document paid options only as labeled fallback.

## Multi-Actor Coordination (generalized)

When multiple agents or actors share a dev environment:
- **Concurrent Agent Environment**: Assume multiple agents are working in the same workspace simultaneously.
- **Git Safety - FORBIDDEN**: Never run `git restore`, `git reset`, or `git clean` on the workspace. These commands destroy work-in-progress from other agents.
- **Respect Dirty Files**: Modified files are active work-in-progress. Do not revert, "cleanup", or overwrite them unless specifically instructed to finish a task started by another agent. Work around existing changes.
- **Debounce commands:** Use project-provided wrappers (Makefile targets, scripts) that prevent concurrent execution conflicts.
- **Shared service management:** Use the project's process orchestrator as source of truth for service health.
- **Hold-if-running logic:** Prefer wrappers that allow multiple actors to share processes without force-killing.
- **Consolidated targets:** Prefer consolidated build/lint/test targets over a multitude of specialized ones.

---

# Plugin Ecosystem Awareness

Use AgilePlus for spec tracking: `agileplus \<command\>`

- Check available slash commands (`/`) for documentation workflows
- These complement the auto-detection system -- invoke explicitly for deeper workflows
- Auto-detection hooks handle common cases; plugins handle advanced scenarios
- Start a new conversation to switch agent personas

---

# QA Governance

## Test-First Mandate

- Write tests BEFORE implementation. Test file must exist before source file for new modules.
- For bug fixes, write a failing test that reproduces the bug first, then fix.
- Test naming: descriptive, includes the FR ID when applicable.

## Suppression Policy

- **Zero new suppressions** without inline justification comment.
- Acceptable format: `# noqa: E501 -- line is a long URL` (note the `--` reason separator).
- The `suppression-blocker.sh` hook will BLOCK any Write/Edit that introduces new suppressions.
- If a suppression is genuinely needed, include the specific rule code AND a reason.

## Spec Traceability

- All test functions MUST reference an FR ID via one of:
  - Tag: `# @trace FR-XXX-NNN` in test file or function
  - Marker: `@pytest.mark.requirement("FR-XXX-NNN")`
  - Docstring: `Traces to: FR-XXX-NNN`
  - Test name: `@test "FR-XXX-NNN: description"` (BATS)
- Orphaned FRs (no test) and orphaned tests (no FR) are reported by the quality gate.

## Quality Gate Awareness

- `quality-gate.sh` runs on every Stop event -- it reports lint, security, and traceability findings.
- **Proactively run linters** before finishing work to avoid surprises at the quality gate.
- The gate is advisory (does not block Stop) but findings should be addressed.

## Static Analysis Config

- When scaffolding a new project, copy relevant templates from `~/.claude/templates/quality/` for detected stacks.
- Available templates: ruff.toml, ty-config.toml, oxlintrc.json, tsconfig-strict.json, golangci.yml, clippy.toml, shellcheckrc, pre-commit-config.yaml, pytest-config.toml, coverage-config.toml.

## Test Pyramid Targets

**CRITICAL: Agent-Only Environment Requirement**

Since **NO humans will test this system** - only agents will use it - we require **100% coverage** for all test types:

- **E2E**: **100%** of all CLI commands (CRITICAL - agents interact at CLI boundary)
- **Integration**: **100%** of all workflows (CRITICAL - cross-component behavior)
- **Unit**: **100%** of all functions (ESSENTIAL - isolated behavior)

**Why 100%?** In agent-only environments:
- ❌ NO humans will manually test commands
- ❌ NO manual verification possible
- ✅ **ONLY automated tests can verify behavior**
- ✅ **100% coverage is REQUIRED, not optional**

**Legacy Projects** (with human testers) may override in `.qa-config.json` or `.claude/quality.json`:
- **Unit**: 70% (tolerance: +/-5%)
- **Integration**: 20% (tolerance: +/-5%)
- **E2E**: 10% (tolerance: +/-5%)

**Agent-Only Projects** (thegent and similar): **100% coverage required for all types**.

See `docs/governance/AGENT_ONLY_TEST_STRATEGY.md` and `docs/governance/TDD_BDD_SDD_GOVERNANCE.md` for complete requirements.

## Hook Pipeline Summary (v3)

| Event | Hooks (execution order) |
|-------|------------------------|
| SessionStart | spec-preflight, qa-preflight |
| UserPromptSubmit | prompt-submit-guard |
| PreToolUse:Write | doc-location-guard, pre-write-validator, suppression-blocker |
| PreToolUse:Edit | pre-write-validator, suppression-blocker |
| PostToolUse:Edit\|Write | change-doc-tracker, post-edit-checker, async-test-runner |
| SubagentStart | subagent-quality-gate (start) |
| SubagentStop | subagent-quality-gate (stop) |
| TaskCompleted | task-completion-verifier |
| PreCompact | pre-compact-snapshot |
| Stop | quality-gate, stop-reconcile, spec-verifier, complexity-ratchet, security-pipeline, test-maturity |
| SessionEnd | session-cleanup |

## Test-First Development (TDD/BDD)

### TDD Mandate
- For NEW modules: test file MUST exist before implementation file
- For BUG FIXES: failing test MUST be written before the fix
- For REFACTORS: existing tests must pass before AND after

### BDD Requirements
- Feature files (*.feature / *.bdd) map to user stories in PRD
- Given/When/Then steps must be traceable to FRs
- BDD test names reference FR IDs: "Feature: FR-AUTH-001 User Login"

### Test Type Requirements (by project maturity)

| Test Type | New Project | Established | Critical System |
|-----------|-------------|-------------|-----------------|
| Unit | Required | Required | Required |
| Integration | Required | Required | Required |
| E2E | Optional | Required | Required |
| Property-based | Optional | Optional | Required |
| Contract | Optional | Required (if APIs) | Required |
| Mutation | Optional | Optional | Required |
| Security (SAST) | Required | Required | Required |
| Accessibility | Optional | Required (if UI) | Required |
| Performance | Optional | Optional | Required |
| Snapshot/Golden | Optional | Optional (if UI) | Required |

### Smart Contract Pattern (Spec Verification)
Specs (PRD/FR) -> Tests (must reference FR IDs) -> Checks (must be green) = Verified
- Every FR-XXX-NNN in FUNCTIONAL_REQUIREMENTS.md MUST have >=1 test referencing it
- Every test MUST reference >=1 FR-XXX-NNN (no orphan tests)
- All linters + type checkers + security scanners MUST pass (0 errors)
- **Coverage MUST meet threshold**:
  - **Agent-Only Projects**: **100%** (E2E, Integration, Unit)
  - **Legacy Projects**: 80% (default)
- If ALL checks green AND ALL FRs have tests -> spec is "programmatically verified"

**Agent-Only Requirement**: Every CLI command MUST have E2E tests. Every workflow MUST have integration tests. Every function MUST have unit tests. See `docs/governance/AGENT_ONLY_TEST_STRATEGY.md` and `docs/governance/TDD_BDD_SDD_GOVERNANCE.md`.

### Architecture Enforcement
- Python: import-linter config enforces layer boundaries
- Go: depguard in golangci.yml enforces package dependency rules
- TypeScript: eslint-plugin-boundaries enforces module boundaries
- When scaffolding: always add architecture enforcement config

### Universal Language Support

The QA system supports 25+ language stacks. See `~/.claude/qa-config.json` for the full list.
Stack detection is automatic via marker files (package.json, go.mod, Cargo.toml, etc.).
Quality templates for all supported languages are in `~/.claude/templates/quality/`.

## Subagent Quality Enforcement

Subagents and tasks are NOT exempt from quality gates. The following hooks fire on subagent lifecycle:
- SubagentStart: tracks subagent scope and expected file changes
- SubagentStop: runs lint/syntax/security on all files the subagent modified
- TaskCompleted: verifies task output meets quality standards (test files, lint, syntax)

## Specification Verification ("Smart Contract")

The spec-verifier runs on Stop and produces a verification verdict:
- VERIFIED: all FRs have tests, all checks green, coverage met
- GAPS: lists uncovered FRs, orphan tests, failing checks
This is the "if green, it works" guarantee — programmatic proof that specs are implemented.

## Complexity Ratchet

Complexity must never increase. The ratchet enforcer:
- Measures cyclomatic complexity, cognitive complexity, maintainability index
- Compares against baseline — any increase is flagged
- Baseline auto-updates downward (tighter over time)
- Max function: 40 lines. Max cyclomatic: 10. Max cognitive: 15.

## Security Pipeline

4-layer security scanning on every Stop:
1. Secret detection (gitleaks + regex patterns)
2. SAST (Semgrep, bandit, gosec, brakeman, psalm)
3. Dependency audit (pip-audit, npm audit, govulncheck, cargo audit)
4. Infrastructure (tfsec, hadolint, trivy)

## Test Maturity Model

Projects are assessed on a 5-level scale:
- Level 1 — MVP: tests exist and are runnable
- Level 2 — Production-Ready: coverage >= 60%, integration tests, no bare suppressions
- Level 3 — Scale: coverage >= 80%, FR traceability >= 50%, security scanning, strict linters
- Level 4 — High-Reliability: FR traceability >= 80%, architecture enforcement, complexity ratchet
- Level 5 — Mission-Critical: 100% FR traceability, mutation testing, chaos tests, runtime verification

**Target**: Level 3 for all projects, Level 4+ for critical systems.

**Agent-Only Projects** (thegent and similar): **Level 5 REQUIRED**
- **100% E2E coverage** (all CLI commands)
- **100% Integration coverage** (all workflows)
- **100% Unit coverage** (all functions)
- **100% FR traceability** (all requirements have tests)
- **Mutation testing** (80%+ mutation score)
- **BDD scenarios** (Gherkin-style for all user journeys)
- **SDD alignment** (tests validate SDD requirements)

See `docs/governance/AGENT_ONLY_TEST_STRATEGY.md` for complete requirements.

## Agent-Only Test Coverage Requirements

**CRITICAL**: For agent-only environments (thegent and similar) where **NO humans will test the system**, comprehensive automated test coverage is **REQUIRED**.

### Coverage Targets (Agent-Only)
- **E2E Tests**: **100%** of all CLI commands (agents interact at CLI boundary)
- **Integration Tests**: **100%** of all workflows (cross-component behavior)
- **Unit Tests**: **100%** of all functions (isolated behavior)

### Why 100%?
In agent-only environments:
- ❌ NO humans will manually test commands
- ❌ NO manual verification possible
- ✅ **ONLY automated tests can verify behavior**
- ✅ **100% coverage is REQUIRED, not optional**

### Test Strategy
- **BDD-Style**: Use Gherkin scenarios for agent journeys
- **TDD Mandate**: Write tests BEFORE implementation
- **SDD Alignment**: Tests validate SDD requirements

### Coverage Analysis
Run coverage analysis:
```bash
python scripts/analyze_test_coverage.py
```

### Documentation
- `docs/governance/AGENT_ONLY_TEST_STRATEGY.md` - Complete test strategy
- `docs/governance/TDD_BDD_SDD_GOVERNANCE.md` - TDD/BDD/SDD alignment
- `docs/governance/TEST_COVERAGE_CRITICAL_GAP.md` - Current coverage gaps

### Current Status (thegent)
- **E2E Coverage**: 21.21% (63/297 commands)
- **Gap**: 234 commands need E2E tests
- **Target**: 100% coverage required

**Agent-Only Projects**: **Level 5 REQUIRED** (100% coverage, mutation testing, BDD scenarios, SDD alignment)

## Runtime Verification

For projects that opt in (via qa-config.json `runtime_verification`):
- Python: beartype (O(1) type checking at runtime), deal (Design by Contract)
- Go: goleak (goroutine leak detection), race detector
- Resilience: toxiproxy (network fault injection), chaos-toolkit (experiments)
Templates available in `~/.claude/templates/quality/runtime/`.

## QA Governance v3.1 — Deep Enforcement Enhancements

### Cognitive Complexity Enforcement
The complexity-ratchet hook now measures both cyclomatic AND cognitive complexity:
- Cognitive complexity weights branching by nesting depth (branch at nesting level N = score 1+N)
- Max cognitive complexity per function: 15 (configurable in qa-config.json)
- Code duplication detection via jscpd (max 5% duplication)
- Dead code detection via vulture (Python) and knip (JS/TS)

### AI Slop Detection
The post-edit-checker now scans every Write/Edit for AI-generated antipatterns:
- Placeholder TODOs ("TODO: implement", "TODO: add")
- Lorem ipsum filler text
- Placeholder domains (example.com in non-test files)
- LLM leakage ("As an AI", "I cannot", "I apologize")
- Lazy AI comments ("This function does...", "This is a helper...")
- Placeholder bodies (pass # TODO, throw new Error("not implemented"))
Advisory only — prints warnings, does not block.

### Dead Import & Dead Code Detection
Quality-gate.sh (Stop) and post-edit-checker.sh (PostToolUse) now detect:
- Dead imports: ruff F401 (Python), oxlint no-unused-vars (JS/TS)
- Dead code: vulture --min-confidence 80 (Python), knip --no-progress (JS/TS)
- Code duplication: jscpd with 5% threshold

### Supply Chain Security (Layer 5)
Security pipeline expanded from 4 to 5 layers:
1. Secrets (gitleaks + regex patterns)
2. SAST (semgrep, bandit, gosec)
3. Dependencies (pip-audit, npm audit, govulncheck, cargo-audit, osv-scanner)
4. Infrastructure (hadolint, tfsec, trivy)
5. **Supply Chain** (syft SBOM generation, OSV-Scanner, opengrep)

### Enhanced Test Maturity Model
Test maturity expanded from 16 to 20 criteria across 5 levels:
- **L4 new**: Snapshot/golden tests (3pts), Approval tests (2pts)
- **L5 new**: Chaos/resilience tests (3pts), Fuzz testing (3pts)
- Enhanced property-based test detection: hypothesis, fast-check, gopter, proptest
- Points rebalanced: 20pts per level, 100pts total

### Hook Stderr Convention
All hooks that exit non-zero now write descriptive failure messages to stderr.
Format: `HOOK_NAME FAIL: reason` (e.g., "SUPPRESSION BLOCKER FAIL: 2 new lint suppression(s)")
This ensures Claude Code displays the actual failure reason instead of "No stderr output".

---

## Development Philosophy

### Proactive Agent Mandate
- **NEVER** ask the user to run a command, search for code, or perform an edit that you have the tools to perform yourself.
- If a task is clear, execute it. If a dependency is missing and you can install/fix it, do so.
- Only ask for clarification if the requirements are truly ambiguous or require a strategic decision that only the user can make.
- "Proactive execution" is the default state. Assume you have permission to use all available tools to achieve the goal.

### Extend, Never Duplicate
- NEVER create a v2 file. Refactor the original.
- NEVER create a new class if an existing one can be made generic.
- NEVER create custom implementations when an OSS library exists.
- Before writing ANY new code: search the codebase for existing patterns.

### Primitives First
- Build generic building blocks before application logic.
- A provider interface + registry is better than N isolated classes.
- Template strings > hardcoded messages. Config-driven > code-driven.

### Research Before Implementing
- Check project deps (pyproject.toml) for existing libraries.
- Search PyPI before writing custom code.
- For non-trivial algorithms: check GitHub for 80%+ implementations to fork/adapt.

### Library Preferences (DO NOT REINVENT)
| Need | Use | NOT |
|------|-----|-----|
| Retry/resilience | tenacity | Custom retry loops |
| HTTP client | httpx | Custom wrappers |
| Logging | structlog | print() or logging.getLogger |
| Config | pydantic-settings | Manual env parsing |
| CLI | typer | argparse |
| Validation | pydantic | Manual if/else |
| Rate limiting | tenacity + asyncio.Semaphore | Custom rate limiter class |

### Code Quality Non-Negotiables
- Zero new lint suppressions without inline justification
- All new code must pass: ruff check, type checker, tests
- Max function: 40 lines. Max cognitive complexity: 15.
- No placeholder TODOs in committed code

### thegent-Specific Rules
- Use tach.toml for boundary enforcement (already configured)
- All new agents must use the agent runner strategy pattern
- **Rust tooling**: Prefer `rg` over `grep`, `fd` over `find`, `jaq` over `jq` for faster hook/agent execution. Hooks use grep-wrapper (routes to rg), fd-wrapper, and JQ_CMD (jaq first). For Claude Code: `export USE_BUILTIN_RIPGREP=0` to use system ripgrep (5-10x faster than bundled).
- All new hooks must follow existing hook patterns in hooks/
- Provider pattern: use ProviderRegistry for extensible services
- MCP tools go through the standard FastMCP registration

---

## Domain-Specific Patterns

### What thegent Is

thegent is an **MCP server + agent hook system** for governing AI agent lifecycle and quality. The core domain is: define agents (personas with capabilities), dispatch hooks at lifecycle events (session start, tool use, stop), enforce governance policies (cost, quality, security), and expose MCP tools for agent management. It is fundamentally an **agent orchestration and governance platform**.

### Local Development (Present)

**Dev stack**: MCP server + CLIProxyAPIPlus proxy via process-compose. Taskfile drives setup and dev.

| Task | Purpose |
|------|---------|
| `task setup` | Install deps, build cliproxy plusplus source, ensure config, install shims |
| `task dev` | Build cliproxy, ensure config, start MCP + proxy (TUI) |
| `task dev:bg` | Same as dev, background |
| `task dev:down` | Stop all services |
| `task dev:logs` | Follow service logs |
| `task cliproxy:build` | Build `../cliproxyapi-plusplus/cli-proxy-api-plus` |
| `task cliproxy:ensure-config` | Ensure cliproxy config (port, auth-dir) |
| `task cliproxy:start`, `stop`, `restart` | Proxy lifecycle |

**Proxy binary**: `scripts/start_proxy_dev.sh` uses the plusplus binary when built (`task cliproxy:build`), else falls back to `cli-proxy-api-plus` from PATH. process-compose runs this wrapper for the proxy process.

**Ports**: MCP 3847, proxy 8317. Canonical source at `../cliproxyapi-plusplus`; metrics at `GET /v1/metrics/providers`.

**Debug**: `thegent run --debug` / `thegent bg --debug` sets `THGENT_DEBUG=1`; proxy gets `-debug` when env set. See `docs/plans/DEBUG_TAGS_AND_METRICS.md`.

### Key Ports and Interfaces

| Port | Responsibility | Location |
|------|---------------|----------|
| **AgentRunner** | Strategy pattern for executing agent personas | `agents/` |
| **HookDispatcher** | Dispatches lifecycle hooks (pre/post tool use, stop, etc.) | `hooks/hook-dispatcher/`, `hooks/*-dispatcher.sh` |
| **PolicyEngine** | Evaluates governance rules (cost caps, quality gates, security) | `hooks/qa-policy-engine.sh`, `contracts/` |
| **MCPToolRegistry** | Registers and serves MCP tools to connected clients | MCP server entry point |
| **CommandRegistry** | CLI commands for agent management, DAG compilation, spec ops | `commands/` |
| **ContractStore** | Stores and validates governance contracts and policies | `contracts/` |

### Provider Registry and Agent Strategy

- **Agent personas** live in `agents/` as markdown definitions. New agents = new `.md` file describing the persona, capabilities, and constraints.
- **Hooks** follow a strict naming and dispatch pattern. The dispatcher routes events to matching hook scripts. New hooks = new `.sh` file in `hooks/` following the naming convention (`qa-*.sh` for quality gates, `pre-*.sh` for pre-tool hooks, etc.).
- **Commands** in `commands/` define CLI-accessible operations (DAG compilation, ledger init, spec hashing). New commands = new entry in `commands/` + registration.
- **Contracts** define governance policies (cost limits, SLOs, migration rules). New governance rule = new contract JSON in `contracts/`.

### Common Anti-Patterns to Avoid

- **Direct MCP message handling in domain logic** -- MCP protocol concerns stay in the MCP server layer. Domain logic (agents, hooks, policies) must not import or depend on MCP transport
- **Custom agent discovery** -- Use the agent registry pattern. Never glob for agent files at runtime outside the registry
- **Hooks that bypass the dispatcher** -- All hooks fire through `hook-dispatcher/`. Never call hook scripts directly from application code
- **Inline governance rules** -- Cost caps, quality thresholds, and policy rules belong in `contracts/` or `hooks/hook-config.yaml`, not hardcoded in hook scripts
- **Monolithic hook scripts** -- Shared logic goes in `hooks/lib/`. Hook scripts should be thin dispatchers that call library functions

### Sitback Agent

`thegent sitback` launches Claude Code with a Sitback Agent persona: dashboard (cockpit + terminals + ps), FastMCP tools first, CLI fallback. Skills: `skills/sitback-agent/` (default), overridable via `--skill`. MCP precondition: `thegent serve` for full toolset.

### Workflow Triggers (Skill / MCP / Instruction)

Idea/task prompts, quality green, and "next thing to do" are wired at multiple levels:

| Level | Location | Purpose |
|-------|----------|---------|
| **Hook** | `hooks/prompt-submit-guard.sh` | UserPromptSubmit: pattern-detect, inject instructions to agent context |
| **Skill** | `skills/agent-orchestra/SKILL.md`, `skills/sitback-agent/SKILL.md` | Baked-in workflow section; agents with these skills follow it |
| **MCP resource** | `thegent://workflow/triggers` | URI-addressable; agent can read when needed |
| **MCP resource** | `thegent://workstream` | Work stream (canonical backlog) |
| **MCP prompts** | `thegent_workflow_idea`, `thegent_workflow_quality_green`, `thegent_workflow_next_item`, `thegent_workflow_gardening` | Template prompts for structured invocation |
| **MCP resource** | `thegent://workflow/gardening` | Gardening workflow (converge to empty backlog + green) |
| **MCP tool** | `thegent_do_next` | Find next actionable items from WORK_STREAM (canonical), PLAN_STATUS, FR_TRACKER, docs/plans/, escalation; returns prompt_suggestion for thegent_run/thegent_bg |
| **CLI** | `thegent plan do-next` | Same as thegent_do_next |

**Unified work stream**: Single source of truth is `docs/reference/WORK_STREAM.md`. All agents read it for work items; claim in CLAIMED before starting; update COMPLETED when done. Incorporator agent (`work-stream-incorporator`) merges fragments from plans, research, specs into the stream. See [UNIFIED_WORK_STREAM_DESIGN.md](docs/reference/UNIFIED_WORK_STREAM_DESIGN.md).

**Idea/task** → dump research to docs/research/, specs to docs/docset/, work items to unified stream. **Quality green** → `task quality`. **Next item** → `thegent_do_next` (or read WORK_STREAM.md), pick highest-priority, execute via `thegent_run`/`thegent_bg` with `prompt_suggestion`. **Gardening** → check gov traceability, tests, plan items; dispatch; converge to empty backlog and complete green (`thegent govern go health`, `go cycle`, `task quality`).

### Lifecycle Loops

| Command / Tool | Purpose |
|----------------|---------|
| `thegent orchestrate loop "prompt" "todo"` | Run Lifecycle loop (worker + checker) |
| `thegent orchestrate loop-send \<session_id\> \<prompt\>` | Send next prompt to running loop (human/agent takeover) |
| `thegent orchestrate loop-stop \<session_id\>` | Stop loop |
| `thegent takeover \<session\>` | Attach to tmux session; human types next prompt |
| `thegent_loop_takeover` (MCP) | Agent injects prompt into running loop |
| `--continuation \<session_id\>` | Resume from prior session (adds resumption appendix) |
| `--resume` (Codex/Claude) | Use when agent supports native resume |

**Premature session end:** If Codex/Claude supports `--resume`, use it. Otherwise: `thegent run/bg --continuation \<prior_session_id\> "Task"` — builds context from prior stdout + resumption appendix.

### WBS Agent Coordination (Multi-Agent "Do All")

When the user says **"do all"** or assigns work to multiple agents:

1. **Read** `docs/reference/WORK_STREAM.md` (canonical) — or `docs/plans/02-UNIFIED-WBS.md` + `docs/reference/WBS_AGENT_PROGRESS.md` for WBS-only coordination
2. **Claim before starting**: Append your work items to the **CLAIMED** table in `WORK_STREAM.md` (or `WBS_AGENT_PROGRESS.md` if using WBS-only) with a unique agent_id (e.g. `agent-1`, `runner-A`)
3. **Avoid overlap**: Do NOT pick items already in CLAIMED. Pick an equal batch of unclaimed items.
4. **Update progress**: When done, move items from CLAIMED to COMPLETED and update source file (e.g. `02-UNIFIED-WBS.md`) status to DONE

**Preferred**: Use `WORK_STREAM.md` — single file for all work types. `WBS_AGENT_PROGRESS.md` remains for backward compatibility with WBS-only "do all" flows.

### Where to Add New Functionality

| Want to add... | Put it in... |
|----------------|-------------|
| New agent persona | `agents/<persona-name>.md` -- follows existing agent template |
| New lifecycle hook | `hooks/\<event\>-\<name\>.sh` + register in `hooks/hook-config.yaml` |
| New governance policy | `contracts/\<policy\>.json` + wire into `qa-policy-engine.sh` |
| New MCP tool | MCP server registration (FastMCP pattern) |
| New CLI command | `commands/\<command\>/` + register in command dispatch |
| New quality gate | `hooks/qa-<gate-name>.sh` following existing `qa-*.sh` patterns |
| Shared hook utility | `hooks/lib/\<utility\>.sh` -- sourced by hook scripts, never called directly |


---

## Source: upstream-governance/thegent/CLAUDE.md

# Global Claude Code Instructions

These rules apply to ALL projects. Project-level CLAUDE.md files supplement (and may override) these.

---

# Instruction Architecture (Global vs Project)

This file is the **global instruction index**. Keep it readable, stable, and link-first.

## Layering and Precedence

1. **System + developer prompts** (runtime/platform enforced)
2. **Global CLAUDE** (`CLAUDE.md` + linked reference docs)
3. **Project CLAUDE** (repo-local overrides and specifics)
4. **Task artifacts** (`docs/plans/`, `docs/research/`, `docs/reports/`, `docs/reference/WORK_STREAM.md`)

When layers conflict, higher precedence wins. Project docs should extend global policy, not duplicate it.

## Canonical Roles

| Artifact | Role | Content Rules |
|----------|------|---------------|
| `CLAUDE.md` | Global index and critical guardrails | Keep concise; route detail to doc map |
| `docs/reference/CLAUDE_CORE_GUIDELINES.md` | Full global baseline | Long-form policy source |
| `docs/reference/CLAUDE_THEGENT_RUNTIME_APPENDIX.md` | Runtime and project operations | thegent-specific execution details |
| Project-local `CLAUDE.md` | Project overlays | Domain/runtime overrides, local commands |

## Instruction Doc Map

- Global baseline: `docs/reference/CLAUDE_CORE_GUIDELINES.md`
- Runtime appendix: `docs/reference/CLAUDE_THEGENT_RUNTIME_APPENDIX.md`
- Governance summary: `docs/governance/GOVERNANCE_SUMMARY.md`
- Polyglot runtime policy: `docs/governance/POLYGLOT_RUNTIME_COVERAGE_AND_CONVERSION_MATRIX_2026-02-21.md`
- Active execution ledger: `docs/reference/WORK_STREAM.md`
- Current upgrade worklog: `docs/reports/2026-02-21-CLAUDE-INSTRUCTION-ARCHITECTURE-UPGRADE-WORKLOG.md`

---

# 🔒 CRITICAL SECURITY RULES - NEVER VIOLATE

## ⛔ FORBIDDEN: Killing Agent or Terminal Processes

**ABSOLUTELY FORBIDDEN** - Agents MUST NEVER kill other agent processes or terminal processes.

### ❌ NEVER RUN THESE COMMANDS:
```bash
# FORBIDDEN - Killing cursor-agent (EXACT PATTERN YOU MUST NOT USE)
ps -ao pid,command | grep "cursor-agent" | grep -v grep | grep -v 40690 | awk '{print $1}' | xargs kill -9
ps | grep cursor-agent | xargs kill -9
pkill cursor-agent
killall cursor-agent

# FORBIDDEN - Killing any agent process
kill -9 <pid>  # where PID is cursor-agent, thegent, claude, codex, droid, etc.

# FORBIDDEN - Killing shell/terminal processes  
kill -9 <pid>  # where PID is bash, zsh, sh, ghostty, terminal, iterm, etc.
```

### ✅ CORRECT ALTERNATIVES:
```bash
# Safe cleanup of orphaned LSP/MCP processes
thegent mcp prune

# See what would be pruned (dry run)
thegent mcp prune --dry-run

# List active sessions
thegent ps

# Properly stop a session
thegent stop <session_id>
```

### 🛡️ PROTECTED PROCESSES:
The following processes are PROTECTED and MUST NEVER be killed:
- **Agent processes**: `cursor-agent`, `thegent`, `claude`, `codex`, `droid`, `opencode`, `copilot`, `gemini`
- **Shell processes**: `bash`, `zsh`, `sh`, `fish`, `tcsh`, `csh`
- **Terminal emulators**: `ghostty`, `terminal`, `iterm`, `alacritty`, `kitty`, `wezterm`, `warp`

### ⚠️ SECURITY ENFORCEMENT:
- All commands are validated before execution
- Commands attempting to kill protected processes will be **BLOCKED**
- Violations are logged and reported
- Rate limiting prevents abuse

**If you need to clean up processes, use the safe pruning tools provided by thegent, NOT manual kill commands.**

---

## ⛔ FORBIDDEN: Fallbacks, Legacy Compatibility, and Silent Failures

**ABSOLUTELY FORBIDDEN** - Agents MUST NEVER add fallbacks, legacy compatibility, or silent error handling.

### ❌ NEVER ADD:
- **Fallback code paths**: `try: new(); except: old()` or `try: fast(); except: slow()`
- **Legacy compatibility shims**: `if legacy_flag: old(); else: new()`
- **Backwards compatibility layers**: `def old(): warnings.warn(); return new()`
- **Silent error handling**: `try: thing(); except: pass` or `try: thing(); except: return default`
- **Error hiding**: `try: thing(); except: delete_from_db()` (hiding bugs)
- **"Just in case" code**: Code added "just in case" something fails
- **Import fallbacks**: `try: from X import Y; except: from Z import Y`
- **Migration systems for simple changes**: Don't create versioning/migration for simple edits

### ✅ CORRECT APPROACH:
- **Code should FAIL and STOP** on errors - fail fast, fail loudly
- **No fallbacks** unless explicitly requested (and even then, prefer fixing the root cause)
- **No legacy compatibility** - Zero user debt = zero backwards compatibility
- **No silent failures** - All errors must be visible and logged
- **Fix bugs, don't hide them** - If something fails, fix it, don't work around it
- **Verify parity BEFORE removal** - Always verify feature parity and migration completeness before removing code

### 🎯 "Aim Towards" Framing:
When removing code, frame it positively:
```
BAD: "Don't add fallbacks"
GOOD: "Now that we have fully transitioned to a new system and it has been 
confirmed to work as intended, let's clean out all backwards compatibility 
and fallbacks so we have a DRY, modular system with clear and clean separation 
of responsibilities. Once finished, we have a fresh system with no technical debt."
```

### ⚠️ AI AGENT PATTERN:
AI coding agents (Claude, Codex, ChatGPT) have a **systemic tendency** to add fallbacks and legacy compatibility even when explicitly told not to. This is a known pattern requiring:
- **Explicit rules** (like this section)
- **"Aim towards" framing** (positive direction, not just "don't do X")
- **Fail fast philosophy** (code should fail and stop)
- **Parity verification** (verify before removal)
- **CI checks** (automated detection of fallback patterns)

### 🛡️ ENFORCEMENT:
- All code is checked for fallback patterns
- Commits with fallbacks will be flagged
- Silent error handling is detected and blocked
- Legacy compatibility code is identified and removed

**Remember: Zero user debt = zero backwards compatibility. All changes are breaking changes by design. Code should fail fast and fail loudly, not silently work around problems.**

---

# Terminology (Layer Vocabulary)

**For ease of communication.** See also: `docs/governance/TERMINOLOGY_LAYERS.md`

| Term | Definition | Examples |
|------|------------|----------|
| **Harness** | The agent layer. May or may not come with a CLI, API, or other interface. | Codex CLI, Claude Code CLI, Claude Agent SDK, Factory Droid |
| **LLM** | The model (as known). | GPT-5, Claude, Gemini, etc. |
| **Presentation layer** | The UI layer of a harness. | Terminal UI, IDE panel, web UI |
| **Various layers** | Layers between and around (routing, proxy, auth, orchestration). | CLIProxy, LiteLLM Router, thegent orchestration |

---

# Heavy Web Research Policy
- Use DuckDuckGo (`thegent_ddg_search`) for comprehensive web research when local knowledge is insufficient.
- **Deep Research Protocol**: For multi-source or blocked sites (Reddit, Google), use `docs/guides/DEEP_RESEARCH_PROTOCOL.md`.
- **Resilience**: Use `thegent_reddit_search` for Reddit; `thegent_scrape_url` (Playwright-backed) to bypass site blocks.
- **Protocol Tools**: Prefer `thegent_deep_research` orchestrator for complex investigations.
- Summarize findings for the user, providing links only for deep dives.

---

# Library-First Policy

**CRITICAL**: Prefer **library + thin wrapper** over full custom implementation. Apply from the start of development and throughout.

- **First question**: "Is there a library that solves this?"
- **Generic problems** (retry, cache, file watch, circuit breaker, rate limit): Use a library. Keep wrapper \< 50 LOC.
- **Custom logic**: Only for domain-specific behavior. **ADR required** if choosing custom over library.

| Need | Library | Notes |
|------|---------|-------|
| Retry/backoff | tenacity | No manual retry loops; use `wait_random_exponential` |
| HTTP | httpx | No requests/urllib |
| File watching | watchdog | No os.walk polling |
| Caching | cachetools / diskcache | No custom TTL logic |
| Circuit breaker | pybreaker | Or tenacity + custom state |
| Logging | structlog | Structured, JSON for aggregation |

See: `docs/research/LIBRARY_FIRST_AUDIT_AND_PLAN.md`, `docs/guides/anti-patterns.md`

---

# Proactive Governance Evolution

**Do not wait for the user to ask.** When work touches a governance domain (retry, cache, file watch, HTTP, auth, logging):

1. **Check** existing governance (anti-patterns.md, LIBRARY_FIRST_AUDIT_AND_PLAN.md, CLAUDE.md).
2. **Follow** it. If governance is missing or outdated, **propose or add** an update in the same task.
3. **At task completion**: Run a governance checkpoint. Update if you touched a governed domain and governance is incomplete.

See: `docs/research/PROACTIVE_GOVERNANCE_EVOLUTION_PLAN.md`

---

# Conversation Dumps (Always Write)

**CRITICAL**: After any conversation producing research, plans, decisions, or implementation details:

1. **Write a dump** to `docs/research/CONVERSATION_DUMP_YYYY-MM-DD.md` (or append to existing same-day file).
2. **Include:** Research findings, plans, decisions, fixes applied, open questions, next steps.
3. **Do not defer:** Write the dump as part of the same response/task. Do not say "I'll add it later."

**Format:** Dated filename. Sections: Issues Addressed, Fixes Applied, Research Findings, Plans, Open Questions, Cursor-Agent Recovery Note (if applicable).

**Tooling:** `thegent prompts sessions` to list sessions; `thegent prompts dump \<session_id\>` to dump to docs/research/.

---

# Context Management Strategy

## The Manager Pattern

**CRITICAL**: Operate as a strategic manager, not a worker. Delegate to subagents.

**Delegate** when: >3 files, codebase-wide search, >2000 tokens of output, multi-step sequences, independent parallel work.
**Handle directly** when: single file, quick answer, <3 files, config/tweak.

## Strategy Quick Reference

| Need | Tool/Provider |
|------|---------------|
| Heavy Web Research | DuckDuckGo (`ddgr`) |
| Find code patterns | `Explore` agent |
| Design approach | `Plan` agent |
| Multi-step implementation | `thegent free` or `thegent bg` |
| Work stream integration | `thegent free --do-next` |
| Continuous autonomous work | `thegent plan loop` |
| Background execution | `thegent bg` |
| Idle waiting | `thegent plan wait-next` |

## Anti-Patterns

| Bad | Good |
|-----|------|
| Reading 10 files to "understand" | Delegate exploration, get summary |
| `ls -l` in project root (node_modules) | `fd -t f -d 1` or `ls -l src/` |
| Editing files for multi-file changes | Delegate to `general-purpose` |
| Sequential explorations one-by-one | Batch parallel explores |
| `git restore .` or `git clean` to "reset" | Leave modified files; assume active agent work |
| Overwriting a "dirty" file | Merge or work around existing changes |
| Custom retry/cache/watch logic | Library-first: tenacity, cachetools, watchdog |

## Idle / Session Continuity

**CRITICAL**: When idle, ALWAYS check backlog with `thegent plan do-next` and work on items DIRECTLY. Never terminate the session while work exists.

```bash
thegent plan wait-next --timeout 0 --poll 10  # Block until work arrives
thegent plan loop --max 1000 --sleep 30        # Continuous work loop
thegent wait <session_id> --timeout 300        # Wait for specific agent
```

## DX/UX Friction

The `friction-detector.sh` hook runs automatically. **Act on its alerts.**
- `cd &&` → CLI should work from any directory
- `head -n` → CLI should have `--limit`
- Bash loops → CLI should have `--repeat N`
- Multiple sequential `read_file()` → use `batch_read_files()`
- Manual path resolution → use `normalize_path()`

**Helpers:** `batch_file_ops.py` (3-5x fewer tool calls), `scripts/path_utils.py` (cross-platform safe paths).

See: `~/.claude/docs/friction-reduction.md` for full helpers, detection patterns, and examples.

---

# Thegent Command Reference

> Full reference with all options, patterns, and providers: `~/.claude/docs/thegent-commands.md`

## Command Selection

| Task | Command |
|------|---------|
| Default agent task | `thegent free "Task"` |
| Work stream next item | `thegent free --do-next` |
| Run N items sequentially | `thegent free --do-next --repeat N` |
| Background execution | `thegent free "Task" --bg` |
| Continuous loop | `thegent plan loop` |
| Idle wait | `thegent plan wait-next` |
| Model-specific | `thegent run "Task" -M claude-sonnet-4.5` |
| Cost-optimized | `thegent run "Task" -M gemini-3-flash -R cheapest` |
| Continue prior session | `thegent bg "Task" -C \<session_id\>` |
| Session mgmt | `thegent ps` / `thegent status \<id\>` / `thegent wait \<id\>` |
| Role-based | `thegent research/review/fix/code/explain/summarize "..."` |

## Key Providers

| Provider | Default Model | Notes |
|----------|---------------|-------|
| `free` | `gpt-5-mini` | **Default. Free tier, work stream integration.** |
| `claude` | `claude-haiku-4.5` | Anthropic API |
| `gemini` | `gemini-3-flash` | Google Gemini |
| `codex` | `gpt-5.3-codex` | Codex API |
| `cursor`/`antigravity`/`kiro` | various | Proxy providers |

## Environment Variables

- `THGENT_DEFAULT_TIMEOUT_FREE`: Free agent timeout (default: 300s)
- `THGENT_DEFAULT_ROUTING`: Routing policy (`prefer_direct` | `prefer_proxy`)
- `THGENT_DEBUG`: Debug mode (1=enabled)

## Anti-Patterns

- **Don't** use busy loops → use `plan wait-next` or `wait \<id\>`
- **Don't** use bash wrappers for loops → use native `--repeat`, `--do-next`, `plan loop`
- **Don't** hardcode agents → use `free` as default, override when needed
- **Don't** `ls -l` in project root → use `fd` or subdirectories

---

# Documentation Organization

**CRITICAL**: All project documentation follows strict organization.

### Root-Level Files (Keep in Root)
`README.md`, `CHANGELOG.md`, `AGENTS.md`, `CLAUDE.md`, `00_START_HERE.md`, `PRD.md`, `ADR.md`, `FUNCTIONAL_REQUIREMENTS.md`, `PLAN.md`, `USER_JOURNEYS.md`

### docs/ Structure

```
docs/
  guides/           # Implementation guides; quick-start/
  reports/          # Completion reports, summaries, status reports
  research/         # Research summaries, CONVERSATION_DUMP_*.md, analysis
  reference/        # Quick references, trackers, maps
  checklists/       # Implementation checklists
  changes/          # Per-change docs; archive/ for completed
```

**NEVER** create `.md` files in the project root (except allowed root-level files above).
**ALWAYS** place new documentation in the appropriate `docs/` subdirectory.

---

# Optionality and Failure Behavior

**Require** dependencies where they belong; **require** clear, loud failures — no silent or "graceful" degradation.

- **Force requirement where it belongs.** Don't make dependencies "optional" just to avoid failure.
- **Fail clearly, not silently.** Explicit failures — not reduced functionality, logging-only warnings, or hidden errors.
- **Graceful in other ways.** Retries with visible feedback; actionable messages; non-obscure stack traces.

---

# Planner Agents: No Code in Docs or Plans

**Planner agents** (PM, Analyst, Architect) must **never write code** in documentation or plans. Write specs, acceptance criteria, architecture decisions, and clear handoffs. Brief pseudocode only if necessary.

---

# Phased WBS and Plans with DAGs

Plans must have:
- **Phases**: Discovery → Design → Build → Test/Validate → Deploy/Handoff
- **DAG**: Tasks with explicit predecessors; no cycles
- **Output**: Phase | Task ID | Description | Depends On table

---

# Timescales: Agent-Led, Aggressive Estimates

**Assume agent-driven environment.** No human intervention beyond prompts.

- **Forbidden in plans:** "Schedule audit", "Stakeholder Presentation", "Human checkpoint", "Get approval from X"
- **Use instead:** "N tool calls", "N parallel subagents", "~M min wall clock"
- Trivial: 1-2 calls, <1 min | Small: 3-6 calls, 1-3 min | Cross-stack: 2-3 subagents, 3-8 min | Major: 3-5 subagents, 8-20 min

---

# Generalized Dev Environment Pattern

## Service Management

- **Never** start/stop/restart entire dev stack (`make dev`, `make dev-tui`). Only the user does that.
- Use CLI introspection and per-service manipulation. Services use hot reload — save files and let watchers pick up changes.
- Restart only the specific service that needs it, never the whole stack.
- Read logs via CLI or log files — never attach to or interfere with the user's TUI terminal.

## Package Manager

Detect from lockfiles: `bun.lockb`/`bun.lock` → bun | `pnpm-lock.yaml` → pnpm | `yarn.lock` → yarn | `package-lock.json` → npm. Check `package.json` `packageManager` field if unclear.

## Native Over Docker / OSS First

Prefer native services over Docker for local dev. Prefer local, OSS, and free tools over paid SaaS.

## Multi-Actor Coordination

- **FORBIDDEN**: `git restore`, `git reset`, `git clean` — destroys other agents' work-in-progress.
- **Respect Dirty Files**: Modified files are active work. Never revert/overwrite without explicit instruction.
- Use project-provided wrappers (Makefile targets); use process orchestrator as source of truth.

---

# Plugin Ecosystem Awareness

Use AgilePlus for spec tracking: `agileplus \<command\>`. Check `/` for slash commands. Start new conversation to switch agent personas.

---

# Opinionated Quality Enforcement

- Enforce opinionated styling strictly.
- Fix code properly; never add disables/ignores.
- Use project linters, formatters, and type checkers. Never bypass them.

---

# Specification Documentation System

Every non-trivial project SHOULD maintain:

| File | Purpose |
|------|---------|
| `PRD.md` | Product Requirements: epics, user stories, acceptance criteria |
| `ADR.md` | Architecture Decision Records: decisions, rationale, alternatives |
| `FUNCTIONAL_REQUIREMENTS.md` | FR SHALL statements, traces to PRD |
| `PLAN.md` | Phased WBS with DAG dependencies |
| `USER_JOURNEYS.md` | User journeys with ASCII flow diagrams |

Trackers in `docs/reference/`: `PRD_TRACKER.md`, `ADR_STATUS.md`, `FR_TRACKER.md`, `PLAN_STATUS.md`, `JOURNEY_VALIDATION.md`, `CODE_ENTITY_MAP.md`

**On session start:** If spec docs are missing, offer to generate them. Do NOT auto-generate without user confirmation.

## Doc Format Standards

- **IDs:** E{n}.{m}.{k} (epics), FR-{CAT}-{NNN} (requirements), ADR-{NNN} (decisions), P{n}.{m} (tasks), UJ-{N} (journeys)
- **Cross-reference** between docs. **ASCII diagrams** for flows. **Tables** for tracking.

## Global Reference Docs

| Domain | Path |
|--------|------|
| UI Design | `docs/reference/UI_DESIGN_PRINCIPLES_REFERENCE.md` |
| Architecture | `docs/reference/SOFTWARE_ARCHITECTURE_REFERENCE.md` |
| Performance | `docs/reference/performance/PERFORMANCE_OPTIMIZATION.md` |
| Security | `docs/reference/security/SECURITY_BEST_PRACTICES.md` |
| Full Index | `docs/reference/INDEX.md` |

## Project Setup

VitePress docsite: `copier copy thegent/templates/initialize-project ./my-new-project`

See `~/.claude/docs/project-setup-checklist.md` for the full checklist: VitePress docsite, Taskfile, linters (25+ stacks), IDE config, pre-commit hooks, quality gates, test infra, FR traceability setup.

## Change Documentation

For significant changes: create `docs/changes/{change-name}/` with `proposal.md`, `design.md`, `tasks.md`. Archive completed to `docs/changes/archive/`.

---

# QA Governance

## Core Mandates

- **Test-First**: Write tests BEFORE implementation. Failing test MUST exist before bug fix. Test file before source file.
- **Suppressions**: Zero new suppressions without inline justification (`# noqa: E501 -- reason`). `suppression-blocker.sh` BLOCKS violations.
- **Spec Traceability**: All tests MUST reference FR ID via `@pytest.mark.requirement("FR-XXX-NNN")`, `# @trace FR-XXX-NNN`, or test name.
- **Quality Gate**: `task quality` runs full strict pipeline (max-lines, lint, core-boundary, deprecated-aliases, instruction-architecture, harness-contracts, runtime-contracts). Always run `task quality` before stopping work.
- **Static Analysis**: When scaffolding, copy templates from `~/.claude/templates/quality/` for detected stack.

## Coverage Targets

| Project Type | Unit | Integration | E2E |
|-------------|------|-------------|-----|
| Agent-Only (thegent) | **100%** | **100%** | **100%** |
| Legacy (human testers) | 70% | 20% | 10% |

## Hook Pipeline (v3)

| Event | Hooks |
|-------|-------|
| SessionStart | spec-preflight, qa-preflight |
| PreToolUse:Write | doc-location-guard, pre-write-validator, suppression-blocker |
| PreToolUse:Edit | pre-write-validator, suppression-blocker |
| PostToolUse:Edit\|Write | change-doc-tracker, post-edit-checker, async-test-runner |
| Stop | quality-gate, stop-reconcile, spec-verifier, complexity-ratchet, security-pipeline, test-maturity |

## Test Maturity Target

- **All projects**: Level 3+ (&gt;80% coverage, FR traceability &gt;50%, security scanning, strict linters)
- **Agent-Only**: Level 5 (100% E2E/Integration/Unit, mutation testing, BDD, SDD alignment)

## Security Pipeline (5 layers)

1. Secrets (gitleaks) 2. SAST (semgrep, bandit, gosec) 3. Dependencies (pip-audit, npm audit, govulncheck) 4. Infrastructure (hadolint, tfsec, trivy) 5. Supply Chain (syft SBOM, osv-scanner)

## Complexity Limits

Max function: 40 lines. Max cyclomatic: 10. Max cognitive: 15. Max duplication: 5% (jscpd).
Dead code: vulture (Python), knip (JS/TS). AI slop detection on every Write/Edit.

See: `~/.claude/docs/qa-governance-detail.md` for full TDD/BDD mandates, smart contract pattern, architecture enforcement, runtime verification, QA v3.1 enhancements.

---

# Development Philosophy

## Proactive Agent Mandate
- **NEVER** ask the user to run a command, search for code, or perform an edit that you have tools to do yourself.
- Execute if task is clear. Only ask if requirements are truly ambiguous or require a strategic user decision.
- "Proactive execution" is the default state.

## Core Principles

- **Extend, Never Duplicate**: Refactor originals. Never create v2 files. Never create a new class if existing one can be generic.
- **Primitives First**: Build generic building blocks before application logic. Config-driven > code-driven.
- **Research Before Implementing**: Check project deps, search PyPI, check GitHub for 80%+ implementations before writing custom code.

## Library Preferences (DO NOT REINVENT)

| Need | Use | NOT |
|------|-----|-----|
| Retry/resilience | tenacity | Custom retry loops |
| HTTP client | httpx | Custom wrappers |
| Logging | structlog | print() or logging.getLogger |
| Config | pydantic-settings | Manual env parsing |
| CLI | typer | argparse |
| Validation | pydantic | Manual if/else |
| Rate limiting | tenacity + asyncio.Semaphore | Custom rate limiter |

## Code Quality Non-Negotiables

- Zero new lint suppressions without inline justification
- All new code must pass: ruff check, type checker, tests
- Max function: 40 lines. Max cognitive complexity: 15.
- No placeholder TODOs in committed code

## thegent-Specific Rules

- Use tach.toml for boundary enforcement (already configured)
- All new agents use the agent runner strategy pattern
- New hooks: `hooks/\<event\>-\<name\>.sh` + register in `hooks/hook-config.yaml`
- Shared hook logic: `hooks/lib/\<utility\>.sh` (sourced by hooks, never called directly)
- **Rust tooling**: Prefer `rg` over `grep`, `fd` over `find`, `jaq` over `jq`. Export `USE_BUILTIN_RIPGREP=0` for system ripgrep (5-10x faster).
- Provider pattern: use ProviderRegistry for extensible services. MCP tools through FastMCP registration.

---

# Domain-Specific Patterns (thegent)

> Full reference: `~/.claude/docs/thegent-domain-patterns.md`

**What thegent is:** MCP server + agent hook system for governing AI agent lifecycle and quality. Agent orchestration and governance platform.

## Key Architecture

| Component | Responsibility | Location |
|-----------|---------------|----------|
| AgentRunner | Strategy pattern for agent personas | `agents/` |
| HookDispatcher | Lifecycle hooks dispatch | `hooks/hook-dispatcher/` |
| PolicyEngine | Governance rules evaluation | `hooks/qa-policy-engine.sh`, `contracts/` |
| MCPToolRegistry | MCP tools registration | MCP server entry point |
| CommandRegistry | CLI commands | `commands/` |
| ContractStore | Governance contracts and policies | `contracts/` |

## Where to Add Functionality

| Add... | Put in... |
|--------|-----------|
| Agent persona | `agents/\<name\>.md` |
| Lifecycle hook | `hooks/\<event\>-\<name\>.sh` + `hooks/hook-config.yaml` |
| Governance policy | `contracts/\<policy\>.json` + `qa-policy-engine.sh` |
| MCP tool | MCP server (FastMCP pattern) |
| CLI command | `commands/\<command\>/` + register in dispatch |
| Quality gate | `hooks/qa-<gate-name>.sh` |

## Work Stream

Canonical: `docs/reference/WORK_STREAM.md`. Claim before starting → mark COMPLETED when done. `thegent plan incorporate` merges fragments from plans/research/specs.

## Agent Memory (MTSP-17/18)

- `thegent_memory_add`: log discoveries, lessons, friction points
- `thegent_memory_scrape_session`: ingest user prompts/intents
- `thegent_memory_synthesize`: generate summary before finishing major task

## Lifecycle Loops

| Command | Purpose |
|---------|---------|
| `thegent plan loop` | Continuous work loop (RECOMMENDED) |
| `thegent orchestrate loop "prompt" "todo"` | Worker + checker lifecycle loop |
| `thegent bg "Task" -C \<session_id\>` | Continue from prior session |
| `thegent_loop_takeover` (MCP) | Agent injects prompt into running loop |

**Ports:** MCP 3847, proxy 8317. Debug: `thegent run --debug` sets `THGENT_DEBUG=1`.

---

# Polyglot Runtime and Coverage Governance

Canonical matrix: `docs/governance/POLYGLOT_RUNTIME_COVERAGE_AND_CONVERSION_MATRIX_2026-02-21.md`

## Required Baseline

1. Python: `uv` on CPython 3.14 (primary), PyPy 3.11 (secondary), CPython 3.13 (fallback compatibility lane).
2. Rust: stable `fmt + clippy -D warnings + test`.
3. Go: `go test ./...` and `go vet ./...` on supported version lanes.
4. Zig: pinned stable `zig test`.
5. Mojo: pinned version with parity checks against reference implementations.

## Conversion/Refactor Rules

1. Prefer refactor-in-place before full language conversion.
2. Convert only when measured SLO/tooling triggers are met and documented.
3. Every conversion requires baseline metrics, parity harness, and phased cutover plan.

## Frontmatter/Backmatter Defaults

1. Governance/spec docs must use standard frontmatter (`title/date/status/owner/tags`).
2. Decision-heavy docs must include backmatter summary:
- decision delta,
- validation commands,
- residual risks,
- follow-up review date.

## CLAUDE Filename/Size Policy

1. Canonical file is `CLAUDE.md` (uppercase).
2. Typo files like `calude.md` must be merged into canonical `CLAUDE.md` and removed.
3. If `CLAUDE.md` grows beyond ~20k tokens:
- keep `CLAUDE.md` as concise index/policy spine,
- split detailed sections into `docs/docsets/claude/`,
- maintain explicit links from canonical file.

---

# Context Documentation System

thegent maintains **authoritative context docs** for all integrated technologies in `docs/context/`. These are not tutorials or marketing material — they are **precise technical references** for implementing against each technology.

## Key Principles

- **Before integration:** Check if `docs/context/{technology}.md` exists. If not, create it first.
- **During implementation:** Reference the context doc for exact API shapes, auth requirements, and patterns.
- **After research:** Update the relevant context doc with findings; move knowledge from `docs/research/` to `docs/context/` as tech is adopted.
- **Standalone:** An AI agent should understand the tech from just the context doc, without external references.
- **No hallucination:** Every claim is verifiable against official docs.

## Organization

| Location | Purpose |
|----------|---------|
| `docs/context/GOVERNANCE.md` | Standards, format, required sections, automation |
| `docs/context/INDEX.md` | Catalog of all context docs by category and priority |
| `docs/governance/CONTEXT_DOCS_PROCESS.md` | Step-by-step creation, update, and verification process |
| `docs/context/{technology}.md` | Atomic context doc (single technology) |
| `docs/context/{technology}/` | Doc set (multi-part documentation) |

## Required Sections (Every Doc)

All context docs MUST include:

1. **Header** — What is it? Source URL + fetch date
2. **What is {Tech}** — Definition, problem solved, capabilities, thegent integration
3. **Key Concepts** — Domain-specific terminology (if applicable)
4. **API/Interfaces** — Exact endpoint/method specs with request/response shapes
5. **Authentication** — How to authenticate, where to get credentials
6. **Code Examples** — 1-3 working examples for main use cases
7. **Sources & References** — Complete citations with URLs and dates
8. **Quick Reference** — One-page cheat sheet

## Priority Coverage

| Priority | Level | Examples | Status |
|----------|-------|----------|--------|
| **P0** | Critical | Ante, Claude Code, Codex, FastMCP, MCP, OpenRouter | 3/8 exist |
| **P1** | High | WorkOS, AuthKit, Nix, process-compose | 1/8 exist |
| **P2** | Optional | Stripe, PostHog, Grafana | Create on-demand |

Check `docs/context/INDEX.md` for current coverage and roadmap.

## Workflow

### Create a New Context Doc

1. Check if already exists: `ls docs/context/{technology}.md`
2. Gather official sources (docs, GitHub, API responses)
3. Extract technical specs (API, auth, concepts, examples)
4. Write following template in `GOVERNANCE.md`
5. Test all code examples
6. Pass pre-write validation (all required sections present)
7. Add to `docs/context/INDEX.md`

See `docs/governance/CONTEXT_DOCS_PROCESS.md` for full process (2-4 hours).

### Update an Existing Doc

**Minor** (typo, date refresh): Direct edit, no review needed. Commit: `fix: update {tech} context doc - {description}`

**Major** (API change, version bump): Fetch latest docs, update sections, test examples, request peer review. Commit: `update: {tech} context doc for v{version}`

Refresh any doc > 90 days old. Mark stale docs with `⚠️ Possibly stale - last verified YYYY-MM-DD`.

### Verify Before Using

Before referencing a context doc in code:
- Header has recent fetch date (< 6 months)
- No `⚠️ Possibly stale` banner
- Spot-check 3-5 API examples against official docs
- Run at least one code example without modification

## Automation

**Pre-write validation** (on write/edit): Ensure doc has all required sections. Reject incomplete docs.

**Weekly staleness check**: Scan all docs, create issues for docs > 90 days old.

**Version release monitoring**: When technology version released, create issue to update context doc.

## Cross-Reference

Context docs link to each other. Common patterns:
- Harness docs → SDK/protocol docs
- Protocol docs → implementation examples
- Auth docs → harness integration guides

Search within `docs/context/INDEX.md` and linked docs; use Ctrl+F to navigate.


---
