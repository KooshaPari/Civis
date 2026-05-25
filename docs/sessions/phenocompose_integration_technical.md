# Phenocompose Integration Technical Details

## Quick Reference: What You Found

### Repository Map
```
phenocompose (GitHub: KooshaPari/phenocompose)
├── Contains: nanovms (Go) + pheno-compose-driver (Rust) + full integration
├── Does NOT contain: Separate "nvms" repo (merged, doesn't exist separately)
├── License: Apache-2.0
└── Latest Activity: 2026-04-23, active development
```

---

## Architecture Deep Dive

### 3-Tier Isolation Model

**Tier 1: WASM Runtime**
- Runtime: Wasmtime (Rust-based, production-grade)
- Startup: ~1ms (instant)
- Isolation: Language-level (WASM sandbox)
- Use: Fast, trusted tools; lightweight utilities
- CPU/Memory: Minimal (<10MB)
- Example: CLI helpers, data transformations

**Tier 2: gVisor Containers**
- Runtime: gVisor runsc (Google, syscall isolation)
- Startup: ~90ms (acceptable for parallel tests)
- Isolation: Kernel syscall filtering + namespace
- Use: Browser automation, CI/CD containers, semi-trusted workloads
- CPU/Memory: Medium (100-500MB per container)
- Example: Selenium tests, build agents
- Advantage: Can run 50-100 in parallel on consumer hardware

**Tier 3: Firecracker MicroVMs**
- Runtime: Firecracker (AWS, KVM-based)
- Startup: ~125ms (full VM)
- Isolation: Hardware VM boundary
- Use: Full OS, game automation, untrusted code
- CPU/Memory: Full allocation (4GB-16GB per VM)
- Example: Windows game instance, full test environment
- Advantage: GPU passthrough support (VFIO)

### Platform Adapters

| OS | Tier 1 | Tier 2 | Tier 3 | Notes |
|----|--------|--------|--------|-------|
| **Linux** | Native Wasmtime | gVisor native | Firecracker + KVM | Tier 0 (bare metal) via VFIO |
| **macOS** | Native Wasmtime | gVisor via Lima | Virtualization.framework | Colima for development |
| **Windows** | Native Wasmtime | WSL2 gVisor | WSL2 Firecracker | Requires WSL2 installed |

### Hexagonal Architecture (Internal)

```go
// ports/ports.go
type VMAdapter interface {
    CreateVM(ctx context.Context, cfg VMConfig) (*VM, error)
    StartVM(ctx context.Context, id string) error
    StopVM(ctx context.Context, id string) error
    DeleteVM(ctx context.Context, id string) error
    SnapshotVM(ctx context.Context, id string, name string) error
    RestoreVM(ctx context.Context, id string, snapshot string) error
}

type SandboxAdapter interface {
    CreateContainer(ctx context.Context, cfg ContainerConfig) (*Container, error)
    ExecContainer(ctx context.Context, id string, cmd []string) (output []byte, error)
    // ...
}

type RuntimeAdapter interface {
    Deploy(ctx context.Context, modules []Module) error
    // ...
}

// Implementations:
// adapters/linux/     -> Native KVM + gVisor + Wasmtime
// adapters/mac/       -> Lima/Colima + Virtualization.framework
// adapters/windows/   -> WSL2 + Hyper-V
// adapters/wasm/      -> Wasmtime
// adapters/sandbox/   -> gVisor/runsc wrapper
```

---

## Game Automation Workflow (Key to DINOForge Integration)

### Documented in: `docs/journeys/game-automation.md`

#### Phase 1: Setup GPU Passthrough
```bash
# Check IOMMU
dmesg | grep -i iommu

# Bind GPU to vfio-pci (makes it available for VMs)
nanovms vfio bind --gpu=01:00.0

# Now VM can use this GPU directly
```

#### Phase 2: Create Game Template
```yaml
# game-vm-template.yaml
name: windows-steam-base
flavor: vfio                    # GPU passthrough flavor
resources:
  cpus: 8
  memory: 32GB
  gpu: 01:00.0                  # Bound GPU from Phase 1

software:
  - steam
  - steamcmd
  - bepinex                     # DINOForge mod loader!

looking_glass:
  enabled: true                 # Low-latency remote display
  ivshmem_size: 256MB
```

```bash
nanovms vm create --template game-vm-template.yaml
# Creates 45GB compressed template
```

#### Phase 3-4: Install Game + BepInEx Mods
```bash
# Install game via Steam
nanovms game install \
  --vm windows-steam-base \
  --appid <DINO_APP_ID> \
  --steam-token $STEAM_TOKEN

# Upload DINOForge BepInEx plugins
nanovms vm upload \
  --vm windows-steam-base \
  --local ./BepInEx/ \
  --remote "C:\\Games\\MyGame\\"

# Verify
nanovms vm exec windows-steam-base -- \
  powershell "Get-ChildItem 'C:\\Games\\MyGame\\BepInEx\\plugins'"
```

