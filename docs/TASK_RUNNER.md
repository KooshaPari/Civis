# Polyglot Task Runner

DINOForge uses **[task](https://taskfile.dev/)** (v3.49.1+) for unified build, test, and deployment commands across all languages and components.

## Installation

### Windows

```powershell
# Option 1: Chocolatey (recommended)
choco install task

# Option 2: Scoop
scoop install task

# Option 3: Direct download
# Download from https://taskfile.dev/installation/
# Or run from the repo after first clone:
./bin/task --version
```

### macOS

```bash
brew install go-task
```

### Linux

```bash
sh -c '$(curl --location https://taskfile.dev/install.sh)' -- -d
sudo mv ./task /usr/local/bin/
```

Verify: `task --version` (should show 3.49.1+)

## Quick Start

```bash
# List all available tasks
task --list

# Build everything
task build:all

# Run all tests
task test:all

# Full pre-commit validation (lint + build + test)
task precommit

# Clean build artifacts
task clean
```

## Common Tasks

### C# Components

```bash
# Build C# solution (Release)
task build:csharp

# Run C# unit + integration tests with code coverage
task test:csharp

# Format check (no modifications)
task lint
```

### Python MCP Server

```bash
# Install MCP server in editable mode (development)
task build:python

# Run Python tests (pytest)
task test:python
```

### Pack Management

```bash
# Validate all content packs
task pack:validate

# Build a specific pack
task pack:build PACK_NAME=example-balance
task pack:build PACK_NAME=warfare-starwars
```

### Game Automation

```bash
# Launch the main game instance
task game:launch

# Kill all game processes
task game:kill

# Deploy Runtime to MAIN game instance
task deploy:runtime

# Deploy Runtime to isolated TEST instance (concurrent launch)
task deploy:runtime:test
```

### Documentation

```bash
# Build VitePress docs (static HTML)
task docs

# Start local dev server (with live reload)
task docs:dev
```

## Development Workflow

### First-Time Setup

```bash
# Install all dependencies (dotnet restore + Python deps)
task dev:install
```

### Pre-Commit Validation

Before pushing code, run the full validation suite:

```bash
task precommit
# This runs: lint → build:all → test:all
```

### Build-Deploy-Test Loop

```bash
# Build everything
task build:all

# Deploy to game (main instance)
task deploy:runtime

# In another terminal, verify game state
task check-game  # (if using /check-game skill)

# Run tests
task test:csharp
```

## Task Internals

### Taskfile Structure

- **Location**: `C:\Users\koosh\Dino\Taskfile.yaml`
- **Format**: YAML v3 syntax
- **Environment**: Shared vars: `RUST_BACKTRACE=1`, `DOTNET_SKIP_FIRST_TIME_EXPERIENCE=true`

### Task Categories

| Category | Purpose | Examples |
|----------|---------|----------|
| `build:*` | Compile components | `build:all`, `build:csharp`, `build:python` |
| `test:*` | Run test suites | `test:all`, `test:csharp`, `test:python` |
| `deploy:*` | Deploy to game | `deploy:runtime`, `deploy:runtime:test` |
| `game:*` | Game automation | `game:launch`, `game:kill` |
| `pack:*` | Pack management | `pack:validate`, `pack:build` |
| `docs:*` | Documentation | `docs`, `docs:dev` |
| utility | Helpers | `lint`, `clean`, `dev:install`, `precommit` |

### Custom Parameters

Some tasks accept variables:

```bash
# Pack build requires PACK_NAME
task pack:build PACK_NAME=economy-balanced

# Both PACK_NAME and other flags
task pack:build PACK_NAME=warfare-starwars
```

## Wrapper Scripts

For convenience, the repo includes optional wrapper scripts:

### PowerShell

```powershell
# List all tasks
./scripts/task-wrapper.ps1 -List

# Run a task
./scripts/task-wrapper.ps1 -Task build:csharp
```

### Bash

```bash
# List all tasks
./scripts/task-wrapper.sh --list

# Run a task
./scripts/task-wrapper.sh build:csharp
```

## Performance Notes

- **C# build**: ~20s (Release config, incremental)
- **C# tests**: ~50-60s (1,269 tests, code coverage enabled)
- **Python tests**: ~15-30s (if dependencies installed)
- **Full precommit**: ~2-3 minutes

## Troubleshooting

### Task not found

```bash
# Windows: install via choco or download directly
choco install task

# Verify installation
task --version
```

### "XPlat Code Coverage" not found

This warning is non-fatal — tests still run and code coverage collection happens via Coverlet. The message appears but doesn't fail the build.

### Python test skip (missing deps)

If Python tests skip, install dev dependencies first:

```bash
task build:python
# Then:
task test:python
```

### Performance test flaky

The `PerformanceBenchmarkTests.CompatibilityCheck_ComplexRanges_StillFast` test is timing-sensitive and may fail under heavy system load. This is expected — rerun if CI shows intermittent failures.

## Integration with CI/CD

GitHub Actions workflows use `task` for all builds/tests:

```yaml
# Example from .github/workflows/build.yml
- name: Build
  run: task build:all

- name: Test
  run: task test:all
```

## Future Extensions

Planned task additions as polyglot components expand:

```yaml
# Rust asset pipeline (coming)
build:rust       # cargo build --release
test:rust        # cargo test --release
bench:rust       # cargo bench

# Go dependency resolver (coming)
build:go         # go build -o bin/dinoforge-resolver
test:go          # go test -v ./...

# Zig asset pipeline (coming)
build:zig        # zig build -Doptimize=ReleaseFast
test:zig         # zig build test
```

## References

- **Official Docs**: https://taskfile.dev/
- **Taskfile Syntax**: https://taskfile.dev/api/
- **Task Installation**: https://taskfile.dev/installation/
