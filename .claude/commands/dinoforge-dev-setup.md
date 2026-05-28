# dinoforge-dev-setup

First-time developer environment setup: clone repo, install tools, build, and deploy.

**Usage**: `/dinoforge-dev-setup [--skip-clone] [--install-unity-explorer] [--target-path <game-path>]`

**Arguments**:
- `--skip-clone`: If you've already cloned the repo, skip the clone step
- `--install-unity-explorer`: Also install UnityExplorer for advanced debugging (optional)
- `--target-path`: Path to DINO game directory (auto-detected from Steam registry if not provided)

## Purpose

Sets up a complete DINOForge development environment from scratch. Suitable for:
- New team member onboarding
- Local development machine setup
- CI/CD environment preparation

## Steps

1. **Clone repository** (unless `--skip-clone`):
   ```
   git clone https://github.com/KooshaPari/Dino.git
   cd Dino
   ```

2. **Verify prerequisites**:
   - .NET 11 SDK (check: `dotnet --version`, must be >= 11.0.100-preview)
   - Git (check: `git --version`)
   - Game path detected or provided (check: `{game-path}\Diplomacy is Not an Option.exe` exists)

3. **Install .NET 11 preview** (if needed):
   ```
   # Windows (via Installer)
   # macOS/Linux (via dotnet-install.sh)
   ```

4. **Build solution**:
   ```
   dotnet build src/DINOForge.sln -c Release
   ```

5. **Deploy to game** (BepInEx plugin):
   ```
   dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release -p:DeployToGame=true
   ```

6. **Optional: Install UnityExplorer** (if `--install-unity-explorer`):
   ```
   dotnet run --project src/Tools/Cli -- dev-tools install --unity-explorer
   ```

7. **Verify deployment**:
   - Check DLL exists: `{game-path}\BepInEx\plugins\DINOForge.Runtime.dll`
   - Check file size > 500KB (sanity check)
   - Report hash: `Get-FileHash ... -Algorithm SHA256`

8. **Next steps (printed)**:
   - Launch game with `/launch-game`
   - Run smoke test with `/dinoforge-smoke`
   - Create a test pack with `/dinoforge-pack-new`
   - Check the troubleshooting guide if anything fails

## Time

~3 minutes (clone: 30s + build: 90s + deploy: 15s + verification: 10s).

## Use When

- Setting up a new dev machine
- Onboarding a new contributor
- Automated environment setup (CI/CD)
- Complete clean slate is needed

## Requirements

- **Windows 10+** with PowerShell 7+ or Bash (WSL2)
- **Internet** (to clone and download NuGet packages)
- **Disk space**: ~5 GB (repo + .NET packages + build artifacts)
- **DINO game** installed via Steam (for BepInEx integration)
- **.NET 11 SDK** (auto-detected or installed by this script)

## Troubleshooting

If the setup fails:

1. **Build errors**: Check `.NET version` with `dotnet --version`
2. **Game path not found**: Provide `--target-path "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"`
3. **Deploy failed**: Check BepInEx folder permissions (write access required)
4. **Hash mismatch**: Run build again; may be caching issue

Run `/check-game` after setup to verify runtime is loaded.
