$ErrorActionPreference = 'Continue'

# kill any existing instance
Stop-Process -Name 'Diplomacy is Not an Option' -Force -ErrorAction SilentlyContinue
Get-Process | Where-Object { $_.ProcessName -like '*Diplomacy*' } | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 3
Write-Host "STEP1: killed existing instances"

# launch
$exe = 'G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe'
$dir = 'G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option'
Start-Process -FilePath $exe -WorkingDirectory $dir
Write-Host "STEP2: launched"

# wait 12s — capture before fatal error dialog auto-pops if possible
Start-Sleep -Seconds 12

# list all matching processes
$procs = @(Get-Process -Name 'Diplomacy is Not an Option' -ErrorAction SilentlyContinue)
Write-Host "STEP3: found $($procs.Count) processes"
foreach ($p in $procs) {
    Write-Host "  PID=$($p.Id) Responding=$($p.Responding) Title='$($p.MainWindowTitle)' WS=$([math]::Round($p.WorkingSet64/1MB,1))MB"
}

if ($procs.Count -eq 0) {
    Write-Host "STEP3_FAIL: no process"
    exit 1
}

# Pick the largest WS (likely the actual game, not the error dialog wrapper)
$target = $procs | Sort-Object WorkingSet64 -Descending | Select-Object -First 1
$dinoPid = $target.Id
Write-Host "STEP4: target PID=$dinoPid"

# procdump
$dumpPath = 'C:\Users\koosh\Dino\docs\sessions\iter144-wedge-dump.dmp'
if (Test-Path $dumpPath) { Remove-Item $dumpPath -Force }
Write-Host "STEP5: running procdump64 -ma -64 $dinoPid"
$psi = New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName = 'C:\tools\sysinternals\procdump64.exe'
$psi.Arguments = "-accepteula -ma -64 $dinoPid `"$dumpPath`""
$psi.RedirectStandardOutput = $true
$psi.RedirectStandardError = $true
$psi.UseShellExecute = $false
$pd = [System.Diagnostics.Process]::Start($psi)
$stdout = $pd.StandardOutput.ReadToEnd()
$stderr = $pd.StandardError.ReadToEnd()
$pd.WaitForExit(120000) | Out-Null
Write-Host "STEP5 exit=$($pd.ExitCode)"
Write-Host "--- stdout ---"
Write-Host $stdout
Write-Host "--- stderr ---"
Write-Host $stderr

# stop DINO
Stop-Process -Name 'Diplomacy is Not an Option' -Force -ErrorAction SilentlyContinue
Get-Process | Where-Object { $_.ProcessName -like '*Diplomacy*' } | Stop-Process -Force -ErrorAction SilentlyContinue
Write-Host "STEP6: stopped DINO"

# report
if (Test-Path $dumpPath) {
    $size = (Get-Item $dumpPath).Length
    $sizeMb = [math]::Round($size / 1MB, 2)
    Write-Host "STEP7: DUMP_OK size=${sizeMb}MB path=$dumpPath"
    $bytes = [System.IO.File]::ReadAllBytes($dumpPath)[0..15]
    $hex = ($bytes | ForEach-Object { '{0:X2}' -f $_ }) -join ' '
    Write-Host "DUMP_HEADER: $hex (MDMP = 4D 44 4D 50)"
} else {
    Write-Host "STEP7: DUMP_MISSING"
}
