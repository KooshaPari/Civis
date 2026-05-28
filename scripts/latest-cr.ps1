$c = gh api repos/KooshaPari/Dino/issues/221/comments | ConvertFrom-Json
$last = $c | Where-Object { $_.user.login -match 'coderabbit' } | Select-Object -Last 1
Write-Host "created: $($last.created_at)"
Write-Host $last.body
$r = gh api repos/KooshaPari/Dino/pulls/221/reviews | ConvertFrom-Json
Write-Host '=== APPROVED ==='
$r | Where-Object { $_.state -eq 'APPROVED' } | ForEach-Object { Write-Host "$($_.user.login) $($_.submitted_at)" }
