# MCP Harness Service Setup

MCP should stay running in HTTP mode for persistent CC sessions and HMR workflows.

## Canonical launchers

- Windows: `scripts/start-mcp.ps1`
- Linux/macOS: `scripts/start-mcp.sh`

The service-manager files below should only wrap those launchers. That keeps host, port, health-check, PID-file, and working-directory logic in one place per platform.

## Windows (Task Scheduler, no extra dependencies)

Install the per-user scheduled task:

```powershell
pwsh -File scripts/services/windows/register-mcp-task.ps1 -Action Install
```
For a cross-platform harness install flow, use:

```powershell
pwsh -File scripts/services/mcp-service.ps1 -Action Install
```

That creates `DINOForge MCP` and runs:

```text
pwsh.exe -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File <repo>\scripts\start-mcp.ps1 -Action start -Detached
```

Task commands:

```powershell
pwsh -File scripts/services/windows/register-mcp-task.ps1 -Action Status
pwsh -File scripts/services/windows/register-mcp-task.ps1 -Action Start
pwsh -File scripts/services/windows/register-mcp-task.ps1 -Action Stop
pwsh -File scripts/services/windows/register-mcp-task.ps1 -Action Uninstall
```

Enable watcher mode automatically by setting `DINOFORGE_MCP_WATCH=1` before install, or pass `-Watch` during install.

## Linux (systemd user service)

1. Replace `__REPO_ROOT__` in `scripts/services/systemd/dinoforge-mcp.service` with the absolute repo path.
2. Copy the rendered file to `~/.config/systemd/user/dinoforge-mcp.service`.
3. Enable and start:

```bash
repo_root="$(pwd)"
sed "s|__REPO_ROOT__|$repo_root|g" scripts/services/systemd/dinoforge-mcp.service > ~/.config/systemd/user/dinoforge-mcp.service
systemctl --user daemon-reload
systemctl --user enable --now dinoforge-mcp.service
systemctl --user status dinoforge-mcp.service
```

Cross-platform install/uninstall:

```bash
pwsh -File scripts/services/mcp-service.ps1 -Action Install
pwsh -File scripts/services/mcp-service.ps1 -Action Status
pwsh -File scripts/services/mcp-service.ps1 -Action Stop
pwsh -File scripts/services/mcp-service.ps1 -Action Uninstall
```

Stop and remove:

```bash
systemctl --user disable --now dinoforge-mcp.service
rm ~/.config/systemd/user/dinoforge-mcp.service
```

## macOS (launchd)

1. Replace `__REPO_ROOT__` in `scripts/services/launchd/com.dinoforge.mcp.plist` with the absolute repo path.
2. Copy the rendered file to `~/Library/LaunchAgents/com.dinoforge.mcp.plist`.
3. Load it:

```bash
repo_root="$(pwd)"
sed "s|__REPO_ROOT__|$repo_root|g" scripts/services/launchd/com.dinoforge.mcp.plist > ~/Library/LaunchAgents/com.dinoforge.mcp.plist
launchctl unload ~/Library/LaunchAgents/com.dinoforge.mcp.plist 2>/dev/null || true
launchctl bootstrap gui/$UID ~/Library/LaunchAgents/com.dinoforge.mcp.plist
```

Cross-platform install/uninstall on macOS:

```bash
pwsh -File scripts/services/mcp-service.ps1 -Action Install
pwsh -File scripts/services/mcp-service.ps1 -Action Status
pwsh -File scripts/services/mcp-service.ps1 -Action Stop
pwsh -File scripts/services/mcp-service.ps1 -Action Uninstall
```

Stop and remove:

```bash
launchctl bootout gui/$UID ~/Library/LaunchAgents/com.dinoforge.mcp.plist 2>/dev/null || true
rm ~/Library/LaunchAgents/com.dinoforge.mcp.plist
```