#### Phase 5: Create Snapshot (CRITICAL)
```bash
# Create baseline snapshot
nanovms vm snapshot windows-steam-base --name game-ready

# Snapshot details:
# - Base layer: 45GB (Windows + Steam + game)
# - Diff layer: 2GB (BepInEx config)
# - Format: zstd compressed
# - Clone time: <2s (instant VM creation from snapshot)
```

#### Phase 6: Launch Test Fleet
```bash
# Launch 100 parallel game instances from snapshot
nanovms game test \
  --snapshot game-ready \
  --count 100 \
  --test-suite ./automated-tests/ \
  --parallel 10 \
  --output ./results/

# Timeline:
# 0-15s: Spawn all 100 VMs from snapshot
# 15-120s: Run automated tests in parallel
# 120s+: Collect results, generate report
```

### Why This Matters for DINOForge

1. **Instant Test Environment**: Clone from snapshot in <2s (vs. manual 5-10 min setup)
2. **Parallel Testing**: 100 VMs running mod packs simultaneously (impossible with serial launches)
3. **GPU Rendering Tests**: Direct GPU access via VFIO for visual asset validation
4. **Deterministic Snapshots**: Same baseline for all tests (no drift)
5. **BepInEx Native**: phenocompose explicitly supports BepInEx in templates

---

## FFI Bindings (For Advanced Integration)

phenocompose provides FFI bindings to use nanovms from multiple languages:

### Rust FFI (pheno-compose-driver)
```rust
// From pheno-compose-driver/src/lib.rs
use nvms_ffi::*;

#[derive(Serialize)]
pub struct VMConfig {
    pub name: String,
    pub flavor: VMFlavor,
    pub resources: ResourceConfig,
}

pub fn create_vm(config: VMConfig) -> Result<VM, Error> {
    // Calls into Go nanovms core via FFI
    unsafe {
        nvms_create_vm(config)
    }
}

// Available as:
// - Rust library: pheno-compose-driver 0.1.0
// - Static library: libpheno_compose_driver.a
// - Dynamic library: libpheno_compose_driver.so / .dll
// - C headers: bindings/rust-ffi/nvms.h
```

### C FFI (Go Export)
```c
// From bindings/go-c-export/nvms_core.go
// Generates C headers + shared library

#include "nvms_core.h"

nvms_vm_t *vm = nvms_create_vm("game-vm", NVMS_FLAVOR_FIRECRACKER);
nvms_start_vm(vm);
nvms_snapshot_vm(vm, "baseline");
nvms_delete_vm(vm);
```

### Current Bindings
- ✅ Go ↔ Rust (via nvms-ffi crate)
- ✅ Go ↔ C (via cgo, headers in bindings/go-c-export)
- ✅ Rust ↔ C (via libnvms-sys)
- ✅ Go ↔ Zig (recent: 9ee0908)
- ✅ Go ↔ Mojo (recent: ebb5744, GPU ML workloads)

**For DINOForge**: If you ever embed game automation in C#, you could P/Invoke the C library directly. But CLI wrapper is simpler for now.

---

## CLI Commands Available

```bash
# VM Management
nanovms vm create --template template.yaml        # Create from template
nanovms vm start <vm-id>                         # Start VM
nanovms vm stop <vm-id>                          # Stop VM
nanovms vm delete <vm-id>                        # Delete VM
nanovms vm list                                  # List all VMs
nanovms vm exec <vm-id> -- <command>             # Run command in VM
nanovms vm upload --vm <id> --local X --remote Y # Upload files
nanovms vm download --vm <id> --remote X --local Y # Download files

# Snapshots
nanovms vm snapshot <vm-id> --name <name>        # Create snapshot
nanovms vm restore <vm-id> --from <snapshot>     # Restore from snapshot
nanovms snapshot list                            # List snapshots

# Game Automation (Key)
nanovms game install --vm <id> --appid <id> --steam-token $TOKEN
nanovms game import --vm <id> --archive backup.tar.zst
nanovms game test --snapshot <name> --count 100 --test-suite ./tests/
nanovms game uninstall --vm <id>

# GPU Passthrough
nanovms vfio list                                # List GPUs available for VFIO
nanovms vfio bind --gpu <pci-id>                 # Bind GPU to VFIO
nanovms vfio unbind --gpu <pci-id>               # Unbind GPU from VFIO

# Networking
nanovms network create <name>                    # Create virtual network
nanovms network attach <vm-id> <network>         # Attach VM to network
nanovms network port-forward <vm-id> --from 8080 --to 80

# Pool Management (Tier 2/3)
nanovms pool create --size 50 --flavor gvisor    # Create container pool
nanovms pool scale --name <pool> --to 100        # Scale pool
nanovms pool deploy --pool <name> --image <img>  # Deploy to pool

# WASM Runtime (Tier 1)
nanovms wasm run <module.wasm>                   # Run WASM module
nanovms wasm deploy --module <m> --to <pool>     # Deploy to WASM tier
```

