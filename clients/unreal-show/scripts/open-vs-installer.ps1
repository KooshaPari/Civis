#Requires -Version 5.1
<#
.SYNOPSIS
  Open Visual Studio Installer on the CivShow workload tab (human step for MSVC v14.44).
#>
$Setup = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\setup.exe'
if (-not (Test-Path -LiteralPath $Setup)) {
    Write-Error "Installer not found: $Setup"
}
Start-Process $Setup
Start-Process 'https://learn.microsoft.com/en-us/visualstudio/install/modify-visual-studio?view=vs-2022'
Write-Host @"

**Visual Studio Installer is open on your machine** (agent cannot click inside it).

In that window:
1. Click **Modify** on **Visual Studio 2022 Community** (NOT Preview — Preview only has banned 14.42/14.43).
2. Open the **Workloads** tab → check **Desktop development with C++**.
   - Or **Individual components** → search **MSVC v143** → check **... (v14.44-17.14)**.
3. Also check **Windows 11 SDK (10.0.22621)** if missing.
4. Click **Modify** (download ~2–6 GB) and wait until finished.
5. Close Installer, then in PowerShell:

   .\clients\unreal-show\scripts\build.ps1

Agent already ran `setup.exe modify` but it blocked while this UI was open.

"@ -ForegroundColor Cyan
