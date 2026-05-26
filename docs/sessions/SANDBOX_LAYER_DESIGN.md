# DINOForge Sandbox / Micro-VM Layer Design

**Document type**: Architecture design
**Date**: 2026-03-25
**Status**: Specification (not yet implemented)
**Scope**: bare-cua sandbox layer for replication testing and CI/CD isolation
**Repo path**: `C:\Users\koosh\bare-cua\sandbox\`

---

## 1. Purpose

This layer is the **optional isolated environment** path in bare-cua. The primary automation path is still bare-metal (direct game control). The sandbox layer exists for:

- **Replication testing**: install DINO from Steam inside a clean environment, inject DINOForge mods, run the full test suite against a known-good baseline
- **CI/CD pipelines**: give each pipeline run a fresh, untouched game state so test results are reproducible and not contaminated by previous runs
- **Multiple concurrent agent instances**: run several isolated agent sessions simultaneously without game state cross-contamination

The design philosophy mirrors Docker: declarative `Sandboxfile.yaml` describes the environment, a Python class manages its lifecycle, and a `Computer` handle provides the connection to the running instance.

---

## 2. Research Findings

### 2.1 Windows Sandbox (WSB) File Format

A `.wsb` file is standard XML with `&lt;Configuration&gt;` root. It is fully programmatically generatable — no GUI required.

**All supported configuration elements** (Windows 10 build 18342+, fully documented Windows 11 24H2):

| XML Element | Values | Default | Notes |
|---|---|---|---|
| `&lt;MemoryInMB&gt;` | integer | 4096 | OS enforces minimum 2048 MB |
| `<vGPU>` | Enable / Disable / Default | Default (= Enabled on non-ARM64) | Shares host GPU via WDDM GPU-V |
| `&lt;Networking&gt;` | Enable / Disable / Default | Default (= Enabled) | NAT via Hyper-V Default Switch |
| `&lt;AudioInput&gt;` | Enable / Disable / Default | Default (= Enabled) | Shares host mic |
| `&lt;VideoInput&gt;` | Enable / Disable / Default | Default (= Disabled) | Shares host webcam |
| `&lt;ProtectedClient&gt;` | Enable / Disable / Default | Default (= Disabled) | AppContainer on RDP session |
| `&lt;PrinterRedirection&gt;` | Enable / Disable / Default | Default (= Disabled) | Share host printers |
| `&lt;ClipboardRedirection&gt;` | Enable / Disable / Default | Default (= Enabled) | Bidirectional clipboard |
| `&lt;MappedFolders&gt;` | array of `&lt;MappedFolder&gt;` | none | HostFolder must already exist |
| `&lt;LogonCommand&gt;/&lt;Command&gt;` | string | none | Runs after logon, as WDAGUtilityAccount |

**Is it "dockerfile-like"?** Partially. A `.wsb` file specifies the _environment_ (resources, mounts, startup command) but does not express _build steps_ or _layered images_. Our `Sandboxfile.yaml` format (Section 4) adds that layer on top.

**Key constraint**: Windows Sandbox always starts from a fresh copy of the installed Windows. There is no image layering, no persistence between sessions (unless via a mapped write-enabled folder), and no way to pre-bake installed software into the image.

### 2.2 Hyper-V PowerShell Management

Hyper-V VMs can be created, started, stopped, and destroyed entirely via PowerShell with no GUI:

```powershell
# Create
New-VM -Name "dino-test" -MemoryStartupBytes 8GB -Generation 2 `
    -NewVHDPath "C:\VMs\dino-test.vhdx" -NewVHDSizeBytes 120GB `
    -SwitchName "Default Switch"

# Start / Stop
Start-VM -Name "dino-test"
Stop-VM  -Name "dino-test" -Force

# Checkpoint (snapshot)
Checkpoint-VM -Name "dino-test" -SnapshotName "post-install"
Restore-VMSnapshot -VMName "dino-test" -Name "post-install"

