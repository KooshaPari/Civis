[CmdletBinding(DefaultParameterSetName = "Install")]
param(
    [ValidateSet("Install", "Uninstall", "Start", "Stop", "Status")]
    [string]$Action = "Status",
    [string]$TaskName = "DINOForge MCP",
    [switch]$Watch
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$scriptsDir = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$startScript = Join-Path $scriptsDir "start-mcp.ps1"
$pwshExe = (Get-Command pwsh).Source
$taskUser = [System.Security.Principal.WindowsIdentity]::GetCurrent().Name
$taskActionArgs = @(
    '-NoProfile',
    '-ExecutionPolicy', 'Bypass',
    '-WindowStyle', 'Hidden',
    '-File', $startScript,
    '-Action', 'start',
    '-Detached'
)
if ($Watch) {
    $taskActionArgs += '-Watch'
}

function Quote-ArgList {
    param([string[]]$Arguments)
    return (($Arguments | ForEach-Object {
        if ($_ -match '[\s"]') {
            '"' + ($_ -replace '"', '\"') + '"'
        }
        else {
            $_
        }
    }) -join ' ')
}

function Invoke-McpStop {
    if (Test-Path $startScript) {
        & $pwshExe -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File $startScript -Action stop | Out-String | Write-Host
    }
}

function Get-McpTask {
    return Get-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue
}

switch ($Action) {
    "Install" {
        $triggers = New-ScheduledTaskTrigger -AtLogOn
        $action = New-ScheduledTaskAction -Execute $pwshExe -Argument (Quote-ArgList $taskActionArgs)
        $principal = New-ScheduledTaskPrincipal -UserId $taskUser -LogonType InteractiveToken -RunLevel Limited
        $settings = New-ScheduledTaskSettingsSet -StartWhenAvailable -RestartCount 3 -RestartInterval (New-TimeSpan -Minutes 1) -ExecutionTimeLimit (New-TimeSpan -Hours 0)

        Register-ScheduledTask -TaskName $TaskName -Trigger $triggers -Action $action -Principal $principal -Settings $settings -Description "DINOForge MCP HTTP/SSE harness" -Force | Out-Null
        Invoke-McpStop
        Write-Host "Registered scheduled task: $TaskName"
        Start-ScheduledTask -TaskName $TaskName
    }
    "Uninstall" {
        if (Get-McpTask) {
            Invoke-McpStop
            Stop-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue
            Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false
            Write-Host "Uninstalled scheduled task: $TaskName"
        } else {
            Write-Host "Task not found: $TaskName"
        }
    }
    "Start" {
        Start-ScheduledTask -TaskName $TaskName
        Write-Host "Started scheduled task: $TaskName"
    }
    "Stop" {
        Invoke-McpStop
        Stop-ScheduledTask -TaskName $TaskName
        Write-Host "Stopped scheduled task: $TaskName"
    }
    "Status" {
        $task = Get-McpTask
        if (-not $task) {
            Write-Host "Task not installed: $TaskName"
            break
        }
        Write-Host ("Task Name: {0}" -f $task.TaskName)
        Write-Host ("State   : {0}" -f $task.State)
        Write-Host ("Run As  : {0}" -f $task.Principal.UserId)
    }
}
