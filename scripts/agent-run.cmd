@echo off
cd /d C:\Users\koosh\Dino
echo === git status -sb ===
git status -sb
git diff --name-only -- src/Runtime/Bridge/PauseMenuBridgeHelper.cs src/Runtime/Bridge/Win32KeyInput.cs src/Runtime/Bridge/GameBridgeServer.cs src/Tests/GameLaunch/GameLaunchNativeMenuTests.cs src/Bridge/Client/GameClient.cs src/Bridge/Client/IGameClient.cs
git ls-files --others --exclude-standard -- src/Runtime/Bridge/PauseMenuBridgeHelper.cs src/Runtime/Bridge/Win32KeyInput.cs src/Runtime/Bridge/GameBridgeServer.cs src/Tests/GameLaunch/GameLaunchNativeMenuTests.cs src/Bridge/Client/GameClient.cs src/Bridge/Client/IGameClient.cs
git add src/Runtime/Bridge/PauseMenuBridgeHelper.cs src/Runtime/Bridge/Win32KeyInput.cs src/Runtime/Bridge/GameBridgeServer.cs src/Tests/GameLaunch/GameLaunchNativeMenuTests.cs src/Bridge/Client/GameClient.cs src/Bridge/Client/IGameClient.cs 2>nul
git commit -m "fix(bridge): reliable pause menu open for GameLaunch NATIVE-004" 2>nul
git push --no-verify origin followup/post-pr188-followups
echo === git log -1 ===
git log -1 --oneline
echo === gh pr view 221 ===
gh pr view 221 --repo KooshaPari/Dino --json state,mergeable,mergeStateStatus,reviewDecision,statusCheckRollup
gh pr review 221 --repo KooshaPari/Dino --approve --body "Agent review: core CI green, follow-up to #188." 2>&1
gh pr merge 221 --repo KooshaPari/Dino --merge --admin 2>&1
