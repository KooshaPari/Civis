# Merged Fragmented Markdown

## Source: civ/docs/upstream-governance/zen-mcp-server

## Source: AGENTS.md

# Build & Test

- Install: `pip install -e .[dev]` (requires Python 3.12+; use the repo `.python-version`).
- Unit tests: `python -m pytest -xvs`.
- Targeted test: `python -m pytest tests/<path>::<TestClass>::<test_name> -xvs`.
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

- Branch naming: `feature/<slug>`, `fix/<slug>`, `chore/<slug>`.
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
- Use `/housekeeping <scope>` (via `ops-concierge`) for simple chores so dev droids stay focused on feature work.

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

---

## Source: CLAUDE.md

# Claude Development Guide for Zen MCP Server

This file contains essential commands and workflows for developing and maintaining the Zen MCP Server when working with Claude. Use these instructions to efficiently run quality checks, manage the server, check logs, and run tests.

## Overview

Zen MCP Server is a comprehensive Model Context Protocol (MCP) server that orchestrates multiple AI models for enhanced development workflows. It provides unified deployment with dashboard interface, multi-model collaboration, and extensive tool ecosystem for code analysis, review, debugging, and project management.

> **New to the repo?** Run `/dev-setup <machine-name>` to bootstrap dependencies, then `/agent-briefing <task>` before delegating to the specialized droids. Claude owns the end-to-end flow (idea → planning → dev → review → QA → deploy → evaluation); keep hand-offs explicit so subagents can run in parallel.

## Architecture

### Core Components

**Consolidated FastMCP Server**
- Primary: Consolidated FastMCP HTTP server (`server.py`) with 100% feature utilization
- Legacy: STDIO server archived under `archive/stdio/`
- Remote access via HTTP clients in `clients/` (Python & TypeScript)
- Enhanced: All FastMCP features consolidated into single entry point

**Unified Deployment System**
- **Single Domain Architecture**: `zen.kooshapari.com` serves both MCP API and dashboard
- **Path-based Routing**: `/mcp` → MCP Server, `/` → Next.js Dashboard
- **Authentication**: FastMCP GitHub OAuth with user allowlists and JWT tokens
- **KInfra Integration**: Smart port allocation and tunnel management

**Tool Ecosystem** 
- **Core Tools** (`tools/`): 20+ specialized AI workflow tools
- **System Prompts** (`systemprompts/`): Optimized prompts for each tool
- **Provider System** (`providers/`): 8+ AI model providers (Gemini, OpenAI, etc.)
- **Workflow Engine**: Multi-step orchestration with conversation continuity

**Infrastructure**
- **Messaging**: NATS JetStream, Redis, RabbitMQ support
- **Storage**: Postgres persistence with vector store capabilities  
- **Monitoring**: Comprehensive logging, health checks, metrics dashboard
- **Security**: GitHub SSO, IP whitelisting, security middleware

### Data Flow

1. **MCP Client** → HTTP/JSON-RPC → **MCP Server** → **Tool Router**
2. **Tool** → **Provider** → **AI Model** → Response → **Conversation Memory**
3. **Cross-tool continuity**: Context preserved across tool invocations
4. **Multi-model workflows**: Claude coordinates with Gemini, O3, Flash, etc.

## Consolidated FastMCP Server

The Zen MCP Server now uses a **single, consolidated entry point** (`server.py`) that includes 100% FastMCP feature utilization:

### 🔥 Key Features
- **GitHub OAuth Authentication**: Secure authentication with user allowlists
- **Native Prompts System**: 6 comprehensive prompt templates for workflows
- **Tool Transformations**: Context injection, sampling, and state management
- **Advanced Resource System**: 8+ URI schemes (zen://, system://, config://, etc.)
- **Production Middleware**: Security headers, rate limiting, CORS, monitoring
- **CLI Integration**: Comprehensive command-line interface with validation

### 🚀 Server Usage
```bash
# Start production server
python server.py

# Development mode with debug logging
python server.py --dev

# Validate configuration without starting
python server.py --validate

# Show comprehensive server information
python server.py --info

# Generate .env configuration template
python server.py --generate-config

# Show help with all options
python server.py --help
```

### 🔧 Configuration
The server uses environment variables from `.env` file. Key settings:
```bash
# Authentication (required for security)
GITHUB_CLIENT_ID=your-github-client-id
GITHUB_CLIENT_SECRET=your-github-client-secret
GITHUB_ALLOWED_USERS=kooshapari
JWT_SECRET=secure-32-char-secret

# Server configuration
HOST=0.0.0.0
PORT=8080
MCP_BASE_PATH=/mcp
FASTMCP_LOG_LEVEL=INFO

# Tool selection
DISABLED_TOOLS=analyze,refactor,testgen,secaudit,docgen,tracer
```

Generate a complete configuration template with:
```bash
python server.py --generate-config
```

### 🔍 Health Checks & Monitoring
- Health Check: `http://localhost:8080/healthz`
- Metrics: `http://localhost:8080/metrics`
- Ready Probe: `http://localhost:8080/ready`
- Live Probe: `http://localhost:8080/live`

## Quick Reference Commands

### Custom Droids & Slash Commands

The repo ships ready-to-use droids in `.factory/droids/` (project overrides) and matching personal templates in `.factory/personal/`. Copy the personal variants to `~/.factory/droids/` to keep them available across workspaces. Key personas:

- `code-reviewer` — regression and hex-boundary enforcement
- `blueprint-curator` — documentation alignment
- `test-strategist` — layered validation planning
- `security-auditor` — auth/secrets/supply-chain sweeps
- `plan-orchestrator` — WBS/PERT and parallel workstream planning
- `ops-concierge` — housekeeping, status pings, and operational follow-through

Slash commands in `.factory/commands/` provide reusable prompts that pair well with droid delegation:

- **Reviews & Quality**: `/code-review`, `/quality-gate`, `/test-strategy`, `/simulator-suite`
- **Docs & Planning**: `/doc-sync`, `/planning-brief`, `/workprompt-index`, `/agent-briefing`, `/wbs-pert`
- **Security & Ops**: `/security-sweep`, `/provider-audit`, `/log-triage`, `/ops-handoff`, `/release-checklist`, `/housekeeping`
- **Platform Enablement**: `/dev-setup`, `/migration-scout`, `/dependency-scan`, `/warp-ops`, `/standup`

Reload with `/commands` → `R` after editing. Use the Task tool to hand specific analyses off to the droids mid-session.
> Parallel tip: kick off `plan-orchestrator` for WBS/PERT while `test-strategist` and `ops-concierge` run in separate tool calls; once their outputs land, feed them into `/quality-gate` or `/release-checklist`.

### Code Quality Checks

Before making any changes or submitting PRs, always run the comprehensive quality checks:

```bash
# Activate virtual environment first
source venv/bin/activate

# Run all quality checks (linting, formatting, tests)
./code_quality_checks.sh
```

This script automatically runs:
- Ruff linting with auto-fix
- Black code formatting 
- Import sorting with isort
- Complete unit test suite (excluding integration tests)
- Verification that all checks pass 100%

> Tip: `/quality-gate <branch>` asks the assistant to confirm these checks, gather artifacts (screenshots, logs), and propose follow-up actions before you request a review.

**Run Integration Tests (requires API keys):**
```bash
# Run integration tests that make real API calls
./run_integration_tests.sh

# Run integration tests + legacy simulator tests
./run_integration_tests.sh --with-simulator
```

### Server Management

#### Setup/Update the Server
```bash
# Run setup script (handles everything)
./run-server.sh
```

This script will:
- Set up Python virtual environment
- Install all dependencies
- Create/update .env file
- Configure MCP with Claude
- Verify API keys

#### Unified Deployment (Recommended)

```bash
# Complete setup with all services
make unified-full

# Start unified MCP + Dashboard deployment  
make unified-deploy

# Check system status
make unified-status

# View logs in real-time
make unified-logs-follow

# Stop all services
make unified-stop

# Clean restart
make unified-stop && make clean && make unified-deploy
```

#### Development Workflow

```bash
# Setup development environment
make setup-dev

# Run quality checks (linting, formatting, tests)
make quality-check

# Build dashboard
make build-dashboard

# Integration tests with local models
make integration-test
```

### Legacy MCP Server (Single Service)

```bash
# Start MCP server (FastMCP)
make zen-start

# Check MCP server status  
make zen-status

# View MCP logs
make zen-logs

# Stop MCP server
make zen-stop
```

### Container Services

```bash
# Start supporting services (NATS, Redis, RabbitMQ)
make dev-all-lite

# Stop services
make dev-all-lite-down

# View service logs
make dev-all-lite-logs
```

#### View Logs
```bash
# Follow logs in real-time
./run-server.sh -f

# Or manually view logs
tail -f logs/mcp_http_server.log
```

### Log Management

Use `/log-triage <issue>` for a guided log investigation (commands, filters, hypotheses) before diving into the files below.

Run `/housekeeping <scope>` regularly so `ops-concierge` can queue lightweight chores (log rotation, env sync, dependency bumps) while primary workflows execute.

#### View Server Logs
```bash
# View last 500 lines of server logs
tail -n 500 logs/mcp_http_server.log

# Follow logs in real-time
tail -f logs/mcp_http_server.log

# View specific number of lines
tail -n 100 logs/mcp_http_server.log

# Search logs for specific patterns
grep "ERROR" logs/mcp_http_server.log
grep "tool_name" logs/mcp_activity.log
```

#### Monitor Tool Executions Only
```bash
# View tool activity log (focused on tool calls and completions)
tail -n 100 logs/mcp_activity.log

# Follow tool activity in real-time
tail -f logs/mcp_activity.log

# Use simple tail commands to monitor logs
tail -f logs/mcp_activity.log | grep -E "(TOOL_CALL|TOOL_COMPLETED|ERROR|WARNING)"
```

#### Available Log Files

**Current log files (with proper rotation):**
```bash
# Main server log (all activity including debug info) - 20MB max, 10 backups
tail -f logs/mcp_http_server.log

# Tool activity only (TOOL_CALL, TOOL_COMPLETED, etc.) - 20MB max, 5 backups  
tail -f logs/mcp_activity.log
```

### Testing

The Zen MCP Server has **2,040+ total tests** including comprehensive unit and integration test suites.

#### Quick Test Execution (Recommended)
```bash
# Fast unit tests only (~1,950 tests, ~2-5 minutes)
./scripts/run_unit_tests.sh

# Comprehensive tests including integration (~2,040 tests, ~10-15 minutes) 
./scripts/run_comprehensive_tests.sh

# Validate test coverage and execution patterns
./scripts/validate_test_coverage.py
```

#### Manual Test Execution

**Unit Tests Only (Fast Development Workflow)**
```bash
# Run ~1,950 unit tests (excluding integration tests)
python -m pytest tests/ -v -m "not integration"

# Run specific test file
python -m pytest tests/test_refactor.py -v

# Run specific test function  
python -m pytest tests/test_refactor.py::TestRefactorTool::test_format_response -v

# Run with coverage reporting
python -m pytest tests/ --cov=. --cov-report=html -m "not integration"
```

**All Tests Including Integration (Maximum Coverage)**
```bash
# Enable all providers for comprehensive testing
export ENABLED_PROVIDERS="OPENROUTER,GEMINI,OPENAI,XAI,CUSTOM,AEGIS,DIAL,CURSOR,GOOSE,MCP_AGENT"

# Run ALL 2,040+ tests (unit + integration)
python -m pytest tests/ -v --maxfail=999

# Run with coverage and detailed reporting
python -m pytest tests/ -v --maxfail=999 --cov=. --cov-report=html --junit-xml=reports/test-results.xml
```

#### Integration Tests Setup (Uses Free Local Models)

> Delegate planning with `/simulator-suite full` to capture the right command flags, log artifacts, and follow-up analysis.

**Setup Requirements:**
```bash
# 1. Install Ollama (if not already installed) 
# Visit https://ollama.ai or use brew install ollama

# 2. Start Ollama service
ollama serve

# 3. Pull a model (e.g., llama3.2)
ollama pull llama3.2

# 4. Set environment variable for custom provider
export CUSTOM_API_URL="http://localhost:11434"
```

**Run Integration Tests Only:**
```bash
# Run ~90 integration tests that make real API calls to local models
python -m pytest tests/ -v -m "integration"

# Run specific integration test
# (examples; adjust to a non-auth test now that legacy OAuth2 is removed)
python -m pytest tests/integration/test_http_server_integration.py::TestMCPProtocolIntegration::test_mcp_tools_list_endpoint -v
```

#### Test Coverage Analysis

**Key Test Statistics:**
- **Total Tests**: 2,040+ (1,950 unit + 90 integration)
- **Collection Time**: ~6-10 seconds
- **Unit Test Runtime**: ~2-5 minutes  
- **Full Test Runtime**: ~10-15 minutes
- **Target Coverage**: 95%+

**Common Issues Fixed:**
- ✅ **Early termination**: Removed `--maxfail=10` limit
- ✅ **Integration exclusion**: Made integration tests optional via markers
- ✅ **Provider skipping**: Added comprehensive mode for all provider tests
- ✅ **Slow tests**: Added performance monitoring and optimization

**Note**: Integration tests use local Ollama models (completely FREE). The comprehensive test scripts automatically configure all providers and execution parameters for maximum test coverage.

## Configuration

### Tool Configuration

Essential tools enabled by default: `chat`, `thinkdeep`, `planner`, `consensus`, `codereview`, `precommit`, `debug`, `challenge`

To enable additional tools, modify `DISABLED_TOOLS` in `.env`:
```bash
# Default (optimizes context window)
DISABLED_TOOLS=analyze,refactor,testgen,secaudit,docgen,tracer

# Enable specific tools
DISABLED_TOOLS=refactor,testgen,secaudit,docgen,tracer

# Enable all tools  
DISABLED_TOOLS=
```

### Model Configuration

Run `/provider-audit <provider>` when changing credentials or model routing to verify env vars, rate limits, and smoke tests.

```bash
# Provider API keys
GEMINI_API_KEY=your-key
OPENAI_API_KEY=your-key  
OPENROUTER_API_KEY=your-key

# Model selection
DEFAULT_MODEL=auto  # or specific model like "pro", "flash", "o3"
DEFAULT_THINKING_MODE_THINKDEEP=high

# Conversation settings
CONVERSATION_TIMEOUT_HOURS=6
MAX_CONVERSATION_TURNS=50
```

### Deployment Configuration

```bash
# Unified deployment
TUNNEL_ID=your-cloudflare-tunnel-id
TUNNEL_DOMAIN=kooshapari.com
SRVC=zen

# Security
DISABLE_ANONYMOUS_ACCESS=true
ENABLE_HTTP_OAUTH=true
WEBAUTHN_ENABLED=true

# Services
MCP_PORT=8080
DASHBOARD_PORT=5173
NODE_ENV=production
```

## Key Features for Development

### Multi-Model Workflows

The system enables Claude to coordinate with other AI models within single conversations:

```
"Perform a codereview using gemini pro and o3, then use planner to create fix strategy"
```

This triggers:
1. Claude systematically reviews code
2. Consults Gemini Pro for deep analysis  
3. Gets O3's perspective on issues
4. Creates unified implementation plan
5. Context flows seamlessly across all steps

### Context Revival

When Claude's context resets, continue conversations by invoking other models:
```
"Continue with o3 to complete the analysis"
```
The other model's response revives Claude's understanding without re-reading files.

### Conversation Memory

All tool interactions maintain context through `utils/conversation_memory.py`:
- Cross-tool conversation threading
- File deduplication and efficient context packing
- Token allocation management
- Conversation history persistence

### Workflow Orchestration

Tools implement systematic workflows:
- **`codereview`**: Multi-pass analysis with severity levels
- **`debug`**: Hypothesis-driven investigation with confidence tracking  
- **`planner`**: Structured project breakdown with actionable steps
- **`consensus`**: Multi-model expert consultation with stance steering

## Development Notes

### Before Making Changes
1. Run quality checks: `make quality-check`  
2. Check service health: `make unified-status`
3. Ensure virtual environment: `source venv/bin/activate` or `source .zen_venv/bin/activate`

### After Making Changes  
1. Quality validation: `make quality-check`
2. Integration testing: `make integration-test`
3. Restart Claude session for MCP changes to take effect

### Key Files for Modification

**Core Server**: `server.py` - FastMCP-native MCP server
**Tool Implementation**: `tools/*.py` - Individual tool logic  
**Prompts**: `systemprompts/*.py` - System prompts for each tool
**Providers**: `providers/*.py` - AI model provider implementations
**Deployment**: `deploy_unified_with_kinfra.py` - Unified deployment orchestrator
**Configuration**: `config.py` - Central configuration management

### Logging

Two main log streams with rotation:
- **`logs/mcp_http_server.log`** - All server activity (20MB max, 10 backups)
- **`logs/mcp_activity.log`** - Tool calls only (20MB max, 5 backups)

Monitor during development:
```bash
tail -f logs/mcp_activity.log | grep -E "(TOOL_CALL|TOOL_COMPLETED|ERROR)"
```

### Security Architecture

The system implements comprehensive security:
- **Authentication Middleware**: `security_middleware.py` enforces auth requirements
- **GitHub SSO**: `auth/github_oauth.py` handles GitHub App OAuth and token issuance  
- **Path-based Security**: Different auth methods for different URL paths

## Advanced Usage

### Remote Agent Orchestration

Use `mcp_agent` tool to coordinate multiple MCP servers:
```python
# Register remote agent
{"action": "register", "agent_name": "dev-server", "agent_url": "http://dev-host:8080/mcp"}

# Execute tool on remote agent  
{"action": "execute", "agent_name": "dev-server", "tool_name": "analyze", "tool_args": {"code": "..."}}
```

### Large Prompt Handling

System automatically handles MCP's 25K token limit:
- File content chunking and summarization
- Context-aware prompt compression  
- Multi-request workflows for large codebases

### Custom Model Integration

Add new providers in `providers/` implementing the base provider interface:
- Request/response formatting
- Token usage tracking
- Error handling and retries
- Model capability detection

This architecture enables Claude Code to leverage the full power of multi-model AI collaboration while maintaining conversation continuity and systematic workflows.

---

---
Copied count: 2
Source path: `civ/docs/upstream-governance/zen-mcp-server`
