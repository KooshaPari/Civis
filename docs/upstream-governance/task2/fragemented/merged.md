# Merged Fragmented Markdown

## Source: civ/docs/upstream-governance/task2

## Source: AGENTS.md

<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

# Agents and Automation Guide: Stdio MCP Task Server

This repository is designed to work seamlessly with Claude (and other advanced AI agents) as autonomous software engineers building a stdio MCP server that delegates tasks to agent CLIs.

**Authority and Scope**
- The FastMCP Canonical Contract lives in `llms-full.txt`. Treat `llms-full.txt` as the final authority for all FastMCP behavior; override any model priors with it.
- This file summarizes repo-specific mandates; FastMCP fundamentals are fully detailed in `llms-full.txt`.
- Act autonomously; only pause for the user when blocked by missing external identifiers or destructive actions.

## Core Expectations for Agents

### Autonomous Operation (Critical - Minimal Human Intervention)

Agents MUST operate with **maximum autonomy**:

**When to proceed without asking:**
- Implementation details and technical approach decisions
- Agent CLI selection and model routing
- Rate limiting fallback strategies
- Code structure and organization
- Test strategies and coverage approaches
- Refactoring and optimization decisions
- Bug fixes and performance improvements
- Documentation updates

**Only ask when truly blocked by:**
- Missing credentials/secrets (cannot be inferred from environment)
- External service access permissions
- Genuine product ambiguity (behavior not determinable from specs/code/tests)
- Destructive operations (production data deletion, forced pushes)

**Default behavior: Research → Decide → Implement → Validate → Continue**

### Research-First Development (CRITICAL)

Before implementing ANY feature or fix, agents MUST conduct comprehensive research:

**1. Codebase Research (Always Required):**
```bash
# Find agent CLI patterns
rg "cursor-agent|droid-agent" --type py -A 5 -B 5

# Find model selection logic
rg "model.*select|select.*model" --type py

# Find rate limiting handling
rg "429|400|rate.*limit" --type py

# Find task delegation patterns
rg "delegate|subprocess|invoke" --type py
```

**2. Web Research (When Needed):**
- FastMCP documentation: https://fastmcp.wiki/
- Agent CLI documentation
- Model API rate limiting patterns
- Async task execution patterns

**3. Research Documentation:**
- Document findings in `docs/sessions/<session-id>/01_RESEARCH.md`
- Include URLs, code examples, and decision rationale
- Update continuously as new information discovered

### File Size & Modularity Constraints

**Hard constraint: All modules ≤500 lines (target ≤350)**

- Check line count before adding features
- If file approaches 350+ lines → decompose immediately
- Extract cohesive responsibilities (model selection, rate limiting, task delegation)
- Use clear, narrow interfaces
- Update imports in all callers; test thoroughly

### Aggressive Change Policy (CRITICAL)

**NO backwards compatibility. NO gentle migrations. NO MVP-grade implementations.**

- **Avoid ANY backwards compatibility shims or legacy fallbacks**
- **Always perform FULL, COMPLETE changes** when refactoring
- **Do NOT preserve deprecated patterns** for transition periods
- **Remove old code paths entirely** when replacing them
- **Update ALL callers simultaneously** when changing signatures
- **This enables clarity, performance, and maintainability**

## Repo-Specific Architecture Mandates

### Agent CLI Delegation

- **Cursor-Agent**: Auto model selection for unlimited/spam tasks
- **Droid-Agent**: Custom model routing for all other tasks
- **Model Selection Strategy**:
  - Sonnet 4.5: Premium reasoning, conserve usage
  - GPT-5.1 Codex High: Highest-accuracy coder (slow but safe)
  - Claude Haiku 4.5: Fast worker (watch quota)
  - GPT-5.1 Codex Medium/Low: Everyday coding w/ “think down to zero” steps
  - GLM 4.6 (Z.AI + Cerebras): Specialist burst pair
  - Cursor Auto (`router:free-first`): Emergency-only fallback

### Rate Limiting Handling

- **GLM 4.6**: Detect 429/400 errors, auto-fallback to next model
- **Fallback Chain**: Sonnet 4.5 → GPT-5.1 Codex High → Haiku → GPT-5.1 Codex Medium/Low → GLM 4.6 (Z.AI then Cerebras) → Cursor Auto
- **Exponential Backoff**: Implement retry logic with jitter
- **Circuit Breaker**: Track model failures, skip temporarily

