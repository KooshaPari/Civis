<#
.SYNOPSIS
    Pre-flight check for the first-external-receipt runbook.

.DESCRIPTION
    Verifies the four prerequisites needed to land the first external Kimi judge
    receipt in docs/proof/judge-receipts/, before the user actually runs the runbook.
    Each check is independent; the script reports all failures rather than stopping
    at the first one.

    Checks:
      1. MOONSHOT_API_KEY env var is set in the current process.
      2. MCP server responds at http://127.0.0.1:8765/health (default port).
      3. DINO process is running (so a screenshot will capture something useful).
      4. httpx is importable in the MCP server's Python env.

    Exit codes:
      0 = all four pass — safe to proceed with first-external-receipt-runbook.md
      1 = at least one check failed — see output for which

.PARAMETER McpUrl
    The MCP server health endpoint. Default http://127.0.0.1:8765/health.

.PARAMETER PythonExe
    Python executable used to verify httpx. Default 'python'.

.EXAMPLE
    pwsh scripts/proof/preflight-runbook.ps1
    Runs all four checks and prints a summary.
#>
[CmdletBinding()]
param(
    [string]$McpUrl = "http://127.0.0.1:8765/health",
    [string]$PythonExe = "python"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Continue"

$results = @()

# Check 1: Judge API key (MOONSHOT_API_KEY or FIREWORKS_API_KEY) — #103: Fireworks runbook
# needs FIREWORKS_API_KEY; either provider satisfies the external-judge requirement.
$moonshotKey = [Environment]::GetEnvironmentVariable("MOONSHOT_API_KEY", "Process")
$fireworksKey = [Environment]::GetEnvironmentVariable("FIREWORKS_API_KEY", "Process")
$presentKeys = @()
if (-not [string]::IsNullOrWhiteSpace($moonshotKey)) {
    $masked = $moonshotKey.Substring(0, [Math]::Min(4, $moonshotKey.Length)) + "..." + $moonshotKey.Substring([Math]::Max(0, $moonshotKey.Length - 4))
    $presentKeys += "MOONSHOT_API_KEY ($masked)"
}
if (-not [string]::IsNullOrWhiteSpace($fireworksKey)) {
    $masked = $fireworksKey.Substring(0, [Math]::Min(4, $fireworksKey.Length)) + "..." + $fireworksKey.Substring([Math]::Max(0, $fireworksKey.Length - 4))
    $presentKeys += "FIREWORKS_API_KEY ($masked)"
}
if ($presentKeys.Count -eq 0) {
    $results += [pscustomobject]@{
        Check = "Judge API key (MOONSHOT_API_KEY or FIREWORKS_API_KEY)"
        Pass = $false
        Detail = "Neither set. Run: `$env:MOONSHOT_API_KEY = '<key>'  -or-  `$env:FIREWORKS_API_KEY = '<key>'"
    }
} else {
    $results += [pscustomobject]@{
        Check = "Judge API key (MOONSHOT_API_KEY or FIREWORKS_API_KEY)"
        Pass = $true
        Detail = "Set: " + ($presentKeys -join ", ")
    }
}

# Check 2: MCP server responds
try {
    $health = Invoke-RestMethod -Uri $McpUrl -Method Get -TimeoutSec 5 -ErrorAction Stop
    $results += [pscustomobject]@{
        Check = "MCP server"
        Pass = $true
        Detail = "Responding at $McpUrl"
    }
} catch {
    $results += [pscustomobject]@{
        Check = "MCP server"
        Pass = $false
        Detail = "Not reachable at $McpUrl. Run: pwsh scripts/start-mcp.ps1 -Action start -Detached"
    }
}

# Check 3: DINO running
$dino = Get-Process -Name "Diplomacy is Not an Option" -ErrorAction SilentlyContinue
if ($dino) {
    $pidList = ($dino | Select-Object -ExpandProperty Id) -join ","
    $results += [pscustomobject]@{
        Check = "DINO process"
        Pass = $true
        Detail = "Running (PID $pidList)"
    }
} else {
    $results += [pscustomobject]@{
        Check = "DINO process"
        Pass = $false
        Detail = "Not running. Launch via Steam, scripts/game/Launch-DINOBoxInstance.ps1, or boot.config single-instance=0 path."
    }
}

# Check 4: httpx importable
try {
    $output = & $PythonExe -c "import httpx; print(httpx.__version__)" 2>&1
    if ($LASTEXITCODE -eq 0) {
        $results += [pscustomobject]@{
            Check = "httpx (Python)"
            Pass = $true
            Detail = "Importable; version $($output.Trim())"
        }
    } else {
        $results += [pscustomobject]@{
            Check = "httpx (Python)"
            Pass = $false
            Detail = "Import failed: $output. Install: pip install httpx"
        }
    }
} catch {
    $results += [pscustomobject]@{
        Check = "httpx (Python)"
        Pass = $false
        Detail = "Could not run python: $_. Set -PythonExe to the right interpreter or activate the MCP venv."
    }
}

# Render
$results | Format-Table -AutoSize

$failed = @($results | Where-Object { -not $_.Pass })
if ($failed.Count -eq 0) {
    Write-Host ""
    Write-Host "All preflight checks passed. Proceed with docs/setup/first-external-receipt-runbook.md."
    exit 0
} else {
    Write-Host ""
    Write-Host "Preflight: $($failed.Count) check(s) failed. Resolve each issue listed above before running the runbook."
    exit 1
}
