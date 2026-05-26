# iter-143 v0.25.0 User Action Runbook

Follow these steps in order to complete iter-143 + release v0.25.0.

## 1. Verify UI Fix in Game

```powershell
# Kill all DINO instances
Stop-Process -Name 'Diplomacy is Not an Option' -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 3

# Launch fresh game instance
Start-Process -FilePath 'G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe' `
  -WorkingDirectory 'G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option'
Start-Sleep -Seconds 8

# Check: menu clickable, no chicken skeletons in loading-circle/publisher-ad
# F10 overlay should respond to clicks
```

## 2. Commit Work

See `docs/sessions/iter-143-COMMIT-DRAFTS.md` for 11 commit messages. Execute each:

```powershell
git add src/Bridge/Client/GameClient.cs
git commit -m "fix(bridge): reinstate Connection ctor logic + thread-safe nullable"

# ... repeat for remaining 10 commits per drafts doc
```

## 3. Push Fix Branch

```powershell
git push -u origin fix/handle-connect-iter142
```

## 4. Open PR

```powershell
gh pr create --title "fix(bridge): Reinstate Connection ctor + thread-safe nullable (iter-143)" `
  --body "$(Get-Content docs/sessions/iter-142-PR-DESCRIPTION-READY.md -Raw)"
```

## 5. Merge PR

Via GitHub UI or:
```powershell
gh pr merge --squash  # or --rebase or --create-issue
```

## 6. Tag v0.25.0

```powershell
git checkout main
git pull
git tag v0.25.0
git push origin v0.25.0
```

release.yml auto-fires on tag push.

## 7. Verify Release

```powershell
gh release view v0.25.0

# Confirm: NuGet packages published to nuget.org
# Verify: SDK + Bridge.Protocol .nupkg + .snupkg visible
```

---

**Timing**: Steps 1-7 ≈ 15 min (8 min game restart, 2 min commits, 5 min GitHub/tag).

**Success Criteria**:
- Game launches cleanly, menu responsive
- 11 commits on branch, cleanly merged to main
- v0.25.0 tag exists, release artifacts published