# Destroy
Remove-VM -Name "dino-test" -Force
```

**Differencing disks** allow a single base VHDX to be shared across many VMs with per-VM delta files — ideal for CI where each run needs a fresh copy but the 6GB DINO install should not be re-downloaded each time:

```powershell
New-VHD -Path ".\run-001-diff.vhdx" -ParentPath ".\dino-base.vhdx" -Differencing
```

**Requirement**: Hyper-V module requires Windows Pro/Enterprise. Must run as Administrator.

### 2.3 Windows Containers and GPU

**Verdict**: Windows containers are not viable for DINO testing.

- GPU acceleration in Windows containers requires **process isolation** (`--isolation=process`). Hyper-V isolation mode (`--isolation=hyperv`) does not support GPU.
- Process isolation only works when container OS version exactly matches host OS version (strict kernel version matching).
- DirectX is the only supported API (no OpenGL, Vulkan, CUDA).
- Windows Server containers have no display stack — no way to render a game window at all.
- Windows Server 2025 adds GPU partitioning, but this is for compute workloads, not interactive game rendering.

**Conclusion**: Docker Windows containers cannot run DINO. Use WSB or Hyper-V VM instead.

### 2.4 MSIX / Win32 App Isolation

MSIX packaging provides lightweight filesystem and registry virtualization (AppContainer-based) without a full VM. Key characteristics:

- **Startup time**: ~1-2 seconds (no VM boot)
- **GPU**: full host GPU access (no virtualization overhead)
- **Isolation level**: filesystem/registry redirection, not kernel isolation
- **Use case**: appropriate for isolating app _state_, not for clean OS environments
- **Limitation for CI**: MSIX containers share the host kernel and cannot give you a "clean" Windows install. A broken game install would contaminate subsequent runs unless carefully reset.
- **Not recommended** for replication testing. Appropriate only for state isolation between runs on a known-good base.

### 2.5 Named Pipes Across Sandbox Boundary

**Named pipes do NOT cross the Windows Sandbox boundary.**

Windows Sandbox runs inside a Hyper-V lightweight VM. Named pipes (`\\.\pipe\NAME`) are scoped to the Windows object namespace of the machine/session they are created in. The sandbox has a separate object namespace from the host.

This is confirmed by the fact that CVE-2022-22715 was a vulnerability specifically because predictable named pipe names were accessible across certain sandbox boundaries — but Windows Sandbox uses a different isolation model (full Hyper-V kernel boundary).

**Viable IPC strategies for WSB:**

1. **TCP over Hyper-V Default Switch** (recommended, requires `networking=Enable`)
   - Sandbox gets a NAT'd IP (typically `172.x.x.x`)
   - Host gateway is the `.1` on the same subnet (accessible from sandbox to host, not directly from host to sandbox without knowing the sandbox IP)
   - Startup script discovers sandbox IP via `Get-NetIPAddress`, writes it to a shared mapped folder file
   - Python host reads that file, then connects to `sandbox_ip:port` via TCP
   - bare-cua-native runs as a TCP listener inside the sandbox

2. **Shared mapped folder file-drop** (fallback, works with `networking=Disable`)
   - Use a read-write `&lt;MappedFolder&gt;` as a shared message queue
   - Python host writes `cmd_<id>.json` to shared dir
   - Agent inside sandbox polls for new files, executes command, writes `res_<id>.json`
   - Simple, reliable, ~0.5s latency per command

3. **What about loopback (127.0.0.1)?** Loopback does NOT cross VM boundaries. `127.0.0.1` from inside the sandbox refers to the sandbox's loopback, not the host. Cannot use this for host-to-sandbox communication.

### 2.6 GPU Support by Approach

| Approach | GPU Mechanism | DirectX Support | Performance |
|---|---|---|---|
| Windows Sandbox | WDDM GPU-V (shared kernel) | Yes (DirectX 11/12) | Adequate for UI automation; 30-60% of native for 3D |
| Hyper-V (Win11 client) | GPU-PV via Add-VMGpuPartitionAdapter | Yes | Similar to WSB (WDDM shared) |
| Hyper-V (Server 2025) | GPU-P (SR-IOV, hardware partitioned) | Yes | Near-native per partition |
| Hyper-V DDA | Direct Device Assignment (full GPU) | Yes | Native performance, 1 VM per GPU |
| Docker Windows | Process isolation only | DirectX only, no render window | Not viable for game |
| WSL2 | GPU via WSLg (D3D12 driver) | DirectX via D3D12 | Linux only, DINO does not run on Linux |

For DINO automated testing (capturing screenshots, checking ECS state via bridge): **WSB vGPU is adequate**. DINO's Burst/ECS simulation runs on CPU; GPU is only needed for rendering the game window for screenshot capture.

### 2.7 SteamCMD Credential Handling

SteamCMD (non-interactive game installation):

```
steamcmd +login <user> <password> +force_install_dir <path> +app_update 1273720 validate +quit
```

**Credential strategies** (in order of CI suitability):

1. **Pre-cached credentials** (best for CI):
   - Log in interactively once on a dedicated CI Steam account
   - SteamCMD stores `ssfn*` files in `steamcmd/config/`
   - Map that config dir read-only into the sandbox
   - Subsequent logins use: `+login <username>` (no password)
   - Cache stays valid for days on the same machine/IP

2. **Email Steam Guard** (workable for first-time setup):
   - Provide username + password; Steam sends email code on first login
   - After entering the code once, credentials are cached for that machine/IP
   - Automated: intercept the email programmatically if using a dedicated account

3. **Mobile TOTP Steam Guard** (most common, hardest to automate):
   - Requires the `shared_secret` from the Steam account
   - Tool: [`steamcmd-2fa`](https://github.com/Weilbyte/steamcmd-2fa) generates TOTP codes non-interactively given the shared secret
   - Store shared secret in an environment variable or Windows Credential Manager
   - Use: `steamcmd-2fa --shared-secret $env:STEAM_SHARED_SECRET -- +login ...`

4. **Windows Credential Manager** (secure storage):
   ```powershell
   # Store once:
   cmdkey /generic:SteamCMD /user:myaccount /pass:mypassword
   # Retrieve in script:
   $cred = Get-StoredCredential -Target SteamCMD  # requires CredentialManager module
   ```

**Practical recommendation for DINOForge CI**: Maintain a dedicated Steam account that owns DINO. Log in once interactively, copy the `config/` directory, and map it read-only into each sandbox session. No password or 2FA code needed for subsequent runs on the same host.

---

## 3. Isolation Technology Comparison

| Approach | Startup time | GPU support | Concurrent instances | Steam install | Network to host | Persistence | Cost |
|---|---|---|---|---|---|---|---|
| **Windows Sandbox (WSB)** | 5-10s | vGPU (WDDM shared, ~60% perf) | **1 at a time** per host | Yes via steamcmd | TCP over NAT / file-drop | None (disposable) | Free (built-in Pro+) |
| **Hyper-V VM** | 30-60s | GPU-PV (Win11 shared) / GPU-P (Server 2025) | Multiple | Yes | TCP over vSwitch | Snapshots available | Free (built-in Pro+) |
| **Docker Windows containers** | 2-5s | **None** (process isolation only, no display) | Multiple | No (no interactive display) | Yes | Layer-based | Free |
| **WSL2** | 2-3s | GPU via WSLg (compute only) | Multiple | No (DINO is Windows-only) | Yes | Persistent | Free |
| **MSIX app isolation** | ~1-2s | Full native GPU | Multiple | N/A | Full | State reset only | Free |
| **Hyper-V + GPU-P (Server 2025)** | 30-60s | Hardware-partitioned GPU (near-native) | Multiple | Yes | TCP | Snapshots | Windows Server license |

**For DINOForge automated testing (current recommendation):**
- **WSB** for single-run smoke tests and CI (simple, disposable, no base image required)
- **Hyper-V VM with differencing disk** for parallel test matrix or persistent environments
- **MSIX** is not suitable (no clean OS environment)
- **Docker** is not suitable (no game window rendering)

---

## 4. Sandboxfile Format Specification

A `Sandboxfile.yaml` is the declarative specification for a reproducible sandbox environment. It serves the same role as a `Dockerfile` — a human-readable, version-controllable description of a test environment.

### 4.1 Full Schema

```yaml
# Sandboxfile.yaml — DINOForge replication test environment

