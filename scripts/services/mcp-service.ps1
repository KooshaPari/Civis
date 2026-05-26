#!/usr/bin/env pwsh
<#
.SYNOPSIS
Install, uninstall, or control the MCP harness service with one command.

.DESCRIPTION
This wrapper selects the appropriate service mechanism for the current OS:
- Windows: Task Scheduler (`scripts/services/windows/register-mcp-task.ps1`)
- Linux: systemd user unit (`scripts/services/systemd/dinoforge-mcp.service`)
- macOS: launchd agent (`scripts/services/launchd/com.dinoforge.mcp.plist`)
#>

[CmdletBinding()]
param(
    [ValidateSet("Install", "Uninstall", "Start", "Stop", "Status")]
    [string]$Action = "Status",
    [switch]$Watch
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$serviceName = "dinoforge-mcp"

function Invoke-ServiceTemplateRender {
    param(
        [string]$TemplatePath,
        [string]$OutputPath,
        [string]$RepoRoot
    )

    $template = Get-Content -Raw -Path $TemplatePath -ErrorAction Stop
    $rendered = $template.Replace("__REPO_ROOT__", $RepoRoot.Replace("\", "/"))
    $parentDir = Split-Path -Parent $OutputPath
    if (-not (Test-Path $parentDir)) {
        New-Item -ItemType Directory -Path $parentDir -Force | Out-Null
    }
    Set-Content -Path $OutputPath -Value $rendered -Encoding UTF8
}

function Invoke-CommandSafe {
    param(
        [string]$Command,
        [string[]]$ArgumentList = @()
    )

    Write-Host "[MCP] $Command $($ArgumentList -join ' ')"
    & $Command @ArgumentList
}

if ($IsWindows) {
    $installer = Join-Path $repoRoot "scripts\services\windows\register-mcp-task.ps1"
    $args = @(
        "-NoProfile",
        "-ExecutionPolicy",
        "Bypass",
        "-File",
        $installer,
        "-Action",
        $Action
    )
    if ($Watch) {
        $args += "-Watch"
    }
    Invoke-CommandSafe -Command "pwsh" -ArgumentList $args
    return
}

if ($IsLinux) {
    $systemdDir = Join-Path $HOME ".config/systemd/user"
    $serviceFile = Join-Path $systemdDir "$serviceName.service"
    $templatePath = Join-Path $repoRoot "scripts/services/systemd/dinoforge-mcp.service"

    switch ($Action) {
        "Install" {
            Invoke-ServiceTemplateRender -TemplatePath $templatePath -OutputPath $serviceFile -RepoRoot $repoRoot
            Invoke-CommandSafe -Command "systemctl" -ArgumentList @("--user", "daemon-reload")
            Invoke-CommandSafe -Command "systemctl" -ArgumentList @("--user", "enable", "--now", $serviceName)
            Invoke-CommandSafe -Command "systemctl" -ArgumentList @("--user", "status", "--no-pager", "--full", $serviceName)
            return
        }
        "Uninstall" {
            Invoke-CommandSafe -Command "systemctl" -ArgumentList @("--user", "disable", "--now", $serviceName)
            if (Test-Path $serviceFile) { Remove-Item $serviceFile -Force }
            Invoke-CommandSafe -Command "systemctl" -ArgumentList @("--user", "daemon-reload")
            return
        }
        "Start" {
            Invoke-CommandSafe -Command "systemctl" -ArgumentList @("--user", "start", $serviceName)
            return
        }
        "Stop" {
            Invoke-CommandSafe -Command "systemctl" -ArgumentList @("--user", "stop", $serviceName)
            return
        }
        "Status" {
            Invoke-CommandSafe -Command "systemctl" -ArgumentList @("--user", "status", "--no-pager", "--full", $serviceName)
            return
        }
    }
}

if ($IsMacOS) {
    $agentDir = Join-Path $HOME "Library/LaunchAgents"
    $plistFile = Join-Path $agentDir "com.dinoforge.mcp.plist"
    $templatePath = Join-Path $repoRoot "scripts/services/launchd/com.dinoforge.mcp.plist"
    Invoke-ServiceTemplateRender -TemplatePath $templatePath -OutputPath $plistFile -RepoRoot $repoRoot

    switch ($Action) {
        "Install" {
            try {
                Invoke-CommandSafe -Command "launchctl" -ArgumentList @("bootout", "gui/$UID", $plistFile)
            }
            catch {}
            Invoke-CommandSafe -Command "launchctl" -ArgumentList @("bootstrap", "gui/$UID", $plistFile)
            return
        }
        "Uninstall" {
            try {
                Invoke-CommandSafe -Command "launchctl" -ArgumentList @("bootout", "gui/$UID", $plistFile)
            }
            catch {}
            if (Test-Path $agentDir) { Remove-Item $plistFile -Force -ErrorAction SilentlyContinue }
            return
        }
        "Start" {
            Invoke-CommandSafe -Command "launchctl" -ArgumentList @("kickstart", "-k", "gui/$UID/com.dinoforge.mcp")
            return
        }
        "Stop" {
            try {
                Invoke-CommandSafe -Command "launchctl" -ArgumentList @("bootout", "gui/$UID", $plistFile)
            }
            catch {}
            return
        }
        "Status" {
            Invoke-CommandSafe -Command "launchctl" -ArgumentList @("print", "gui/$UID/com.dinoforge.mcp")
            return
        }
    }
}

throw "Unsupported platform for mcp-service.ps1"
