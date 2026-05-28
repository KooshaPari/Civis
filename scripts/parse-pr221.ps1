$r = Get-Content 'C:\Users\koosh\Dino\scripts\pr221-reviews.json' -Raw | ConvertFrom-Json
Write-Host '=== Reviews ==='
$r | ForEach-Object { Write-Host "$($_.user.login) $($_.state) $($_.submitted_at)" }
$c = Get-Content 'C:\Users\koosh\Dino\scripts\pr221-issue-comments.json' -Raw | ConvertFrom-Json
Write-Host '=== CodeRabbit comments ==='
$c | Where-Object { $_.user.login -match 'coderabbit' } | ForEach-Object {
    $snippet = if ($_.body.Length -gt 400) { $_.body.Substring(0, 400) + '...' } else { $_.body }
    Write-Host "--- $($_.created_at) ---`n$snippet`n"
}
