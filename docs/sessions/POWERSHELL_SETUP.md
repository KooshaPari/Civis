# PowerShell Core Setup for DINOForge Development

## Status: ✅ Ready to Use

PowerShell Core 7 is already installed via winget. Here's the 3-minute setup:

## Quick Setup (3 minutes)

### Step 1: Open Windows Terminal
```
Windows Key → Type "Terminal" → Press Enter
```

### Step 2: Set PowerShell as Default
1. Click the dropdown arrow next to the "+" button
2. Select "Settings"
3. Go to "Startup" section
4. Set "Default profile" to "PowerShell"
5. Click "Save"

### Step 3: Close and Reopen Windows Terminal
- Close Terminal completely
- Open Windows Terminal again
- New tabs will now open in PowerShell by default

## Verify Setup

Run the test suite to confirm everything works:

```powershell
cd C:\Users\koosh\Dino
./test-cli.ps1
```

Expected output:
```
Testing: Help
  ✅ PASS (2.85s)

Testing: Validate Pack
  ✅ PASS (3.12s)

Testing: Unit Tests
  ✅ PASS (12.45s)

Results: 3 passed, 0 failed
```

## Available Commands

Once setup, you have these shortcuts in PowerShell:

```powershell
build                    # Build entire solution
test                     # Run unit tests
pack-validate <pack>     # Validate a pack
pack-build <pack>        # Build a pack
asset-import <pack>      # Import 3D models
asset-validate <pack>    # Validate assets
asset-optimize <pack>    # Generate LOD variants
asset-generate <pack>    # Generate prefabs
```

Example:
```powershell
build
test
pack-validate packs/warfare-starwars
asset-import packs/warfare-starwars
```

## Why PowerShell Instead of MSYS2?

| Feature | MSYS2 | PowerShell |
|---------|-------|-----------|
| .NET CLI | ❌ Hangs | ✅ Works |
| Build | ✅ Works | ✅ Works |
| Tests | ✅ Works | ✅ Works |
| Unix tools | ✅ Yes | ❌ No |
| Native Windows | ❌ Emulated | ✅ Native |
| .NET integration | ❌ Broken pipes | ✅ Perfect |

**For .NET development: Always use PowerShell**
**For Unix tools: Still use MSYS2 when needed**

## Troubleshooting

### Profile not loading?
```powershell
# Check profile path
$PROFILE

# Check if file exists
Test-Path $PROFILE

# If not, create it
New-Item -ItemType File -Path $PROFILE -Force
```

### PowerShell as default not working?
1. Restart your computer
2. Or manually launch: `pwsh` from Command Prompt

### Still seeing old prompt?
- Your profile is cached. Restart PowerShell completely.

## Keeping MSYS2

You can keep MSYS2 installed for Unix tools:
- Grep, sed, awk, find, diff
- GCC/MinGW compilation
- Make, autotools, CMake
- Bash scripting

Just don't use it for .NET work.

## What's Next?

1. ✅ PowerShell configured
2. ✅ CLI commands working
3. ⬜ Now you can build packs, import assets, run the full pipeline

Start with:
```powershell
build
test
pack-validate packs/warfare-starwars
```

All commands should complete successfully in PowerShell!
