# Your First Mod

Create a DINOForge mod pack in 5 minutes.

## Prerequisites

- [.NET 11 SDK](https://dotnet.microsoft.com/download/dotnet/11.0) installed
- DINOForge source cloned (`git clone https://github.com/KooshaPari/Dino`)
- Game installed at default Steam path

## 1. Scaffold Your Pack

```bash
cd Dino
dotnet run --project src/Tools/Cli -- new my-first-mod --author "YourName"
```

This creates `packs/my-first-mod/` with:
- `pack.yaml` — manifest (ID, version, dependencies)
- `units/example-unit.yaml` — sample unit override
- `stats/balance.yaml` — sample stat tweaks
- `README.md` — pack documentation template

## 2. Edit Your Pack

Open `packs/my-first-mod/pack.yaml`:

```yaml
id: my-first-mod
name: My First Mod
version: 0.1.0
framework_version: ">=0.24.0 <0.26.0"
author: YourName
type: content
depends_on: []
conflicts_with: []
```

Add a stat override in `stats/balance.yaml`:

```yaml
id: archer-buff
target: vanilla-archer
overrides:
  - stat: Health
    value: 150
    mode: Override
  - stat: AttackDamage
    value: 25
    mode: Override
```

## 3. Validate

```bash
dotnet run --project src/Tools/PackCompiler -- validate packs/my-first-mod
```

You should see: `Validation successful!`

## 4. Deploy & Test

```bash
# Build the Runtime DLL with your pack
dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release -p:TargetFramework=netstandard2.0

# Copy DLL to game
copy src\Runtime\bin\Release\netstandard2.0\DINOForge.Runtime.dll "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\plugins\"
```

Launch the game. Press **F10** to open the mod menu — your pack should appear in the list.

## 5. Iterate

Edit your YAML files and press **F10 → Reload Packs** to see changes without restarting.

For live file watching, the HMR system auto-detects changes after 15 seconds.

## What's Next?

- Add **factions** in `factions/` subdirectory
- Add **buildings** in `buildings/` subdirectory  
- Add **scenarios** with victory/defeat conditions
- Check `packs/warfare-starwars/` for a complete example with 28 units + 10 buildings
- Run `dinoforge verify-pack packs/my-first-mod` with the game running for live verification

## Pack Types

| Type | Use Case |
|------|----------|
| `content` | Units, buildings, factions — additive content |
| `balance` | Stat overrides only — no new content |
| `ruleset` | Game rule modifications |
| `scenario` | Campaign scenarios with conditions |
| `total_conversion` | Full game overhaul |
| `utility` | Tools and helpers |

## Troubleshooting

- **Pack not showing in F10?** Check `framework_version` — must match installed DINOForge version
- **YAML parse error?** Run `dotnet run --project src/Tools/PackCompiler -- validate packs/my-first-mod`
- **Stat override not applying?** Ensure `target` matches a vanilla entity ID (check F8 dump)
- **F10 empty?** Press F9 to check Platform Status — verify packs loaded count > 0
