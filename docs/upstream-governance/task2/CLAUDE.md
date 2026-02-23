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

