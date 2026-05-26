$ErrorActionPreference = 'Continue'
$dir = 'C:\tools\sysinternals'
if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Path $dir -Force | Out-Null }
$zip = Join-Path $dir 'Procdump.zip'
try {
    Invoke-WebRequest -Uri 'https://download.sysinternals.com/files/Procdump.zip' -OutFile $zip -UseBasicParsing -ErrorAction Stop
    Write-Host 'DOWNLOAD_OK'
    Expand-Archive -Path $zip -DestinationPath $dir -Force
    Write-Host 'EXTRACT_OK'
    Get-ChildItem $dir | Select-Object Name, Length | Format-Table -AutoSize
    & "$dir\procdump64.exe" -? 2>&1 | Select-Object -First 8
} catch {
    Write-Host "DOWNLOAD_FAIL: $_"
}
