# add-unit

Add a new unit definition to an existing pack.

**Usage**: `/add-unit <pack-id> <unit-id> <unit-class>`

**Arguments**: $ARGUMENTS

## Valid Unit Classes

`MilitiaLight`, `CoreLineInfantry`, `EliteLineInfantry`, `HeavyInfantry`, `Skirmisher`, `AntiArmor`, `ShockMelee`, `SwarmFodder`, `FastVehicle`, `MainBattleVehicle`, `HeavySiege`, `Artillery`, `WalkerHeavy`, `StaticMG`, `StaticAT`, `StaticArtillery`, `SupportEngineer`, `Recon`, `HeroCommander`, `AirstrikeProxy`, `ShieldedElite`

## Steps

1. Verify `packs/<pack-id>/pack.yaml` exists
2. Read the pack manifest to understand its structure
3. Create or append to `packs/<pack-id>/units/<unit-id>.yaml` with a well-formed UnitDefinition
4. Use the provided unit-class to determine archetype defaults
5. Validate: `dotnet run --project src/Tools/PackCompiler -- validate packs/<pack-id>/`
6. Report the created unit and any validation results

This command scaffolds unit definitions with sensible defaults for the chosen archetype.
