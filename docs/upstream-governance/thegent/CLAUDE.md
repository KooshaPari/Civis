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
- **Generic problems** (retry, cache, file watch, circuit breaker, rate limit): Use a library. Keep wrapper < 50 LOC.
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

**Tooling:** `thegent prompts sessions` to list sessions; `thegent prompts dump <session_id>` to dump to docs/research/.

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
| Continue prior session | `thegent bg "Task" -C <session_id>` |
| Session mgmt | `thegent ps` / `thegent status <id>` / `thegent wait <id>` |
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

- **Don't** use busy loops → use `plan wait-next` or `wait <id>`
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

Use AgilePlus for spec tracking: `agileplus <command>`. Check `/` for slash commands. Start new conversation to switch agent personas.

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

- **All projects**: Level 3+ (≥80% coverage, FR traceability ≥50%, security scanning, strict linters)
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
- New hooks: `hooks/<event>-<name>.sh` + register in `hooks/hook-config.yaml`
- Shared hook logic: `hooks/lib/<utility>.sh` (sourced by hooks, never called directly)
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
| Agent persona | `agents/<name>.md` |
| Lifecycle hook | `hooks/<event>-<name>.sh` + `hooks/hook-config.yaml` |
| Governance policy | `contracts/<policy>.json` + `qa-policy-engine.sh` |
| MCP tool | MCP server (FastMCP pattern) |
| CLI command | `commands/<command>/` + register in dispatch |
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
| `thegent bg "Task" -C <session_id>` | Continue from prior session |
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