### Server Architecture

- **Server**: Single consolidated FastMCP in `server.py`
- **Tool design**: Place business logic in `services/`; orchestrate only in tools
- **Adapters**: Go through `infrastructure/`; never bypass
- **Tests**: Run via `uv run pytest`; use FastMCP client In-Memory for unit tests
- **File size (critical)**: All modules ≤500 lines; target ≤350 lines

## Recommended Agent Behaviors

1. **Discovery**
   - Inspect `server.py`, `tools/`, `services/`, `infrastructure/`, `tests/`
   - Use `rg`/search to trace call chains before edits
   - Check line counts on files you'll modify; plan decomposition if near 350+ lines
   - Identify ALL agent CLI invocation patterns

2. **Planning & Implementation**
   - Draft a concise plan per task; implement directly without waiting for confirmation
   - Align with existing style, typing, logging, and error handling
   - **Size-aware design**: If a feature would push a file above 350 lines, plan modular decomposition upfront
   - Plan model selection and rate limiting strategies upfront

3. **Testing & Validation**
   - Run relevant `uv run pytest` targets after modifications
   - If failures appear, analyze, patch, and re-run until resolved or clearly blocked
   - Verify decomposed modules have equivalent test coverage
   - Test rate limiting fallback scenarios

4. **Safety & Secrets**
   - Never add real credentials or tokens; use env vars and placeholders
   - Respect environment-driven configuration and deployment files
   - Verify final line counts and commit structure before pushing
   - Ensure rate limiting doesn't expose sensitive model information

## Interaction Rules

- Operate in the tight loop referenced above
- Do not ask for next steps unless truly blocked by secrets or irreversible actions
- Keep communication lean; prioritize code and commands referencing this contract and `llms-full.txt`
- Document work in session folders; consolidate aggressively to prevent proliferation

---

## Source: CLAUDE.md

<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

# Claude Usage Guide: Stdio MCP Task Server

This repository is optimized for Claude (and similar LLM agents) acting as an autonomous senior software engineer for the stdio MCP task server that delegates to agent CLIs.

**Core rules (non-optional):**
- Always activate the project environment: `source .venv/bin/activate` before running Python/uv.
- Prefer `uv` for all Python execution and dependency operations (`uv run`, `uv pip`, `uvx`).
- Follow the closed-loop workflow: review → research (code/web/reasoning) → plan → execute → test → polish → repeat.
- Do not ask the user what to do next unless blocked by missing secrets, external access, or true ambiguity.
- Respect repo architecture, abstractions, auth, and env conventions at all times.
- **Keep all modules ≤500 lines; target ≤350 lines. Decompose proactively to maintain clarity.**
- **Avoid backwards compatibility shims, legacy fallbacks, or gentle migrations. Always perform FULL, COMPLETE changes.**

## 1. Repository Mental Model for Claude

Understand these as first-class constraints before editing:
- Runtime: Python 3.12, FastMCP-based stdio MCP server.
- Key modules:
  - `server.py`: Core MCP server wiring, tool registration, task delegation.
  - `tools/`: Task tool implementations (CRUD, batch, DAG execution).
  - `services/`: Business logic (task execution, agent CLI delegation, model selection, rate limiting).
  - `infrastructure/`: Adapters (agent CLI invoker, model router, rate limiter, logging).
  - `tests/`: Structured into unit, integration, e2e.
- Style: Typed where practical, 100-char line length, Ruff/Black-compatible, clear logging, explicit error handling.
- **File size constraint: all modules ≤500 lines (target ≤350). Larger modules must be decomposed.**

Claude must:
- Reuse these layers instead of bypassing them.
- Keep changes minimal, composable, and driven by tests and call sites.
- Proactively decompose files approaching 350 lines into well-structured submodules.
- Implement agent CLI delegation and model selection strategically.

## 2. File Size & Modularity Mandate (Critical)

**Hard constraint: All modules must stay ≤500 lines; target ≤350 lines.**

Before adding features to any file, check its current line count. If it approaches 350+ lines:
1. **Identify cohesive responsibilities** (task execution, agent CLI delegation, model selection, rate limiting).
2. **Extract into separate modules** following this hierarchy:
   - Domain logic → new `services/<domain>/` submodule
   - Adapters/clients → new `infrastructure/` submodule
   - Tool orchestration → split by feature within `tools/`
   - Helpers/utilities → new `utils/` or domain-specific modules
