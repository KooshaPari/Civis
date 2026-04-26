# ADR-020: Multi-Instance Concurrency Architecture

**Date**: 2026-04-04
**Status**: Proposed
**Deciders**: DINOForge Architecture Team

## Context

DINOForge's testing capabilities are limited by the single-instance nature of most game testing. To enable parallel test execution, scenario comparison, and automated testing pipelines, we need the ability to run multiple game instances concurrently on a single machine.

Current limitations:
- Games typically use mutexes to prevent multiple instances
- Shared save files cause corruption
- Network port conflicts for MCP communication
- Resource contention (CPU, GPU, RAM)

## Decision Drivers

- **Test Parallelism**: Run 5-10 test scenarios simultaneously
- **A/B Testing**: Compare mod versions side-by-side
- **CI/CD Integration**: Automated test suites in CI pipelines
- **Resource Efficiency**: Maximize hardware utilization
- **Isolation**: Prevent cross-instance interference

## Options Considered

### Option A: Mutex Bypass Only

Bypass game-level duplicate instance detection through mutex/window renaming.

**Pros**:
- Simple implementation
- Low overhead per instance
- Works with existing game install

**Cons**:
- Shared save data (corruption risk)
- Shared config (unpredictable behavior)
- No true isolation
- Limited to 2-3 instances reliably

### Option B: Full Directory Isolation (Selected)

Create isolated game directories for each instance using junction points or full copies.

**Pros**:
- Complete save/config isolation
- Independent mod sets per instance
- Unlimited instance count (resource permitting)
- Clean separation enables debugging

**Cons**:
- Higher disk usage (~20GB per instance for DINO)
- More complex setup
- Longer initial launch time

**Implementation**:
```
Instance N Directory Structure:
DINO_Instance_N/
  ├── Game/                    # Junction to base game (saves space)
  ├── Saves/                   # Instance-specific
  ├── Config/                  # Instance-specific
  ├── BepInEx/
  │   ├── plugins/             # Mod DLLs (shared)
  │   └── dinoforge_packs/     # Pack directory (can be unique)
  └── MCP_Port: 8765 + N       # Dedicated MCP port
```

### Option C: Container/WSL Isolation

Run instances in Windows containers or WSL2.

**Pros**:
- True OS-level isolation
- Network namespace separation
- Resource limits (cgroups)

**Cons**:
- Graphics passthrough complexity
- Higher overhead
- Not all games work in containers
- Windows container ecosystem immature for gaming

### Option D: VM Per Instance

Full virtual machine per instance.

**Pros**:
- Maximum isolation
- Snapshot/restore capabilities

**Cons**:
- Extreme overhead (GBs of RAM per VM)
- GPU passthrough complexity
- Slow startup (minutes)
- Not feasible for parallel testing

## Decision

**Adopt Option B (Full Directory Isolation) with junction point optimization** for the primary multi-instance architecture.

### Implementation Details

| Component | Strategy |
|-----------|----------|
| **Game Files** | Junction points to base install (read-only) |
| **Saves** | Instance-specific subdirectories |
| **Config** | Instance-specific copies with modifications |
| **MCP Server** | Port allocation: 8765 + instance_id |
| **Process ID** | Tracked in instance manager |
| **Cleanup** | Automatic on graceful shutdown |

### Instance Manager Architecture

```
InstanceManager
├── CreateInstance(config)
│   ├── Allocate instance_id
│   ├── Create directory structure
│   ├── Setup junction points
│   ├── Configure MCP port
│   └── Return InstanceHandle
├── LaunchInstance(handle)
│   ├── Start game process
│   ├── Wait for MCP readiness
│   └── Return InstanceSession
├── MonitorInstance(handle)
│   ├── Health checks
│   ├── Resource monitoring
│   └── Crash detection
└── TerminateInstance(handle)
    ├── Graceful shutdown
    ├── Force kill if needed
    └── Cleanup directories
```

### Port Allocation Strategy

| Instance | MCP Port | Use |
|----------|----------|-----|
| 0 (Primary) | 8765 | Default game |
| 1 | 8766 | Test instance 1 |
| 2 | 8767 | Test instance 2 |
| N | 8765 + N | Instance N |

### Resource Limits

| Resource | Limit Strategy |
|----------|----------------|
| **CPU** | Process affinity (optional) |
| **GPU** | Frame rate limiting |
| **RAM** | Working set monitoring |
| **Disk** | Shared base, instance-specific deltas |

## Consequences

### Positive

- **Parallel Testing**: Run full test suites concurrently
- **Scenario Comparison**: Side-by-side mod comparisons
- **CI/CD Ready**: Headless test execution
- **Resource Efficiency**: Better hardware utilization
- **Isolation**: Clean separation prevents interference

### Negative

- **Disk Usage**: ~500MB per instance (configs + saves)
- **Setup Complexity**: More moving parts than single instance
- **GPU Limits**: Still limited by VRAM for multiple render contexts
- **Sync Complexity**: Coordinating multiple instances is harder

### Neutral

- **MCP Client Changes**: Must support multiple ports
- **Save Management**: Instance-specific saves need backup strategy

## Implementation Plan

### Phase 1: Foundation
- [ ] InstanceManager implementation
- [ ] Junction point utilities
- [ ] Port allocation service
- [ ] Basic create/terminate lifecycle

### Phase 2: Integration
- [ ] MCP multi-port support
- [ ] Desktop Companion instance browser
- [ ] Instance health monitoring
- [ ] Resource tracking

### Phase 3: Automation
- [ ] Parallel test execution
- [ ] Scenario recording/replay
- [ ] CI/CD GitHub Actions integration
- [ ] Automated cleanup policies

## Related ADRs

- ADR-013: Duplicate Instance Detection Bypass (prerequisite)
- ADR-018: Second Instance Bypass (prerequisite)
- ADR-022: MCP Orchestration Model (client-side changes)

## References

- Windows Junction Points: https://docs.microsoft.com/en-us/windows/win32/fileio/hard-links-and-junctions
- Process Isolation Patterns: Windows SDK
- DINOForge Concurrent Instances Guide: docs/CONCURRENT_INSTANCES.md