---

## Comparison: phenocompose vs. Current DINOForge Game Automation

### Current DINOForge (v0.23)
```
MCP Tool: game_launch_test(hidden=True)
    ↓
Python: src/Tools/DinoforgeMcp/isolation_layer.py
    ↓
HiddenDesktopBackend: Win32 CreateDesktopW()
    ↓
Single isolated desktop with one game instance
```

**Limitations**:
- Only 1 TEST instance at a time
- No GPU passthrough (CreateDesktop is display-only)
- No persistent snapshots
- No cross-platform (Windows-only)

### With phenocompose Integration
```
MCP Tool: game_launch_fleet(count=100)
    ↓
Python wrapper: calls nanovms CLI
    ↓
phenocompose-driver: Rust FFI layer (optional, for direct API)
    ↓
nanovms Go core: Firecracker orchestration
    ↓
100 parallel game VMs (on Linux/macOS) or WSL2 (Windows)
    + GPU passthrough for rendering tests
    + Persistent snapshots
    + Cross-platform support
```

**Advantages**:
- 100x parallel capacity
- GPU-accelerated rendering tests
- Instant VM cloning from snapshot
- Works on Linux, macOS, Windows (WSL2)
- Persistent fleet management

---

## Dependency Analysis

### What phenocompose Requires

**Go 1.23+**
```bash
# Check DINOForge's go usage
grep -r "go 1." $(find . -name "go.mod")
# (Likely: none, DINOForge is .NET 11)
```

**System Dependencies**
```
Linux:
  - Linux 5.10+ (for Firecracker)
  - KVM enabled (vmx/svm in CPU flags)
  - gVisor runsc binary (for Tier 2)
  - Wasmtime runtime (for Tier 1)

macOS:
  - Xcode Command Line Tools
  - Lima or Colima (for Firecracker, Tier 2/3)
  - Virtualization.framework (10.15+)

Windows:
  - WSL2 installed and configured
  - Windows 11 or Windows Server 2022
  - Hyper-V enabled (or WSL2 hypervising)
```

**Optional Dependencies (for game automation)**
```
- Steam (for game installation)
- SteamCMD (for headless game server setup)
- BepInEx (already used by DINOForge!)
- Looking Glass (optional, for low-latency remote display)
- VFIO kernel modules (for GPU passthrough; Linux only)
```

### No Conflicts with DINOForge
- DINOForge: .NET 11, C#, BepInEx plugin
- phenocompose: Go, Rust, system-level orchestration
- **Coexistence**: Perfect; different layers

---

## Installation Path

### Option 1: As External CLI (Simplest)
```powershell
# In DINOForge installer
curl https://get.nvms.dev | sh
# or
wget -O nanovms.exe https://releases.nvms.dev/windows/nanovms-latest.exe
```

### Option 2: Build from Source
```bash
git clone https://github.com/KooshaPari/phenocompose.git
cd phenocompose
go build ./cmd/nanovms
# Output: ./nanovms executable
```

### Option 3: Homebrew / Package Manager
```bash
brew install nanovms  # If published to homebrew-core
# or
choco install nanovms # Windows Chocolatey
```

---

## Next Steps to Evaluate

1. **Test on your system**
   ```bash
   # Clone phenocompose
   git clone https://github.com/KooshaPari/phenocompose.git /tmp/phenocompose-eval
   
   # Check prerequisites
   go version  # Need 1.23+
   uname -a    # Linux/macOS
   # or
   wsl --list -v  # Windows
   
   # Try building
   cd /tmp/phenocompose-eval
   go build ./cmd/nanovms
   ```

2. **Review docs/journeys/game-automation.md** in phenocompose
   - Read the full journey (60KB+ of detailed steps)
   - Identify what maps to DINOForge workflows

3. **Check for integration examples**
   - Look at integrations/ folder in phenocompose
   - See if there are MCP examples already

4. **Plan MCP wrapper**
   - Design: What MCP tools would wrap nanovms commands?
   - Example: `game_launch_fleet(count: int, snapshot: string)`

---

## Summary: The Clear Picture

**phenocompose** = Unified game automation + isolated environments platform
- NVMS (nanovms) inside it = 3-tier orchestration engine
- BytePort/NVMS bits = Already merged (no separate repo)
- PhenoCompose driver = Rust FFI layer

**For DINOForge**: Adopt phenocompose as external dependency for parallel game testing. No code changes needed; just wrap CLI in MCP tools.