name: dino-test-env              # Identifier; used for .wsb filename and VM name
base: windows-sandbox            # Backend: windows-sandbox | hyperv-vm | docker-windows

# Resource allocation
memory_mb: 8192                  # MB; minimum 2048 for WSB
virtual_gpu: true                # Share host GPU (WSB: WDDM GPU-V; Hyper-V: GPU-PV)
networking: true                 # NAT network (required for TCP IPC)
cpu_count: 4                     # CPU count (Hyper-V only; WSB ignores this)

# Folder mappings (evaluated before LogonCommand)
map:
  - host: "G:\\SteamLibrary\\steamapps\\common\\Diplomacy is Not an Option"
    sandbox: "C:\\DINO"
    readonly: true

  - host: "C:\\Users\\koosh\\Dino\\src\\Runtime\\bin\\Release\\net472"
    sandbox: "C:\\DINO\\BepInEx\\plugins"
    readonly: false

  - host: "C:\\Users\\koosh\\Dino"
    sandbox: "C:\\DINOForge"
    readonly: true

  - host: "C:\\Users\\koosh\\steamcmd_cache"   # pre-cached SteamCMD credentials
    sandbox: "C:\\SteamCmdCache"
    readonly: true

# Setup steps executed in order after sandbox boots
# Types: run | wait_for_file | wait_for_process | copy
setup:
  - run: "powershell -ExecutionPolicy Bypass -File C:\\SandboxInit\\setup_bepinex.ps1"
  - wait_for_file: "C:\\SandboxShared\\bepinex_ready.flag"
    timeout_s: 60

