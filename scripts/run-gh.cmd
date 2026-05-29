@echo off
cd /d C:\Users\koosh\Dino
gh workflow run agent-merge-on-bot-approve.yml --repo KooshaPari/Dino --ref followup/post-pr188-followups -f pr_number=221 > C:\Users\koosh\Dino\scripts\gh-out.txt 2>&1
echo EXIT=%ERRORLEVEL%>> C:\Users\koosh\Dino\scripts\gh-out.txt
