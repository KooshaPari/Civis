@echo off
setlocal
cd /d C:\Users\koosh\Dino

echo === 1) CodeRabbit triggers (already posted if comment 4550729597 exists) ===
gh pr comment 221 --repo KooshaPari/Dino --body-file "%~dp0pr221-coderabbit.txt"

echo === 2) Poll until APPROVED (90s interval, 15 min max) ===
set /a n=0
:poll
set /a n+=1
gh pr view 221 --repo KooshaPari/Dino --json reviewDecision,mergeStateStatus,headRefOid
gh api repos/KooshaPari/Dino/pulls/221/reviews --jq "[.[] | select(.user.login==\"coderabbitai[bot]\") | {state,submitted_at}]"
for /f %%i in ('gh pr view 221 --repo KooshaPari/Dino --json reviewDecision -q .reviewDecision') do set RD=%%i
if "%RD%"=="APPROVED" goto merge
if %n% geq 10 goto workflow
echo Waiting 90s... [%n%/10]
timeout /t 90 /nobreak >nul
goto poll

:merge
echo === 3) Merge PR ===
gh pr merge 221 --merge --repo KooshaPari/Dino
if %ERRORLEVEL% equ 0 goto sync

:workflow
echo === 4) Fallback workflow_dispatch ===
gh workflow run agent-merge-on-bot-approve.yml --repo KooshaPari/Dino --ref followup/post-pr188-followups -f pr_number=221
echo Watch: https://github.com/KooshaPari/Dino/actions/workflows/agent-merge-on-bot-approve.yml
exit /b 1

:sync
echo === 5) Sync local main ===
git fetch origin
git checkout main
git pull origin main
gh pr view 221 --repo KooshaPari/Dino --json state,mergedAt,mergeCommit
git rev-parse HEAD
