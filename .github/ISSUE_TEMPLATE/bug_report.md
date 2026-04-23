---
name: Bug Report
description: Report a bug in DINOForge
title: "[Bug]: "
labels: ["bug", "triage"]
---

## Describe the Bug

A clear and concise description of what the bug is. Be specific about what went wrong and under what conditions.

## Steps to Reproduce

How can we reproduce this issue? Provide a minimal reproducible example.

1. Build with `dotnet build ...`
2. Run command `...`
3. Observe error...

## Expected Behavior

What did you expect to happen?

## Actual Behavior

What actually happened instead?

## Environment

Please provide your environment details:

- **DINOForge Version**: Version or commit hash (e.g., `0.23.0` or `2a2bed4`)
- **.NET Version**: Output of `dotnet --version`
- **OS**: Windows 11 / Windows 10 / WSL2 / Linux
- **Game Installation Path**: (if Runtime-related)
- **DINO Game Version**: Steam build ID or version if known

## Component

Which component is affected?

- [ ] Runtime (BepInEx plugin, ECS bridge)
- [ ] SDK (Registries, ContentLoader, schemas)
- [ ] PackCompiler (Pack compilation, validation)
- [ ] Warfare Domain
- [ ] Economy Domain
- [ ] Scenario Domain
- [ ] UI Domain
- [ ] Installer
- [ ] Desktop Companion
- [ ] MCP Server (Game automation)
- [ ] DumpTools
- [ ] Schemas
- [ ] Other

## Logs / Stack Trace

Paste any relevant logs, error output, or stack trace below. Use code blocks for readability:

<details>
<summary>Build Output (if applicable)</summary>

```
<paste build output here>
```

</details>

<details>
<summary>Game Log (BepInEx/dinoforge_debug.log)</summary>

```
<paste game logs here>
```

</details>

<details>
<summary>Stack Trace</summary>

```
<paste stack trace here>
```

</details>

## Additional Context

- [ ] I've checked existing issues for duplicates
- [ ] I can provide a minimal reproducible example
- [ ] I've tried troubleshooting steps (clean build, clear temp files, restart game)

Any other context, screenshots, or examples that might help us debug?
