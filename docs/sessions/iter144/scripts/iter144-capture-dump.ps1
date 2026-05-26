$ErrorActionPreference = 'Continue'

# Step 1: kill any existing instance
Stop-Process -Name 'Diplomacy is Not an Option' -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 3
Write-Host "STEP1: killed existing instances"

# Step 2: launch
$exe = 'G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe'
$dir = 'G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option'
Start-Process -FilePath $exe -WorkingDirectory $dir
Write-Host "STEP2: launched"

# Step 3: wait 15s for wedge
Start-Sleep -Seconds 15

# Step 4: get PID + status
$proc = Get-Process -Name 'Diplomacy is Not an Option' -ErrorAction SilentlyContinue
if (-not $proc) {
    Write-Host "STEP4_FAIL: no process found"
    exit 1
}
$dinoPid = $proc.Id
$responding = $proc.Responding
$title = $proc.MainWindowTitle
Write-Host "STEP4: PID=$dinoPid Responding=$responding Title='$title'"

# Step 5: procdump
$dumpPath = 'C:\Users\koosh\Dino\docs\sessions\iter144-wedge-dump.dmp'
if (Test-Path $dumpPath) { Remove-Item $dumpPath -Force }
Write-Host "STEP5: starting procdump64 -ma -64 $dinoPid $dumpPath"
& 'C:\tools\sysinternals\procdump64.exe' -accepteula -ma -64 $dinoPid $dumpPath 2>&1 | Tee-Object -Variable pdOut | Out-Host
Write-Host "STEP5: procdump exit=$LASTEXITCODE"

# Step 6: stop DINO
Stop-Process -Name 'Diplomacy is Not an Option' -Force -ErrorAction SilentlyContinue
Write-Host "STEP6: stopped DINO"

# Step 7: report
if (Test-Path $dumpPath) {
    $size = (Get-Item $dumpPath).Length
    $sizeMb = [math]::Round($size / 1MB, 2)
    Write-Host "STEP7: DUMP_OK size=${sizeMb}MB path=$dumpPath"
    # First 8 bytes of dump header
    $bytes = [System.IO.File]::ReadAllBytes($dumpPath)[0..15]
    $hex = ($bytes | ForEach-Object { '{0:X2}' -f $_ }) -join ' '
    Write-Host "DUMP_HEADER: $hex"
    # 'MDMP' = 4D 44 4D 50 = valid minidump signature
} else {
    Write-Host "STEP7: DUMP_MISSING"
}
