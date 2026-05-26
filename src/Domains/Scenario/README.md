# DINOForge.Domains.Scenario

NuGet package for scenario scripting, victory conditions, and difficulty scaling in DINOForge.

## Overview

The Scenario domain provides:
- **Scenario Scripting**: Event-driven scenario execution
- **Victory Conditions**: Win/loss condition definitions and detection
- **Defeat Conditions**: Loss condition evaluation
- **Difficulty Scaling**: Dynamic challenge adjustment
- **Scripted Events**: Custom event triggers and handlers

## Installation

```bash
dotnet add package DINOForge.Domains.Scenario --version 0.18.0
```

## Key Classes

- **ScenarioRegistry**: Manages scenario definitions
- **ScenarioRunner**: Executes scenario logic and events
- **VictoryCondition**: Win condition base type
- **DefeatCondition**: Loss condition base type
- **DifficultyScaler**: Adjusts game parameters based on difficulty level
- **ScenarioValidator**: Validates scenario definitions for completeness

## Usage Example

```csharp
using DINOForge.Domains.Scenario;
using DINOForge.SDK.Registry;

// Load a scenario
var scenarioRegistry = serviceProvider.GetRequiredService<IRegistry<Scenario>>();
var scenario = scenarioRegistry.Get("tutorial-campaign");

// Run the scenario
var runner = serviceProvider.GetRequiredService<ScenarioRunner>();
runner.Initialize(scenario);
runner.Start();

// Check victory condition
if (scenario.VictoryCondition.IsMetAsync(world).Result)
{
    Console.WriteLine("Victory!");
}
```

## Testing

95+ scenario tests in `src/Tests/ScenarioTests/`:
- ScenarioRunnerTests
- VictoryConditionTests
- DefeatConditionTests
- DifficultyScalerTests
- ScenarioValidatorTests

Run tests:
```bash
dotnet test src/Tests/ --filter "Category=Scenario"
```

## Dependencies

- **DINOForge.SDK** (0.18.0+) - Registry, validation, models

## License

MIT

## Contributing

See [CONTRIBUTING.md](https://github.com/KooshaPari/Dino/blob/main/CONTRIBUTING.md) in the main repository.
