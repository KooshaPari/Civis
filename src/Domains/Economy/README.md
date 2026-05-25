# DINOForge.Domains.Economy

NuGet package for resource management, trade, and production balancing in DINOForge.

## Overview

The Economy domain provides:
- **Production Models**: Define how resources are generated
- **Trade Engine**: Resource trading and bartering system
- **Balance Profiles**: Economy configuration and tuning
- **Resource Rates**: Production and consumption rates
- **Trade Routes**: Faction-to-faction trade relationships

## Installation

```bash
dotnet add package DINOForge.Domains.Economy --version 0.18.0
```

## Key Classes

- **ProductionCalculator**: Computes resource generation based on buildings/units
- **TradeEngine**: Manages trade agreements and transactions
- **BalanceProfileRegistry**: Stores economy configuration presets
- **ResourceRateRegistry**: Defines production and consumption rates
- **EconomyPlugin**: Main domain plugin initializer

## Usage Example

```csharp
using DINOForge.Domains.Economy;
using DINOForge.SDK.Registry;

// Resolve economy services
var tradeEngine = serviceProvider.GetRequiredService<TradeEngine>();
var calculator = serviceProvider.GetRequiredService<ProductionCalculator>();

// Calculate production for a faction
var production = calculator.CalculateProduction(factionId, buildings);
Console.WriteLine($"Wood per minute: {production.Wood}");

// Execute a trade
var result = tradeEngine.ExecuteTrade(seller, buyer, resource, amount);
```

## Testing

120+ economy tests in `src/Tests/EconomyTests/`:
- ProductionCalculatorTests
- TradeEngineTests
- BalanceProfileTests
- ResourceRateTests

Run tests:
```bash
dotnet test src/Tests/ --filter "Category=Economy"
```

## Dependencies

- **DINOForge.SDK** (0.18.0+) - Registry, validation, models

## License

MIT

## Contributing

See [CONTRIBUTING.md](https://github.com/KooshaPari/Dino/blob/main/CONTRIBUTING.md) in the main repository.
