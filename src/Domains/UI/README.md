# DINOForge.Domains.UI

NuGet package for HUD elements, menu management, and themes in DINOForge.

## Overview

The UI domain provides:
- **HUD Elements**: In-game health bars, unit info, resource counters
- **Menu Management**: Pause menu, settings, overlay integration
- **Theme System**: Customizable colors, fonts, layouts
- **UI Registries**: Centralized UI element registration
- **Event Handlers**: Input handling and UI interaction

## Installation

```bash
dotnet add package DINOForge.Domains.UI --version 0.18.0
```

## Key Classes

- **HudElementRegistry**: Manages in-game HUD elements
- **MenuRegistry**: Stores pause menu entries
- **ThemeRegistry**: Defines UI themes and styles
- **UiDomainPlugin**: Main domain plugin initializer
- **MenuManager**: Controls menu visibility and navigation

## Usage Example

```csharp
using DINOForge.Domains.UI;
using DINOForge.SDK.Registry;

// Register a custom HUD element
var hudRegistry = serviceProvider.GetRequiredService<IRegistry<HudElement>>();
var customHud = new HudElement 
{ 
    Id = "custom-info",
    Label = "Custom Info",
    Position = new Vector2(100, 100)
};
hudRegistry.Register(customHud);

// Apply a theme
var themeRegistry = serviceProvider.GetRequiredService<IRegistry<Theme>>();
var darkTheme = themeRegistry.Get("dark");
themeRegistry.ApplyTheme(darkTheme);
```

## Testing

250+ UI tests in `src/Tests/UiTests/`:
- HudElementTests
- MenuRegistryTests
- ThemeTests
- MenuManagerTests

Run tests:
```bash
dotnet test src/Tests/ --filter "Category=UI"
```

## Dependencies

- **DINOForge.SDK** (0.18.0+) - Registry, validation, models

## License

MIT

## Contributing

See [CONTRIBUTING.md](https://github.com/KooshaPari/Dino/blob/main/CONTRIBUTING.md) in the main repository.
