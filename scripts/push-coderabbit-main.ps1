$ErrorActionPreference = 'Stop'
$sha = (gh api 'repos/KooshaPari/Dino/contents/.coderabbit.yaml?ref=main' | ConvertFrom-Json).sha
$target = Join-Path $PSScriptRoot 'coderabbit-main-target.yaml'
$content = Get-Content $target -Raw
$b64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($content))
gh api -X PUT repos/KooshaPari/Dino/contents/.coderabbit.yaml `
  -f message='chore(coderabbit): enable request_changes_workflow and auto_approve on main' `
  -f content=$b64 `
  -f sha=$sha `
  -f branch='main' `
  --jq '.commit.html_url'