# Health check: how Python host knows sandbox is ready
health_check:
  type: file                     # file | process | tcp | pipe
  path: "C:\\SandboxShared\\ready.flag"
  timeout_s: 120

# Environment variables injected before setup steps
env:
  DINOFORGE_ENV: sandbox
  LOG_LEVEL: debug

# Test commands to run (via computer.run_tests())
test:
  - run: "dotnet test C:\\DINOForge\\src\\Tests --verbosity normal"
  - run: "dotnet test C:\\DINOForge\\src\\Tests\\Integration"
```

### 4.2 Minimal Sandboxfile (read-only test against pre-installed game)

```yaml
name: dino-smoke-test
base: windows-sandbox
memory_mb: 4096
virtual_gpu: true
networking: true

map:
  - host: "G:\\SteamLibrary\\steamapps\\common\\Diplomacy is Not an Option"
    sandbox: "C:\\DINO"
    readonly: true
  - host: "C:\\Users\\koosh\\Dino"
    sandbox: "C:\\DINOForge"
    readonly: true

setup:
  - run: "powershell -ExecutionPolicy Bypass -File C:\\DINOForge\\sandbox\\templates\\setup_bepinex.ps1"

health_check:
  type: file
  path: "C:\\SandboxShared\\ready.flag"
  timeout_s: 90

test:
  - run: "dotnet test C:\\DINOForge\\src\\Tests"
```

### 4.3 Full replication Sandboxfile (installs DINO from Steam)

```yaml
name: dino-full-replication
base: windows-sandbox
memory_mb: 12288
virtual_gpu: true
networking: true

map:
  - host: "C:\\Users\\koosh\\Dino"
    sandbox: "C:\\DINOForge"
    readonly: true
  - host: "C:\\Users\\koosh\\steamcmd_cache"
    sandbox: "C:\\SteamCmdCache"
    readonly: true

