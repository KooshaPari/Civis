# Development Tools

This page collects the runtime tools that help when developing DINOForge locally.

## UnityExplorer Integration

UnityExplorer is a runtime hierarchy inspector plus a C# REPL for BepInEx. It is useful for:

- Inspecting `MainMenuThemer` canvas structure and live UI objects.
- Inspecting runtime entity-adjacent state and component data while the game is running.
- Running quick C# expressions against live objects without restarting the game.

### Installation

Recommended installation through the DINOForge CLI:

```powershell
dinoforge dev-tools install
```

Manual installation is also possible if you are working outside the CLI flow:

1. Install the UnityExplorer BepInEx-compatible release.
2. Place the plugin files into the game’s BepInEx plugins directory.
3. Launch the game and confirm the plugin loads at startup.

### Keybinding

- `F7` toggles UnityExplorer in-game.

### Common Patterns

Find a `GameObject` by name:

```csharp
var go = UnityEngine.GameObject.Find("Main Menu");
```

Inspect components on a live object:

```csharp
var components = go.GetComponents<UnityEngine.Component>();
foreach (var component in components)
{
    UnityEngine.Debug.Log(component.GetType().FullName);
}
```

Evaluate C# against a runtime object:

```csharp
var rect = go.GetComponent<UnityEngine.RectTransform>();
UnityEngine.Debug.Log(rect.anchoredPosition);
```

## Other Recommended Tools

These are commonly useful alongside UnityExplorer:

- `AssemblyPublicizer` for exposing internal members during reverse-engineering or tooling work.
- `ScriptEngine` for running richer live scripts and automation snippets.
- `ThunderKit` for Unity mod pipeline and packaging workflows.
- `dnSpyEx` or `ILSpy` for assembly inspection and decompilation.
- `HarmonyX` diagnostics helpers when patching or debugging runtime hooks.

