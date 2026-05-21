# Parallel Game Automation Infrastructure

DINOForge parallel automation enables concurrent testing and verification of multiple game instances using MCP (Model Context Protocol) tools.

## Overview

The parallel automation infrastructure consists of three main components:

1. **Launch-ParallelGames.ps1** — Spawn N isolated game instances
2. **Test-ParallelAutomation.ps1** — Send MCP commands in parallel, measure success rate
3. **Verify-GameState.ps1** — Validate game state via screenshots and visual analysis

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Parallel Launcher                        │
│  Creates N game processes with unique pipe names & configs  │
└────────────────────────┬────────────────────────────────────┘
                         │
         ┌───────────────┼───────────────┐
         │               │               │
    Instance 1      Instance 2      Instance N
   (Hidden Mode)   (Hidden Mode)   (Hidden Mode)
         │               │               │
         └───────────────┼───────────────┘
                         │
         ┌───────────────┼───────────────┐
         │ MCP Server (FastMCP Python)  │
         │   game_status                │
         │   game_query_entities        │
         │   game_verify_mod            │
         │   game_screenshot            │
         │   game_analyze_screen        │
         └───────────────────────────────┘
```

### MCP Tools Used

The automation test suite calls these MCP endpoints:

| Tool | Purpose |
|------|---------|
| `game_status` | Check if game is running, entity count |
| `game_query_entities` | Query entities by component type |
| `game_verify_mod` | Verify DINOForge runtime is loaded |
| `game_screenshot` | Capture game window |
| `game_analyze_screen` | Detect UI elements, entities (OmniParser) |

## Quick Start

### 1. Start the MCP Server

```powershell
.\scripts\start-mcp.ps1
```

Expected output:
```
Starting MCP server...
FastMCP listening on http://127.0.0.1:8765
```

### 2. Launch Parallel Instances

```powershell
.\scripts\automation\Launch-ParallelGames.ps1 -InstanceCount 2
```

Expected output:
```
=== DINOForge Parallel Game Launcher ===
Instance count: 2
Game path: G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option

Cleaning up existing game processes...
Launching 2 game instances...
  [Instance 1] Started (PID=12345)
  [Instance 2] Started (PID=12346)

Waiting for instances to stabilize...

Verifying instances...
✓ Running instances: 2/2

=== Launch Summary ===
Instances: 2 running
Pipe names:
  Instance 1: dinoforge-game-bridge-instance-1-4521
  Instance 2: dinoforge-game-bridge-instance-2-8934

✓ Parallel game launcher complete
```

### 3. Run Automation Tests

```powershell
.\scripts\automation\Test-ParallelAutomation.ps1 -InstanceCount 2 -TestDurationSeconds 30 -Verbose
```

Expected output:
```
=== DINOForge Parallel Automation Test ===
Instance count: 2
Test duration: 30s
MCP URL: http://127.0.0.1:8765

Checking MCP server health...
✓ MCP server is running

Launching 2 game instances...
Launched: 2 instances

Running test suite...
  ✓ Instance 1: game_status OK
  ✓ Instance 2: game_query_entities OK
  ✓ Instance 1: game_verify_mod OK
[Iteration 1] Passed: 6 | Failed: 0 | Time: 245ms
...

=== Test Results ===
Duration: 30.5 seconds
Iterations: 12
Total tests: 72
Passed: 70
Failed: 2
Success rate: 97.22%

✓ Test PASSED (95%+ success rate)

Cleaning up...
✓ Cleanup complete
```

### 4. Verify Game State

```powershell
.\scripts\automation\Verify-GameState.ps1 -InstanceCount 2 -TestDurationSeconds 30
```

Expected output:
```
=== DINOForge Game State Verification ===
Instance count: 2
Capture interval: 5s
Test duration: 30s
Output directory: docs/automation/screenshots

✓ Created output directory: docs/automation/screenshots
Verifying MCP server...
✓ MCP server is running

Starting verification loop...

=== Verification Report ===
Total duration: 30.3 seconds
Screenshots captured: 12
Analyses performed: 12
Anomalies detected: 0

✓ No anomalies detected

✓ Report saved: docs/automation/screenshots/verification_report.json
✓ Screenshots saved to: docs/automation/screenshots