setup:
  - run: "powershell -ExecutionPolicy Bypass -File C:\\DINOForge\\sandbox\\templates\\install_dino.ps1 -SteamUser ci-account -InstallPath C:\\DINO -PreCachedConfigDir C:\\SteamCmdCache"
  - wait_for_file: "C:\\SandboxShared\\ready.flag"
    timeout_s: 3600
  - run: "powershell -ExecutionPolicy Bypass -File C:\\DINOForge\\sandbox\\templates\\setup_bepinex.ps1 -GameDir C:\\DINO -DINOForgeRepo C:\\DINOForge"

health_check:
  type: file
  path: "C:\\SandboxShared\\ready.flag"
  timeout_s: 3700

test:
  - run: "dotnet test C:\\DINOForge\\src\\Tests"
  - run: "dotnet test C:\\DINOForge\\src\\Tests\\Integration"
```

---

## 5. Python API Design

### 5.1 Module Structure

```
bare-cua/sandbox/
    __init__.py          # Public exports
    config.py            # SandboxConfig, MappedFolder dataclasses + to_wsb_xml()
    sandboxfile.py       # Sandboxfile YAML parser + startup script renderer
    sandbox.py           # Sandbox + HyperVSandbox async context managers
    templates/
        setup_bepinex.ps1   # BepInEx install script for sandbox
        install_dino.ps1    # SteamCMD DINO install script
```

### 5.2 Public API

```python
from bare_cua.sandbox import Sandbox, Sandboxfile, SandboxConfig, MappedFolder

# --- Option A: from Sandboxfile.yaml (recommended) ---
async with Sandbox.from_sandboxfile("Sandboxfile.yaml") as computer:
    result = await computer.run("dotnet test C:\\DINOForge\\src\\Tests")
    assert result.success, result.error

# --- Option B: programmatic config ---
config = SandboxConfig(
    name="quick-test",
    memory_mb=4096,
    virtual_gpu="Enable",
    networking="Enable",
    mapped_folders=[
        MappedFolder(
            host_folder=r"C:\Users\koosh\Dino",
            sandbox_folder=r"C:\DINOForge",
            read_only=True,
        ),
    ],
    startup_script=r"C:\DINOForge\sandbox\templates\setup_bepinex.ps1",
)
async with Sandbox(config) as computer:
    result = await computer.run("dotnet test")

# --- Option C: run full test suite from Sandboxfile ---
sandbox = Sandbox.from_sandboxfile("Sandboxfile.yaml")
async with sandbox as computer:
    results = await sandbox.run_tests()
    failed = [r for r in results if not r.success]
    if failed:
        raise RuntimeError(f"{len(failed)} tests failed")

# --- Option D: Hyper-V for concurrent instances ---
from bare_cua.sandbox import Sandboxfile
from bare_cua.sandbox.sandbox import HyperVSandbox

sf = Sandboxfile.load("Sandboxfile.yaml")
async with HyperVSandbox(sf, base_vhdx=r"C:\HyperV\dino-base.vhdx") as hv:
    ip = await hv.get_ip()
    print(f"VM IP: {ip}")
```

### 5.3 SandboxConfig to .wsb XML

```python
from bare_cua.sandbox import SandboxConfig, MappedFolder

config = SandboxConfig(
    name="dino-test",
    memory_mb=8192,
    virtual_gpu="Enable",
    networking="Enable",
    mapped_folders=[
        MappedFolder(
            host_folder=r"G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option",
            sandbox_folder=r"C:\DINO",
            read_only=True,
        ),
    ],
    startup_command=r"powershell.exe -ExecutionPolicy Bypass -File C:\setup.ps1",
)

print(config.to_wsb_xml())
# -> <?xml version="1.0" encoding="utf-8"?>
# -> <Configuration>
# ->   <MemoryInMB>8192</MemoryInMB>
# ->   <vGPU>Enable</vGPU>
# ->   ...

