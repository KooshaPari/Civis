<#
.SYNOPSIS
  Autonomous E2E probe: verify DINO main-menu UI responds to mouse clicks.
.DESCRIPTION
  Closes the autonomy gap (task #532). Lets the orchestrator self-confirm
  "UI is clickable" without asking the user.

  Pipeline:
    1. Check game process is running (best-effort, non-fatal).
    2. Health-check MCP server at 127.0.0.1:8765 via curl.exe.
    3. Open MCP streamable-HTTP session (FastMCP protocol).
    4. Baseline screenshot via MCP tool game_screenshot.
    5. Inject left click at (X, Y) via MCP tool game_input.
    6. Wait SettleMs, take after-click screenshot via game_screenshot.
    7. pHash diff (8x8 grayscale average-hash, Hamming distance).
    8. Emit JSON {clicked, ui_changed, baseline_path, after_path, diff_score, ...}.

  -DryRun skips MCP session + click; still validates parser + JSON output.
  Tools used: game_input, game_screenshot (no server.py changes).
.PARAMETER X
  Click X (screen-absolute). Default: primary-screen width / 2.
.PARAMETER Y
  Click Y (screen-absolute). Default: primary-screen height / 2.
.PARAMETER SettleMs
  Wait after click before second screenshot. Default 800.
.PARAMETER DiffThreshold
  pHash Hamming distance treated as "UI changed". Default 8 (of 64).
.PARAMETER DryRun
  Skip MCP calls; emit JSON for parser/structure validation.
.PARAMETER OutDir
  Where to write screenshots. Default $env:TEMP\DINOForge\probe-menu-click\
.EXAMPLE
  pwsh -File scripts/diag/probe-menu-click.ps1 -DryRun
#>

[CmdletBinding()]
param(
    [int]$X = -1,
    [int]$Y = -1,
    [int]$SettleMs = 800,
    [int]$DiffThreshold = 8,
    [switch]$DryRun,
    [string]$OutDir = "$env:TEMP\DINOForge\probe-menu-click"
)

$ErrorActionPreference = 'Continue'

$Result = [ordered]@{
    clicked       = $false
    ui_changed    = $false
    baseline_path = $null
    after_path    = $null
    diff_score    = $null
    mcp_reachable = $false
    exit_reason   = 'unknown'
    click_x       = $X
    click_y       = $Y
    dry_run       = [bool]$DryRun
    timestamp_utc = (Get-Date).ToUniversalTime().ToString('o')
}

if (-not (Test-Path $OutDir)) { New-Item -ItemType Directory -Path $OutDir -Force | Out-Null }

Add-Type -AssemblyName System.Windows.Forms -ErrorAction SilentlyContinue
Add-Type -AssemblyName System.Drawing -ErrorAction SilentlyContinue
$primary = [System.Windows.Forms.Screen]::PrimaryScreen
if ($X -lt 0) { $X = [int]($primary.Bounds.Width / 2) }
if ($Y -lt 0) { $Y = [int]($primary.Bounds.Height / 2) }
$Result.click_x = $X
$Result.click_y = $Y

# --- MCP curl helpers --------------------------------------------------------
$McpHost   = '127.0.0.1:8765'
$McpUrl    = "http://$McpHost/mcp"
$HealthUrl = "http://$McpHost/health"
$script:Sid = $null
$ContentType = 'application/json'
$AcceptHdr   = 'application/json, text/event-stream'

function Invoke-Curl {
    param([string]$Url, [string]$Method = 'GET', [string]$Body = $null, [hashtable]$Headers = $null, [int]$TimeoutSec = 10, [switch]$IncludeHeaders)
    $args = @('-s', '-m', "$TimeoutSec")
    if ($IncludeHeaders) { $args += '-i' }
    if ($Method -ne 'GET') { $args += @('-X', $Method) }
    if ($Headers) {
        foreach ($k in $Headers.Keys) { $args += @('-H', "${k}: $($Headers[$k])") }
    }
    if ($Body) {
        $tmp = New-TemporaryFile
        Set-Content -Path $tmp.FullName -Value $Body -Encoding UTF8 -NoNewline
        $args += @('--data-binary', "@$($tmp.FullName)")
    }
    $args += $Url
    try {
        $out = & curl.exe @args 2>$null
        if ($tmp) { Remove-Item -Path $tmp.FullName -Force -ErrorAction SilentlyContinue } # remove-item-ok: temp-cleanup-ok: ephemeral curl request body temp file
        return ($out -join "`n")
    } catch {
        return $null
    }
}

function Test-McpHealth {
    $raw = Invoke-Curl -Url $HealthUrl -TimeoutSec 3
    if (-not $raw) { return $false }
    try {
        $obj = $raw | ConvertFrom-Json -ErrorAction Stop
        return ($obj.status -eq 'ok')
    } catch { return $false }
}

function Open-McpSession {
    $body = '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"probe-menu-click","version":"0.1"}}}'
    $raw = Invoke-Curl -Url $McpUrl -Method POST -Body $body `
        -Headers @{ 'Content-Type' = $ContentType; 'Accept' = $AcceptHdr } `
        -IncludeHeaders -TimeoutSec 8
    if (-not $raw) { return $false }
    foreach ($line in ($raw -split "`n")) {
        if ($line -match '^[Mm]cp-[Ss]ession-[Ii]d:\s*(\S+)') {
            $script:Sid = $matches[1].Trim()
            break
        }
    }
    if (-not $script:Sid) { return $false }
    # MCP requires notifications/initialized after init
    $null = Invoke-Curl -Url $McpUrl -Method POST `
        -Body '{"jsonrpc":"2.0","method":"notifications/initialized"}' `
        -Headers @{ 'Content-Type' = $ContentType; 'Accept' = $AcceptHdr; 'mcp-session-id' = $script:Sid } `
        -TimeoutSec 5
    return $true
}

function Call-McpTool {
    param([string]$Name, [hashtable]$Arguments = @{})
    if (-not $script:Sid) { return $null }
    $payload = @{
        jsonrpc = '2.0'
        id      = [int](Get-Random -Maximum 99999)
        method  = 'tools/call'
        params  = @{ name = $Name; arguments = $Arguments }
    } | ConvertTo-Json -Depth 8 -Compress

    $raw = Invoke-Curl -Url $McpUrl -Method POST -Body $payload `
        -Headers @{ 'Content-Type' = $ContentType; 'Accept' = $AcceptHdr; 'mcp-session-id' = $script:Sid } `
        -TimeoutSec 20
    if (-not $raw) { return $null }
    # SSE-framed: lines starting with "data: " carry the JSON body
    $dataLine = ($raw -split "`n" | Where-Object { $_ -like 'data: *' } | Select-Object -First 1)
    if ($dataLine) {
        try { return ($dataLine.Substring(6).Trim() | ConvertFrom-Json -ErrorAction Stop) } catch { return $null }
    }
    try { return ($raw | ConvertFrom-Json -ErrorAction Stop) } catch { return $null }
}

# --- Screenshot (MCP-only — keeps probe surface narrow) ----------------------
function Get-Screenshot {
    param([string]$Path)
    if (-not $script:Sid -or $DryRun) {
        # In dry-run or when MCP session isn't open, write a tiny placeholder PNG so
        # the diff path can still execute end-to-end on a clean machine.
        try {
            $bmp = New-Object System.Drawing.Bitmap 16, 16
            $g = [System.Drawing.Graphics]::FromImage($bmp)
            $g.Clear([System.Drawing.Color]::Black)
            # Vary one pixel per call so dry-run diff is non-zero
            $bmp.SetPixel((Get-Random -Maximum 16), (Get-Random -Maximum 16), [System.Drawing.Color]::White)
            $bmp.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
            $g.Dispose(); $bmp.Dispose()
            return $true
        } catch { return $false }
    }

    $resp = Call-McpTool -Name 'game_screenshot' -Arguments @{ output_path = $Path }
    $structured = $resp.result.structuredContent
    return ($structured -and $structured.success -eq $true -and (Test-Path $Path))
}

# --- pHash + Hamming ---------------------------------------------------------
function Get-PHash64 {
    param([string]$ImagePath)
    if (-not (Test-Path $ImagePath)) { return $null }
    try {
        $img = [System.Drawing.Image]::FromFile($ImagePath)
        $bmp = New-Object System.Drawing.Bitmap 8, 8
        $g = [System.Drawing.Graphics]::FromImage($bmp)
        $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
        $g.DrawImage($img, 0, 0, 8, 8)
        $vals = New-Object 'System.Collections.Generic.List[int]'
        for ($yy = 0; $yy -lt 8; $yy++) {
            for ($xx = 0; $xx -lt 8; $xx++) {
                $px = $bmp.GetPixel($xx, $yy)
                $vals.Add([int](($px.R + $px.G + $px.B) / 3))
            }
        }
        $avg = ($vals | Measure-Object -Average).Average
        $bits = 0L
        for ($i = 0; $i -lt 64; $i++) {
            if ($vals[$i] -gt $avg) { $bits = $bits -bor ([long]1 -shl $i) }
        }
        $g.Dispose(); $bmp.Dispose(); $img.Dispose()
        return $bits
    } catch { return $null }
}

function Get-Hamming64 {
    param([long]$A, [long]$B)
    $x = $A -bxor $B
    $count = 0
    for ($i = 0; $i -lt 64; $i++) {
        if (($x -shr $i) -band 1L) { $count++ }
    }
    return $count
}

# --- Main flow ---------------------------------------------------------------
try {
    $gameProc = Get-Process -Name 'Diplomacy is Not an Option' -ErrorAction SilentlyContinue
    $Result | Add-Member -NotePropertyName game_running -NotePropertyValue ([bool]$gameProc) -Force

    $Result.mcp_reachable = Test-McpHealth

    if (-not $DryRun -and $Result.mcp_reachable) {
        if (-not (Open-McpSession)) {
            Write-Verbose 'MCP session init failed.'
        }
    }

    $baseline = Join-Path $OutDir ("baseline-" + (Get-Date -Format 'yyyyMMdd-HHmmss-fff') + ".png")
    if (Get-Screenshot -Path $baseline) { $Result.baseline_path = $baseline }

    if (-not $DryRun -and $script:Sid) {
        $resp = Call-McpTool -Name 'game_input' -Arguments @{
            mouse_x = $X
            mouse_y = $Y
            click   = $true
        }
        $structured = $resp.result.structuredContent
        if ($structured -and $structured.success -eq $true) { $Result.clicked = $true }
    }

    if ($Result.clicked -or $DryRun) {
        Start-Sleep -Milliseconds $SettleMs
        $after = Join-Path $OutDir ("after-" + (Get-Date -Format 'yyyyMMdd-HHmmss-fff') + ".png")
        if (Get-Screenshot -Path $after) { $Result.after_path = $after }
    }

    if ($Result.baseline_path -and $Result.after_path) {
        $h1 = Get-PHash64 -ImagePath $Result.baseline_path
        $h2 = Get-PHash64 -ImagePath $Result.after_path
        if ($null -ne $h1 -and $null -ne $h2) {
            $d = Get-Hamming64 -A $h1 -B $h2
            $Result.diff_score = $d
            $Result.ui_changed = ($d -ge $DiffThreshold)
        }
    }

    if ($Result.exit_reason -eq 'unknown') {
        if ($DryRun) { $Result.exit_reason = 'dry_run_complete' }
        elseif (-not $Result.mcp_reachable) { $Result.exit_reason = 'mcp_unreachable' }
        elseif (-not $script:Sid) { $Result.exit_reason = 'mcp_session_init_failed' }
        elseif (-not $Result.clicked) { $Result.exit_reason = 'click_injection_failed' }
        elseif (-not $Result.after_path) { $Result.exit_reason = 'after_screenshot_failed' }
        elseif ($null -eq $Result.diff_score) { $Result.exit_reason = 'phash_failed' }
        else { $Result.exit_reason = 'complete' }
    }
} catch {
    $Result.exit_reason = "exception: $($_.Exception.Message)"
}

$Result | ConvertTo-Json -Depth 4

if ($DryRun) { exit 0 }
if ($Result.ui_changed) { exit 0 } else { exit 1 }