=== Verification Complete ===
```

## Parameter Reference

### Launch-ParallelGames.ps1

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `InstanceCount` | int | 4 | Number of game instances to launch |
| `GamePath` | string | auto | Path to game executable directory (reads from Directory.Build.props if not specified) |
| `Verbose` | switch | off | Enable verbose logging |

### Test-ParallelAutomation.ps1

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `InstanceCount` | int | 4 | Number of instances to test |
| `TestDurationSeconds` | int | 60 | Duration of the test in seconds |
| `McpUrl` | string | http://127.0.0.1:8765 | Base URL of MCP server |
| `Verbose` | switch | off | Enable verbose logging |
| `SkipMcpCheck` | switch | off | Skip MCP health check |

### Verify-GameState.ps1

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `InstanceCount` | int | 4 | Number of instances to verify |
| `CaptureIntervalSeconds` | int | 5 | Interval between screenshots |
| `TestDurationSeconds` | int | 60 | Total test duration |
| `McpUrl` | string | http://127.0.0.1:8765 | Base URL of MCP server |
| `OutputDir` | string | docs/automation/screenshots | Directory for screenshots |
| `Verbose` | switch | off | Enable verbose logging |

## Exit Codes

### Test-ParallelAutomation.ps1

| Code | Meaning |
|------|---------|
| 0 | Success (95%+ success rate) |
| 1 | Degraded (80-95% success rate) |
| 2 | Failed (<80% success rate) |

## CI/CD Integration

The `.github/workflows/game-automation.yml` workflow automatically runs:

- **On every push to main** with changes to `src/` or `scripts/automation/`
- **Daily at 2 AM UTC** (scheduled)
- **On demand** (workflow_dispatch with instance/duration inputs)

### Workflow Matrix

Tests run in parallel with different instance counts:

```yaml
strategy:
  matrix:
    instances: [1, 2, 4]
```

### Artifacts

Each run generates:
- **Screenshots**: `game-automation-screenshots-{N}-instances/`
- **Logs**: `game-automation-logs-{N}-instances/`
- **Test results**: Available in workflow summary

## Troubleshooting

### MCP Server Not Responding

**Problem**: "Cannot reach MCP server at http://127.0.0.1:8765"

**Solution**:
```powershell
# Start MCP in a new terminal
.\scripts\start-mcp.ps1

# Verify it's running
$health = Invoke-WebRequest -Uri "http://127.0.0.1:8765/health"
$health.StatusCode  # Should be 200
```

### All Game Instances Exit Immediately

**Problem**: "All instances exited immediately"

**Solution**:
1. Check that the game is installed: `Test-Path "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe"`
2. Check BepInEx is installed in the game directory
3. Check for DLLs in use: Some Windows antivirus software locks game files during initial launch. Exclude the game directory from real-time scanning.

### Low Success Rate (<80%)

**Problem**: Tests are failing intermittently

**Solution**:
1. Increase test duration: `Test-ParallelAutomation.ps1 -TestDurationSeconds 120`
2. Check MCP logs: `tail -50 src/Tools/DinoforgeMcp/mcp.log`
3. Verify game instances are stable: `Get-Process "Diplomacy is Not an Option" | Select -First 2`
4. Check for missing DLL deps: `dotnet test src/Tests/ --filter "MCP"`

### Screenshots Not Saving

**Problem**: "Screenshots saved to: (empty directory)"

**Solution**:
1. Verify output directory exists: `Test-Path docs/automation/screenshots`
2. Check disk space: `Get-Volume | Where {$_.DriveLetter -eq 'G'}`
3. Check MCP game_screenshot implementation: `Get-Process | Where {$_.Name -like "*bare-cua*"}`

## Performance Baselines

Typical performance on Windows 11 (i7-13700K, 32GB RAM, NVMe):

| Metric | 1 Instance | 2 Instances | 4 Instances |
|--------|-----------|-----------|-----------|
| Launch time | ~5s | ~8s | ~15s |
| Test duration (60s) | 180 tests | 360 tests | 720 tests |
| Success rate | 99%+ | 98%+ | 95%+ |
| Avg test latency | 200ms | 300ms | 450ms |

## Advanced Usage

### Custom Test Sequences

You can extend the automation suite by adding custom test steps:

```powershell
# Example: Custom test with entity spawning
function Test-EntitySpawn {
    param([int]$InstanceId)

    $body = @{
        jsonrpc = "2.0"
        method = "game_query_entities"
        params = @{ component_type = "Health"; limit = 100 }
        id = 1000
    } | ConvertTo-Json

    Invoke-WebRequest -Uri "http://127.0.0.1:8765/api/tools/game_query_entities" `
        -Method POST -Body $body -ContentType "application/json"
}

Test-EntitySpawn -InstanceId 1
```

### Integration with Custom CI/CD

To integrate with your own CI/CD system:

```bash
#!/bin/bash
# Run in CI/CD pipeline

cd /path/to/DINOForge

# Start MCP (background)
powershell -c ".\scripts\start-mcp.ps1" &
sleep 5

# Run tests
powershell -c ".\scripts\automation\Test-ParallelAutomation.ps1 -InstanceCount 4 -TestDurationSeconds 120"
TEST_RESULT=$?

# Check results
if [ $TEST_RESULT -eq 0 ]; then
    echo "✓ Parallel automation tests passed"
    exit 0
else
    echo "✗ Parallel automation tests failed"
    exit 1
fi
```

## See Also

- [Game Automation MCP Tools](https://github.com/KooshaPari/Dino/blob/main/README.md#mcp-bridge)
- [MCP Server Documentation](https://github.com/KooshaPari/Dino/blob/main/src/Tools/DinoforgeMcp/README.md)
- [GitHub Actions Workflow](https://github.com/KooshaPari/Dino/blob/main/.github/workflows/game-automation.yml)
- [DINO Runtime Facts](https://github.com/KooshaPari/Dino/blob/main/docs/DINO_RUNTIME_FACTS.md)
