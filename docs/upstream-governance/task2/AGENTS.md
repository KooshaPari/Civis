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

