# MCP Harness Service Setup

MCP should stay running in HTTP mode for persistent CC sessions and HMR workflows.

## Windows (Task Scheduler, no extra dependencies)

Run:

```powershell
pwsh -File scripts/services/windows/register-mcp-task.ps1 -Action Install
```

That creates a per-user scheduled task `DINOForge MCP` that runs `pwsh.exe` with a command equivalent to:

```text
pwsh.exe -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File "<repo>\scripts\start-mcp.ps1" -Action start -Detached
```

`<repo>` is expanded to your checked-out repository root at install time.

Task commands:

```powershell
pwsh -File scripts/services/windows/register-mcp-task.ps1 -Action Status
pwsh -File scripts/services/windows/register-mcp-task.ps1 -Action Start
pwsh -File scripts/services/windows/register-mcp-task.ps1 -Action Stop
pwsh -File scripts/services/windows/register-mcp-task.ps1 -Action Uninstall
```

Enable watcher mode automatically by setting:

```powershell
$env:DINOFORGE_MCP_WATCH="1"
```

before running with `-Action Install` or when using `-Watch`.

You can pass `-Watch` on install/startup:

```powershell
pwsh -File scripts/services/windows/register-mcp-task.ps1 -Action Install -Watch
```

## Linux (systemd user service)

1. Copy `systemd/dinoforge-mcp.service` to `~/.config/systemd/user/dinoforge-mcp.service`.
2. Edit `ExecStart`/`EnvironmentFile` paths if your repo or repo path differs.
3. Enable and start:

```bash
systemctl --user daemon-reload
systemctl --user enable --now dinoforge-mcp.service
```

The `--user` unit files are used for per-user services; make sure `~/.config/systemd/user` exists and Python can reach your game files from that account.
If the session is not active, enable lingering for the user (`loginctl enable-linger $USER`) before relying on auto-start.

## macOS (launchd)

1. Copy `launchd/com.dinoforge.mcp.plist` to `~/Library/LaunchAgents/com.dinoforge.mcp.plist`.
2. Set `EnvironmentVariables` and `ProgramArguments` for your repo path.
3. Load:

```bash
launchctl bootstrap gui/$UID ~/Library/LaunchAgents/com.dinoforge.mcp.plist
```

Tip: On older macOS where `bootstrap` is unavailable, use `launchctl load -w` instead.
