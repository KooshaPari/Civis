param(
    [string]$Task = "build:all",
    [switch]$List = $false
)

$ErrorActionPreference = "Stop"
$taskBin = Join-Path (Get-Location) "bin" "task.exe"

if ($List) {
    Write-Host "📋 Available tasks:" -ForegroundColor Cyan
    & $taskBin --list
    exit 0
}

Write-Host "🚀 Running task: $Task" -ForegroundColor Green
& $taskBin $Task

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Task failed: $Task" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Task complete: $Task" -ForegroundColor Green
