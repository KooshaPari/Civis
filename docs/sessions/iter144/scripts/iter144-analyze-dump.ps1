$ErrorActionPreference = 'Continue'
$dump = 'C:\Users\koosh\Dino\docs\sessions\iter144-wedge-dump.dmp'
$out  = 'C:\Users\koosh\Dino\docs\sessions\iter144-dump-analysis.txt'

# Write a command file for dotnet-dump analyze
$cmdFile = 'C:\Users\koosh\Dino\docs\sessions\iter144-dump-commands.txt'
@(
    'threads',
    'clrthreads',
    'clrstack -all',
    'pstacks',
    'eestack',
    'syncblk',
    'dumpheap -stat',
    'exit'
) | Set-Content -Path $cmdFile -Encoding ascii

Write-Host "Running dotnet-dump analyze on $dump..."
# dotnet-dump analyze supports -c <command> repeated; pipe via stdin instead
Get-Content $cmdFile | & dotnet-dump analyze $dump *> $out
Write-Host "Exit code: $LASTEXITCODE"
Write-Host "Output size: $((Get-Item $out).Length) bytes"
Write-Host "---- first 200 lines ----"
Get-Content $out -TotalCount 200
