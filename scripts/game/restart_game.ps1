$ErrorActionPreference = "Stop"
Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Starting main game..."

$psi = New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe"
$psi.WorkingDirectory = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"
$psi.UseShellExecute = $true
$psi.WindowStyle = [System.Diagnostics.ProcessWindowStyle]::Normal

$proc = [System.Diagnostics.Process]::Start($psi)
Write-Host "[$(Get-Date -Format 'HH:mm:ss')] PID: $($proc.Id)"

Start-Sleep -Seconds 8

if (-not $proc.HasExited) {
    Write-Host "[$(Get-Date -Format 'HH:mm:ss')] STILL RUNNING after 8s - SUCCESS"
    $proc | Select-Object Id, ProcessName, MainWindowTitle | Format-Table -AutoSize
} else {
    Write-Host "[$(Get-Date -Format 'HH:mm:ss')] EXITED with code: $($proc.ExitCode)"
}