3. **Update imports** in all callers; test each change with `uv run pytest`.
4. **Keep interfaces narrow**: expose only what's needed; hide internals.

**Aggressive Change Policy:**
- **When refactoring, update ALL callers and code paths simultaneously.** No partial migrations.
- **Remove old implementations entirely.** Don't leave deprecated code behind with conditional logic.
- **No feature flags, shims, or backwards compatibility layers.** Clean breaks enable clarity and performance.
- **All tests must pass after refactoring.** No "this part is still migrating."

## 3. Standard Operating Loop (SWE Autopilot)

For every task (bug, feature, infra, test):
1. **Review** - Read the issue/error, relevant code, and existing tests.
2. **Research** - Check related modules and patterns in-repo. Consult external docs if needed.
3. **Plan** - Formulate a short, concrete plan. Ensure alignment with existing abstractions.
4. **Execute** - Implement in small, verifiable increments. Match coding style.
5. **Test** - Run targeted tests via `uv run pytest …` relevant to the change.
6. **Review & Polish** - Re-read diffs mentally; simplify, remove dead code, align naming.
7. **Repeat** - If tests or behavior fail, loop without waiting for user direction.

## 4. Agent CLI Delegation & Model Selection

### Agent CLI Patterns

**Cursor-Agent** (auto model):
```python
# For unlimited/spam tasks
result = await invoke_cursor_agent(task_description, auto_model=True)
```

**Droid-Agent** (custom models):
```python
# For specific model selection
result = await invoke_droid_agent(
    task_description,
    model="gpt-5-codex-medium",  # or haiku, sonnet, etc.
    fallback_models=["haiku", "gpt-5-codex-medium"]
)
```

### Model Selection Strategy

- **Sonnet 4.5 (premium)**: Smartest model; conserve quota for hard reasoning.
- **GPT-5.1 Codex High**: Highest-accuracy coder; slow but safe when Sonnet is exhausted.
- **Claude Haiku 4.5**: Fast worker for throughput tasks (watch usage).
- **GPT-5.1 Codex Medium/Low**: Everyday coding + “think down to zero” fallbacks.
- **GLM 4.6 (Z.AI + Cerebras)**: Specialist burst pair exposed behind a single alias.
- **Cursor Auto (`router:free-first`)**: Last-resort when everything else fails.

### Rate Limiting Handling

```python
# Detect 429/400, auto-fallback
try:
    result = await invoke_droid_agent(task, model="glm-4.6")
except RateLimitError as e:
    if e.status_code in [429, 400]:
        result = await invoke_droid_agent(task, model="gpt-5.1-codex-medium")  # Fallback
```

## 5. Commands Claude Should Prefer

```bash
# Environment setup (always first)
source .venv/bin/activate

# ✅ REQUIRED: Use uv for all operations
uv run pytest tests/unit              # Run unit tests
uv run pytest tests/integration       # Run integration tests
uv run pytest --cov                   # Run with coverage
uv run ruff check .                   # Check code quality
uv run ruff format .                  # Format code
uv run python server.py               # Start MCP server
```

## 6. Behavioral Constraints for Claude

- Never introduce real secrets, keys, or tokens; use env vars and placeholders.
- Do not modify git config or push without explicit instruction.
- Prefer consolidated tools and services over direct implementations.
- Avoid speculative refactors unless required to fix a real issue.
- Keep explanations concise; focus tokens on accurate code and commands.
- **Always respect the 350-line target (500-line hard limit).** Decompose early, test thoroughly.
- **When refactoring, identify ALL callers and update them simultaneously.** No staggered migrations.
- **Remove old code paths entirely when replaced.** Zero tolerance for legacy conditionals or shims.
- **Implement rate limiting fallback gracefully.** Never expose model failures to end users.

## 7. When Claude Must Ask

Only pause for user input when:
- Credentials, domains, or external IDs are required and cannot be inferred.
- There is a genuine product/behavior ambiguity not answered by code/tests/docs.
- An operation may be destructive (data deletion, production migrations, forced pushes).

Otherwise, proceed autonomously following this guide and AGENTS.md as the contract.

---

---
Copied count: 2
Source path: `civ/docs/upstream-governance/task2`
