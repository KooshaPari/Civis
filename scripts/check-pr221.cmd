@echo off
gh pr view 221 --repo KooshaPari/Dino --json state,mergedAt,mergeCommit,reviewDecision,mergeStateStatus,headRefOid
gh api repos/KooshaPari/Dino/pulls/221/reviews > "%TEMP%\pr221-reviews.json"
python -c "import json; r=json.load(open(r'%TEMP%\pr221-reviews.json')); cr=[x for x in r if x.get('user',{}).get('login')=='coderabbitai[bot]']; print('coderabbit:', json.dumps([{'state':x['state'],'at':x.get('submitted_at')} for x in cr]))"
