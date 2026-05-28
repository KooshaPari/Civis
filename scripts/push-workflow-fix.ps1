$path = 'C:\Users\koosh\Dino\.github\workflows\agent-merge-on-bot-approve.yml'
$bytes = [System.IO.File]::ReadAllBytes($path)
$content = [Convert]::ToBase64String($bytes)
$sha = gh api 'repos/KooshaPari/Dino/contents/.github/workflows/agent-merge-on-bot-approve.yml?ref=followup/post-pr188-followups' --jq '.sha'
$payload = @{
  message = 'ci: pin github-script@v7 for agent merge workflow'
  content = $content
  branch  = 'followup/post-pr188-followups'
  sha     = $sha
}
$jsonPath = 'C:\Users\koosh\Dino\scripts\wf-body.json'
[System.IO.File]::WriteAllText($jsonPath, ($payload | ConvertTo-Json -Depth 5 -Compress))
gh api -X PUT 'repos/KooshaPari/Dino/contents/.github/workflows/agent-merge-on-bot-approve.yml' --input $jsonPath