config.write_wsb("dino-test.wsb")
```

### 5.4 Sandboxfile.render_startup_script()

The Sandboxfile renders a self-contained PowerShell script that:
1. Sets `$ErrorActionPreference = 'Stop'`
2. Executes each setup step in order
3. Handles `wait_for_file` and `wait_for_process` with configurable timeouts
4. Writes `ready.flag` on success
5. Writes `ready.flag.error` on failure (Python host reads this to raise RuntimeError)

The script is written to `%TEMP%\bare-cua-sandbox\setup.ps1` and mapped read-only into the sandbox as `C:\SandboxInit\setup.ps1`.

---

## 6. Lifecycle Diagram

```
Host (Python)                          Windows Sandbox (Hyper-V VM)
===================                    =====================================
Sandbox.__aenter__()
  |
  +-- Write setup.ps1 to TEMP
  +-- Inject shared_mf, init_mf
  +-- Write .wsb XML
  +-- subprocess.Popen(                 [sandbox boots]
  |     WindowsSandbox.exe, .wsb)       LogonCommand fires:
  |                                       powershell -File C:\SandboxInit\setup.ps1
  |                                         Step 1: Copy BepInEx
  |                                         Step 2: Copy DLLs
  |                                         ...
  +-- poll(shared/ready.flag) <---------  Set-Content C:\SandboxShared\ready.flag
  |   (every 2s)
  |
  +-- read(shared/sandbox_ip.txt) <-----  Write-NetIPAddress -> sandbox_ip.txt
  |
  +-- Computer(sandbox_ip, port=8765)     bare-cua-native --port 8765 (TCP listener)
  |
  return Computer

computer.run("dotnet test")
  |
  +-- TCP connect to sandbox_ip:8765 --> bare-cua-native receives JSON RPC
  +-- await readline()             <--   runs command, writes JSON result
  +-- CommandResult(...)

Sandbox.__aexit__()
  +-- computer.terminate()            -> WindowsSandbox.exe killed -> sandbox disposed
  +-- cleanup .wsb, setup.ps1
```

---

## 7. Enabling Windows Sandbox

Windows Sandbox is not enabled by default. Enable it with:

```powershell
# Enable (requires reboot)
Enable-WindowsOptionalFeature -FeatureName "Containers-DisposableClientVM" -All -Online

