$deadline = (Get-Date).AddMinutes(10)
$i = 0
while ((Get-Date) -lt $deadline) {
    $i++
    $j = gh pr view 221 --repo KooshaPari/Dino --json reviewDecision,mergeStateStatus | ConvertFrom-Json
    Write-Host "[$i] reviewDecision=$($j.reviewDecision) mergeStateStatus=$($j.mergeStateStatus)"
    if ($j.reviewDecision -eq 'APPROVED') { exit 0 }
    Start-Sleep -Seconds 60
}
exit 1
