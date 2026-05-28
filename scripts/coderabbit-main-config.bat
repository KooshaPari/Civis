@echo off
setlocal EnableDelayedExpansion
cd /d C:\Users\koosh\Dino

echo === fetch ===
git fetch origin
if errorlevel 1 exit /b 1

echo === checkout branch from origin/main ===
git checkout -B agent/coderabbit-main-config origin/main
if errorlevel 1 exit /b 1

echo === apply target .coderabbit.yaml (match followup branch) ===
copy /Y C:\Users\koosh\Dino\scripts\coderabbit-main-target.yaml C:\Users\koosh\Dino\.coderabbit.yaml >nul
git add .coderabbit.yaml
git diff --cached --quiet
if errorlevel 1 (
  git commit -m "chore: enable CodeRabbit bot approve on main"
) else (
  echo No changes to commit
)

echo === push ===
git push -u origin agent/coderabbit-main-config
if errorlevel 1 exit /b 1

echo === create PR ===
gh pr create --repo KooshaPari/Dino --base main --head agent/coderabbit-main-config --title "chore: enable CodeRabbit bot approve on main" --body "Enable request_changes_workflow and auto_approve on main so CodeRabbit reads bot-approve settings for public repo PRs (required before PR #221 can get coderabbitai[bot] APPROVED)."
if errorlevel 1 exit /b 1

for /f %%i in ('gh pr list --repo KooshaPari/Dino --head agent/coderabbit-main-config --json number --jq ".[0].number"') do set CONFIG_PR=%%i
echo CONFIG_PR=!CONFIG_PR!

echo @coderabbitai review> C:\Users\koosh\.cursor\coderabbit-comment.txt
echo @coderabbitai approve>> C:\Users\koosh\.cursor\coderabbit-comment.txt
gh pr comment !CONFIG_PR! --repo KooshaPari/Dino --body-file C:\Users\koosh\.cursor\coderabbit-comment.txt

echo === poll config PR up to 10 min ===
set /a N=0
:poll_config
set /a N+=1
gh api repos/KooshaPari/Dino/pulls/!CONFIG_PR!/reviews --jq "[.[] | select(.user.login==\"coderabbitai[bot]\" and .state==\"APPROVED\")] | length" > C:\Users\koosh\.cursor\cr-approve-count.txt
set /p CNT=<C:\Users\koosh\.cursor\cr-approve-count.txt
echo poll !N!: approved_count=!CNT!
if "!CNT!" GEQ "1" goto merge_config
if !N! GEQ 10 goto config_failed
timeout /t 60 /nobreak >nul
goto poll_config

:merge_config
gh pr merge !CONFIG_PR! --repo KooshaPari/Dino --merge
if errorlevel 1 exit /b 1

echo === PR 221 coderabbit approve ===
gh pr comment 221 --repo KooshaPari/Dino --body-file C:\Users\koosh\.cursor\coderabbit-comment.txt

set /a N=0
:poll_221
set /a N+=1
gh api repos/KooshaPari/Dino/pulls/221/reviews --jq "[.[] | select(.user.login==\"coderabbitai[bot]\" and .state==\"APPROVED\")] | length" > C:\Users\koosh\.cursor\cr-approve-count.txt
set /p CNT=<C:\Users\koosh\.cursor\cr-approve-count.txt
echo poll221 !N!: approved_count=!CNT!
if "!CNT!" GEQ "1" goto merge_221
if !N! GEQ 10 goto end_report
timeout /t 60 /nobreak >nul
goto poll_221

:merge_221
gh pr merge 221 --repo KooshaPari/Dino --merge
if errorlevel 1 exit /b 1

:sync_main
git checkout main
git pull

:end_report
echo === REPORT ===
gh pr view !CONFIG_PR! --repo KooshaPari/Dino --json number,state,mergedAt,headRefOid 2>nul
gh pr view 221 --repo KooshaPari/Dino --json number,state,mergedAt,headRefOid 2>nul
git log -1 --oneline origin/main
exit /b 0

:config_failed
echo CONFIG PR did not get coderabbitai bot APPROVED in 10 min
echo User must one-time approve the tiny config PR OR add AGENT_MERGE_PAT secret
exit /b 2