# Verify
Get-WindowsOptionalFeature -Online -FeatureName Containers-DisposableClientVM
# State: Enabled
```

Requirements:
- Windows 10 Pro/Enterprise build 18305 or Windows 11 Pro/Enterprise/Education
- AMD64 architecture (not ARM64 for vGPU)
- Virtualization enabled in BIOS
- Hyper-V enabled (WSB uses Hyper-V internally)

---

## 8. Key Limitations and Mitigations

### 8.1 WSB: 1 instance at a time

**Limitation**: Windows Sandbox is limited to one running instance per host on Windows 10/11.
**Mitigation**: Use `HyperVSandbox` for parallel test execution. Create N VMs from the same differencing disk base. Requires Windows Server for GPU-P; Windows 11 client supports GPU-PV (acceptable for UI automation).

### 8.2 WSB: No persistence

**Limitation**: Every sandbox session starts from a clean Windows image. Cannot persist installed software between sessions.
**Mitigation**: Use mapped read-only folders for large static content (DINO game files, DINOForge repo). Only dynamic state (BepInEx install, log files) lives inside the sandbox.

### 8.3 vGPU: Limited 3D performance

**Limitation**: WDDM GPU-V (used by both WSB and Hyper-V GPU-PV on Windows 11 client) shares the GPU kernel. Expected 3D performance is 40-70% of native.
**Mitigation**: For DINOForge testing, GPU is only needed to render the game window for screenshot capture. ECS simulation runs on CPU via Burst. The vGPU limitation is acceptable.
**For production**: Windows Server 2025 + GPU-P (SR-IOV) gives hardware-isolated GPU partitions at near-native performance.

### 8.4 SteamCMD: 2FA complications

**Limitation**: Steam Guard TOTP requires a code on every new machine/IP.
**Mitigation**:
1. Use a dedicated CI Steam account with email Steam Guard (simpler to automate)
2. Cache `ssfn*` credentials after first login and reuse via mapped folder
3. For TOTP: store `shared_secret` in environment and use `steamcmd-2fa` wrapper

### 8.5 SteamCMD: Game size (~6GB)

**Limitation**: DINO is ~6GB. Downloading inside a sandbox on every run is impractical.
**Mitigation**: Map the game directory read-only from the host. Only run SteamCMD install once on the host, then reuse via mapped folder. For true clean-room testing, use a pre-built Hyper-V base VHDX with DINO pre-installed.

### 8.6 Named pipes: No cross-sandbox IPC

**Limitation**: Named pipes do not cross the WSB boundary (separate Hyper-V object namespace).
**Mitigation**: TCP over the NAT interface (networking=Enable) or file-drop via shared mapped folder. Both are implemented in `Computer._run_tcp()` and `Computer._run_file_drop()`.

### 8.7 WSB: No LogonCommand timeout

**Limitation**: There is no built-in mechanism in WSB to kill or timeout a runaway startup script.
**Mitigation**: The Python `_wait_for_ready()` loop times out and calls `computer.terminate()` which kills the WindowsSandbox.exe process, which terminates the entire VM.

---

## 9. Future Extensions

1. **Snapshot-based testing with Hyper-V**: build a base VHDX with DINO + BepInEx pre-installed, snapshot it, restore per test run — eliminates install time entirely.
2. **CI integration**: GitHub Actions self-hosted runner on Windows + WSB or Hyper-V for game integration tests.
3. **Multi-sandbox orchestrator**: a `SandboxPool` class that manages N concurrent `HyperVSandbox` instances for parallel test matrix execution.
4. **Sandbox image builder**: a CLI command `bare-cua sandbox build Sandboxfile.yaml` that creates a base Hyper-V VHDX from a Sandboxfile (similar to `docker build`).
5. **Windows Server 2025 GPU-P**: when server hardware is available, upgrade to SR-IOV GPU partitioning for near-native performance in VMs.

---

## 10. File Index

| File | Purpose |
|---|---|
| `C:\Users\koosh\bare-cua\sandbox\__init__.py` | Public API exports |
| `C:\Users\koosh\bare-cua\sandbox\config.py` | `SandboxConfig`, `MappedFolder`, `.wsb` XML generator |
| `C:\Users\koosh\bare-cua\sandbox\sandboxfile.py` | `Sandboxfile` YAML parser, startup script renderer, Hyper-V script generator |
| `C:\Users\koosh\bare-cua\sandbox\sandbox.py` | `Sandbox`, `HyperVSandbox`, `Computer` async context managers |
| `C:\Users\koosh\bare-cua\sandbox\templates\setup_bepinex.ps1` | BepInEx install + bare-cua-native startup script |
| `C:\Users\koosh\bare-cua\sandbox\templates\install_dino.ps1` | SteamCMD DINO installation script with credential handling |

---

*Sources consulted:*
- [Windows Sandbox configuration (.wsb)](https://learn.microsoft.com/en-us/windows/security/application-security/application-isolation/windows-sandbox/windows-sandbox-configure-using-wsb-file)
- [GPU acceleration in Windows containers](https://learn.microsoft.com/en-us/virtualization/windowscontainers/deploy-containers/gpu-acceleration)
- [GPU Partitioning in Windows Server 2025 Hyper-V](https://techcommunity.microsoft.com/blog/itopstalkblog/gpu-partitioning-in-windows-server-2025-hyper-v/4429593)
- [Hyper-V PowerShell module](https://learn.microsoft.com/en-us/windows-server/virtualization/hyper-v/powershell)
- [SteamCMD — Valve Developer Community](https://developer.valvesoftware.com/wiki/SteamCMD)
- [steamcmd-2fa](https://github.com/Weilbyte/steamcmd-2fa)
- [GPU paravirtualization (Windows drivers)](https://learn.microsoft.com/en-us/windows-hardware/drivers/display/gpu-paravirtualization)
- [Easy-GPU-PV (GPU partitioning on Windows)](https://github.com/jamesstringer90/Easy-GPU-PV)
