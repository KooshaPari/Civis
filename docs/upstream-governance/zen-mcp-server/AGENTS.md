# Build & Test

- Install: `pip install -e .[dev]` (requires Python 3.12+; use the repo `.python-version`).
- Unit tests: `python -m pytest -xvs`.
- Targeted test: `python -m pytest tests/\<path\>::\<TestClass\>::\<test_name\> -xvs`.
- Simulator suite: `python communication_simulator_test.py --verbose`.
- Lint & format: `ruff check .`, `black --check .`, `isort --check-only .`.
- Auto-fix style: `ruff check . --fix`, `black .`, `isort .`.
- Logs tailing while tests run: `tail -f logs/mcp_server.log` or `tail -f logs/mcp_activity.log`.

Always run unit tests plus `ruff check .` before committing. Run the simulator suite when touching protocol, provider, or streaming code.

# Architecture Overview

- `api/`: FastAPI HTTP surface (MCP-compatible JSON-RPC bridge, auth guards, health probes).
- `server/` & `services/`: Core orchestration, streaming manager, task execution, and provider wiring.
- `providers/`: Model/client adapters; follow hexagonal ports/adapters. Keep credentials outside repo.
- `tools/`: MCP tool implementations (Read, Execute, Planner, etc.) used by both CLI and automation.
- `work-prompts/`: Canonical prompt sheets and templates that drive architecture, planning, and QA reviews.
- `docs/`: Canonical blueprints, plans, and agent guides (`docs/misc_top_level/AGENTS.md`, `docs/dev-agents/`, `docs/plans/`).
- `tests/`, `smoke/`, `qa_data/`: Pytest suites, smoke harnesses, and fixtures. Respect layering—unit tests stay near subject.

Guardrails: maintain ports/adapters separation, keep provider logic out of domain services, persist environment secrets via `.env*`.

# Agent Tooling & Shortcuts

This repository runs a 100% agent-driven lifecycle: humans provide intent or feedback, and droids handle research → planning → development → refinement → review → QA → deployment → evaluation. Keep requests scoped and trigger the right subagents so the pipeline stays autonomous.

Custom droids live in `.factory/droids/` (project overrides) and `.factory/personal/` (copy to `~/.factory/droids/` if desired):
- `code-reviewer`, `blueprint-curator`, `test-strategist`, `security-auditor`, `plan-orchestrator`, `ops-concierge`.

Custom slash commands are in `.factory/commands/`. Highlights:
- **Reviews & Quality**: `/code-review`, `/quality-gate`, `/test-strategy`, `/simulator-suite`.
- **Docs & Planning**: `/doc-sync`, `/planning-brief`, `/workprompt-index`, `/agent-briefing`, `/wbs-pert`.
- **Security & Ops**: `/security-sweep`, `/provider-audit`, `/log-triage`, `/ops-handoff`, `/release-checklist`, `/housekeeping`.
- **Platform Enablement**: `/dev-setup`, `/migration-scout`, `/dependency-scan`, `/warp-ops`, `/standup`.

Parallel delegations are encouraged: let `plan-orchestrator` build WBS/PERT while `test-strategist` fleshes out coverage and `ops-concierge` clears housekeeping, then feed their outputs into `/quality-gate` or `/release-checklist`.

Use `/commands` → **Reload (R)** after editing. Commands reference the work prompts and blueprints above.

# Git Workflow & Evidence

- Branch naming: `feature/\<slug\>`, `fix/\<slug\>`, `chore/\<slug\>`.
- Keep commits atomic; prefer `feat:`, `fix:`, `test:`, `docs:`, `chore:`.
- Mandatory proof before merge:
  - `python -m pytest -xvs`
  - `ruff check .`
  - Evidence: log excerpt, screenshot, or simulator output when relevant.
  - Update blueprints/work-prompts when architecture or process changes occur.

Use `/release-checklist` before tagging, `/incident-postmortem` after outages, and `/doc-sync` whenever docs drift.

# Security & Ops Notes

- Secrets live in `.env*` (never commit). Auth modules under `auth/` and `conf/` expect environment configuration.
- Network/tool execution is sandboxed; expand permissions via `.codex/permissions.local.toml`, `.claude/settings.local.json`, `.auggie/settings.local.json`.
- When changing provider integrations or auth flows, run `/security-sweep` and coordinate with `docs/reports/` owners.
- Logs rotate at 20 MB; use `logs/mcp_server.log` and `logs/mcp_activity.log` for debugging. Simulator tests require `LOG_LEVEL=DEBUG`.
- Use `/housekeeping \<scope\>` (via `ops-concierge`) for simple chores so dev droids stay focused on feature work.

# Documentation & Prompts

- Architecture source of truth: `ARCHITECTURE_BLUEPRINT.md`.
- Delivery planning: `DELIVERY_BLUEPRINT.md`, `docs/plans/`.
- Prompt canon: `work-prompts/` with quick index via `/workprompt-index`.
- Agent guides: `docs/dev-agents/CLAUDE.md`, `docs/dev-agents/claude-code-sdk.md`, `docs/misc_top_level/AGENTS.md`.
- Update docs alongside code; `/doc-sync` and `blueprint-curator` help enumerate edits.

# Validation Checklist Before Merge

- ✅ Code change reviewed by `code-reviewer` droid or `/code-review`.
- ✅ Tests & lint commands executed with passing output attached to PR.
- ✅ Documentation updates identified via `/doc-sync`.
- ✅ Security review performed (`/security-sweep`) for auth/provider changes.
- ✅ Release implications captured via `/release-checklist` when applicable.

Agents should read this briefing plus the relevant work prompt before planning any change.
