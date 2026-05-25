# Unit Role Matrix

Every faction in the Warfare domain fills the same 13 role slots with theme-appropriate units. This ensures mechanical balance while allowing visual/audio theming.

## The 13 Required Roles

| # | Slot | Tier | Description |
|---|------|------|-------------|
| 1 | T1 Cheap | 1 | Expendable early-game unit, low cost, low stats |
| 2 | T1 Core | 1 | Main line infantry, backbone of the army |
| 3 | T2 Elite | 2 | Premium infantry, high damage/survivability |
| 4 | T2 Anti-Armor | 2 | Specialized anti-vehicle/heavy unit |
| 5 | T2 Support | 2 | Suppression/support fire, MG equivalent |
| 6 | T2 Recon | 2 | Scout/reconnaissance, fast and fragile |
| 7 | T3 Light Vehicle | 3 | Fast vehicle, flanking and pursuit |
| 8 | T3 Heavy Vehicle | 3 | Main battle tank or heavy walker |
| 9 | T3 Artillery | 3 | Long-range bombardment |
| 10 | Defense 1 | — | Light static defense (MG nest, tower) |
| 11 | Defense 2 | — | Heavy static defense (AT, missile) |
| 12 | Commander | — | Hero/command unit with buffs |
| 13 | Spike Unit | — | Faction signature unit, unique capability |

## Full Faction Matrix

| Slot | Republic | CIS | West | Classic Enemy | Guerrilla |
|------|----------|-----|------|---------------|-----------|
| T1 Cheap | Militia clone | B1 | Militia | Conscript | Irregular |
| T1 Core | Clone trooper | B1 improved / B2 | Rifle squad | Rifle infantry | Raider rifle |
| T2 Elite | ARC trooper | Commando droid | Spec ops | Guards / assault | Veteran cell |
| T2 Anti-Armor | Clone AT | Heavy droid AT | ATGM team | AT team | RPG team |
| T2 Support | Repeater team | Heavy blaster | MG team | MG team | HMG / recoilless |
| T2 Recon | Scout trooper | Commando scout | Drone recon | Scout / mech recon | Infiltrator |
| T3 Light Vehicle | AT-RT / speeder | Dwarf / spider | Technical / IFV | IFV | Technical |
| T3 Heavy Vehicle | Walker / tank | AAT / heavy droid | MBT | MBT / assault gun | *(N/A)* |
| T3 Artillery | SP battery | Shelling droid | MLRS / howitzer | Tube + rocket arty | Mortar / rocket |
| Defense 1 | Blaster tower | Droid tower | MG nest | Bunker | Fortified nest |
| Defense 2 | Heavy laser | Shielded turret | AT / missile | Heavy missile | Rocket emplacement |
| Commander | Jedi / clone cmdr | Tactical relay | Officer / UAV | Commissar / cmd | Field emir |
| Spike Unit | Shield elite | Droideka | Drone strike | Thermobaric | Ambush wave |

## Role Details

### T1 Cheap (Expendable)

The early-game filler. Cheap to produce, dies fast, buys time. Every faction needs something to throw at a wall while real units train.

- **Unit class**: `MilitiaLight`
- **Behavior**: `Swarm`, `HoldLine`
- **Defense**: `Unarmored`, `Biological`

### T1 Core (Line Infantry)

The backbone. Reliable damage, reasonable survivability, produced in bulk through mid-game. The unit you build the most of.

- **Unit class**: `CoreLineInfantry`
- **Behavior**: `HoldLine`, `AdvanceFire`
- **Defense**: `InfantryArmor`, `Biological`

### T2 Elite (Premium Infantry)

Expensive but powerful. Used for key engagements, not mass production. Order factions get more value here; Swarm factions get less.

- **Unit class**: `EliteLineInfantry`
- **Behavior**: `AdvanceFire`
- **Defense**: `InfantryArmor`, `Biological`

### T2 Anti-Armor

Specialized counter to vehicles and heavy units. Low rate of fire, high single-target damage. Essential for breaking heavy pushes.

- **Unit class**: `AntiArmor`
- **Behavior**: `AntiArmor`, `HoldLine`
- **Weapon**: `ExplosiveAT` or `MissileGuided`

### T2 Support

Suppression fire and area denial. Holds lanes, forces enemies to slow down or take alternate routes. Machine guns, heavy blasters, recoilless rifles.

- **Unit class**: `HeavyInfantry`
- **Behavior**: `HoldLine`, `MoralePressure`
- **Weapon**: `SuppressionWeapon`

### T2 Recon

Fast, fragile scout. Provides vision, spots for artillery, harasses flanks. Drones in modern theme, speeder bikes in Star Wars.

- **Unit class**: `Recon`
- **Behavior**: `Kite`
- **Defense**: `Unarmored`

### T3 Light Vehicle

Fast vehicle for flanking, pursuit, and exploitation. Technicals, IFVs, AT-RTs, spider droids.

- **Unit class**: `FastVehicle`
- **Behavior**: `AdvanceFire`, `Charge`
- **Defense**: `Mechanical`

### T3 Heavy Vehicle

Main battle tank or equivalent. High armor, high damage, slow. Anchors defensive lines or spearheads assaults.

- **Unit class**: `MainBattleVehicle`
- **Behavior**: `AdvanceFire`, `AntiArmor`
- **Defense**: `HeavyArmor`, `Mechanical`

::: info
Guerrilla faction does not have a heavy vehicle. Their asymmetric doctrine relies on RPG teams and ambushes instead.
:::

### T3 Artillery

Long-range area bombardment. Expensive, devastating against formations. MLRS, howitzers, shelling droids, SP batteries.

- **Unit class**: `Artillery`
- **Behavior**: `SiegePriority`
- **Defense**: `Mechanical`

### Defense 1 (Light Static)

Basic defensive emplacement. MG nests, blaster towers, bunkers. Covers approaches and slows enemy advances.

- **Unit class**: `StaticMG`
- **Defense**: `Fortified`

### Defense 2 (Heavy Static)

Anti-armor or heavy defensive position. Missile emplacements, turbolasers, AT positions. Stops heavy pushes.

- **Unit class**: `StaticAT`
- **Defense**: `Fortified`

### Commander

Hero or command unit that provides buffs to nearby units. Jedi, officers, commissars, field emirs. One per army, powerful but vulnerable.

- **Unit class**: `HeroCommander`
- **Behavior**: `HoldLine`
- **Defense**: `Heroic`

### Spike Unit

Faction signature capability. Something unique that breaks the normal rules — droideka shields, drone strikes, thermobaric weapons, ambush waves.

- **Unit class**: `ShieldedElite` or varies by faction
- **Behavior**: Faction-specific
