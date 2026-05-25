import type {
  Building,
  CivPin,
  MilitaryPin,
  DiplomacyEvent,
  DiplomacyKind,
  DamagePulse,
  EconomySnapshot,
  Faction,
  InstitutionRow,
  PopulationPulse,
  Road,
  RoadKind,
  Snapshot,
  TechNode,
  TimeSpeed,
  TradeRoute,
} from "../store";

function parseTechTree(raw: unknown): TechNode[] {
  if (!Array.isArray(raw)) return [];
  return raw.map((row) => {
    const item = row as Record<string, unknown>;
    return {
      id: String(item.id ?? ""),
      kind: String(item.kind ?? ""),
      era_min: Number(item.era_min ?? 0),
      unlocked: Boolean(item.unlocked ?? false),
    };
  });
}

/** Merge `sim.snapshot` JSON (with optional spectator fields) into dashboard `Snapshot`. */
export function mergeServerSnapshot(result: unknown, speed: TimeSpeed): Snapshot {
  const r = (result ?? {}) as Record<string, unknown>;
  const civPins = parseCivPins(r.civ_pins);
  const militaryUnits = parseMilitaryPins(r.military_units);
  const factions = parseFactions(r.factions);
  const buildings = parseBuildings(r.buildings);
  const roads = parseRoads(r.roads);
  const trade_routes = parseTradeRoutes(r.trade_routes);
  return {
    tick: Number(r.tick ?? 0),
    tick_dt_ms: Number(r.tick_dt_ms ?? 100),
    current_era: Number(r.current_era ?? 0),
    population: Number(r.population ?? 0),
    voxel_dirty_count: Number(r.voxel_dirty_count ?? 0),
    voxel_chunk_count: Number(r.voxel_chunk_count ?? 0),
    sample_civilians: [],
    civ_pins: civPins,
    military_units: militaryUnits,
    factions,
    buildings,
    roads,
    trade_routes,
    births_this_tick: Number(r.births_this_tick ?? 0),
    deaths_this_tick: Number(r.deaths_this_tick ?? 0),
    diplomacy_events: parseDiplomacyEvents(r.diplomacy_events),
    damage_events: parseDamageEvents(r.damage_events),
    birth_events: parsePopulationPulses(r.birth_events),
    death_events: parsePopulationPulses(r.death_events),
    tech_tree: parseTechTree(r.tech_tree),
    is_day: Boolean(r.is_day ?? true),
    economy: parseEconomyForServer(r),
    speed,
  };
}

function parsePopulationPulses(raw: unknown): PopulationPulse[] {
  if (!Array.isArray(raw)) return [];
  return raw.map((row) => {
    const item = row as Record<string, unknown>;
    return {
      tick: Number(item.tick ?? 0),
      entity_id: Number(item.entity_id ?? 0),
      x: Number(item.x ?? 0),
      y: Number(item.y ?? 0),
    };
  });
}

function parseDiplomacyKind(kind: unknown): DiplomacyKind {
  if (kind === "TradeAgreement" || kind === "Conflict" || kind === "Peace") return kind;
  return "Peace";
}

function parseDiplomacyEvents(raw: unknown): DiplomacyEvent[] {
  if (!Array.isArray(raw)) return [];
  return raw.map((row) => {
    const item = row as Record<string, unknown>;
    return {
      tick: Number(item.tick ?? 0),
      faction_a: Number(item.faction_a ?? 0),
      faction_b: Number(item.faction_b ?? 0),
      kind: parseDiplomacyKind(item.kind),
    };
  });
}

function parseDamageEvents(raw: unknown): DamagePulse[] {
  if (!Array.isArray(raw)) return [];
  return raw.map((row) => {
    const item = row as Record<string, unknown>;
    return {
      x: Number(item.x ?? 0),
      y: Number(item.y ?? 0),
    };
  });
}

function parseInstitutions(raw: unknown): InstitutionRow[] {
  if (!Array.isArray(raw)) return [];
  return raw.map((row) => {
    const item = row as Record<string, unknown>;
    return {
      id: Number(item.id ?? 0),
      kind: String(item.kind ?? "unknown"),
      balance_joules: Number(item.balance_joules ?? 0),
    };
  });
}

function parseEconomy(raw: unknown): EconomySnapshot {
  const r = (raw ?? {}) as Record<string, unknown>;
  const treasury = Array.isArray(r.faction_treasury)
    ? r.faction_treasury.map((row) => {
        const item = row as Record<string, unknown>;
        return {
          id: Number(item.id ?? 0),
          name: String(item.name ?? "Faction"),
          balance: Number(item.balance ?? 0),
        };
      })
    : [];
  const production = (r.production_rates ?? {}) as Record<string, unknown>;
  return {
    energy_budget: Number(r.energy_budget ?? 0),
    faction_treasury: treasury,
    production_rates: {
      food_per_tick: Number(production.food_per_tick ?? 0),
      wood_per_tick: Number(production.wood_per_tick ?? 0),
      metal_per_tick: Number(production.metal_per_tick ?? 0),
      energy_per_tick: Number(production.energy_per_tick ?? 0),
    },
    institutions: parseInstitutions(r.institutions ?? (r.economy as Record<string, unknown> | undefined)?.institutions),
    resources: {
      food: Number((r.resources as Record<string, unknown> | undefined)?.food ?? 0),
      wood: Number((r.resources as Record<string, unknown> | undefined)?.wood ?? 0),
      metal: Number((r.resources as Record<string, unknown> | undefined)?.metal ?? 0),
      energy: Number((r.resources as Record<string, unknown> | undefined)?.energy ?? 0),
    },
  };
}

/** Merge nested `economy` plus top-level `sim.snapshot` economy fields. */
function parseEconomyForServer(root: Record<string, unknown>): EconomySnapshot {
  const nested = parseEconomy(root.economy);
  const energy =
    root.energy_budget != null ? Number(root.energy_budget) : nested.energy_budget;
  const institutions =
    parseInstitutions(root.institutions).length > 0
      ? parseInstitutions(root.institutions)
      : nested.institutions;
  return { ...nested, energy_budget: energy, institutions };
}

function parseCivPins(raw: unknown): CivPin[] {
  if (!Array.isArray(raw)) return [];
  return raw.map((pin, idx) => {
    const p = pin as Record<string, unknown>;
    return {
      idx: Number(p.idx ?? idx),
      x: Number(p.x ?? 0),
      y: Number(p.y ?? 0),
      dx: Number(p.dx ?? 0),
      dy: Number(p.dy ?? 0),
      job: parseJob(p.job),
    };
  });
}

function parseMilitaryPins(raw: unknown): MilitaryPin[] {
  if (!Array.isArray(raw)) return [];
  return raw.map((unit, idx) => {
    const u = unit as Record<string, unknown>;
    return {
      id: Number(u.id ?? idx),
      x: Number(u.x ?? 0),
      y: Number(u.y ?? 0),
      unit_type: String(u.unit_type ?? "Soldier"),
      faction: Number(u.faction ?? 0),
      strength: Number(u.strength ?? 0),
    };
  });
}

function parseFactions(raw: unknown): Faction[] {
  if (!Array.isArray(raw)) return [];
  return raw.map((f) => {
    const row = f as Record<string, unknown>;
    const color = row.color as number[] | undefined;
    const capital = row.capital as number[] | undefined;
    return {
      id: Number(row.id ?? 0),
      color: [color?.[0] ?? 128, color?.[1] ?? 128, color?.[2] ?? 128] as [number, number, number],
      capital: [capital?.[0] ?? 0.5, capital?.[1] ?? 0.5] as [number, number],
      radius: Number(row.radius ?? 10),
    };
  });
}

function parseBuildings(raw: unknown): Building[] {
  if (!Array.isArray(raw)) return [];
  return raw.map((b) => {
    const row = b as Record<string, unknown>;
    return {
      id: Number(row.id ?? 0),
      x: Number(row.x ?? 0),
      y: Number(row.y ?? 0),
      kind: parseBuildingKind(row.kind),
      era: Number(row.era ?? 0),
      faction_id: Number(row.faction_id ?? 0),
    };
  });
}

function parseJob(job: unknown): CivPin["job"] {
  if (typeof job !== "string") return null;
  const j = job.toLowerCase();
  if (j === "farmer") return "farmer";
  if (j === "warrior") return "warrior";
  if (j === "scholar") return "scholar";
  if (j === "trader") return "trader";
  if (j === "priest") return "priest";
  if (j === "admin") return "admin";
  return "unemployed";
}

function parseBuildingKind(kind: unknown): Building["kind"] {
  if (typeof kind !== "string") return "Residential";
  if (kind === "Commercial") return "Commercial";
  if (kind === "Industrial") return "Industrial";
  if (kind === "Civic") return "Civic";
  return "Residential";
}

function parseRoadKind(kind: unknown): RoadKind {
  if (kind === "Trail" || kind === "Dirt" || kind === "Paved" || kind === "Highway") return kind;
  return "Dirt";
}

function parseRoads(raw: unknown): Road[] {
  if (!Array.isArray(raw)) return [];
  return raw.map((row) => {
    const item = row as Record<string, unknown>;
    const from = item.from as number[] | undefined;
    const to = item.to as number[] | undefined;
    return {
      from: [Number(from?.[0] ?? 0), Number(from?.[1] ?? 0)] as [number, number],
      to: [Number(to?.[0] ?? 0), Number(to?.[1] ?? 0)] as [number, number],
      width: Number(item.width ?? 0.02),
      kind: parseRoadKind(item.kind),
    };
  });
}

function parseTradeRoutes(raw: unknown): TradeRoute[] {
  if (!Array.isArray(raw)) return [];
  return raw.map((row) => {
    const item = row as Record<string, unknown>;
    return {
      from_faction: Number(item.from_faction ?? 0),
      to_faction: Number(item.to_faction ?? 0),
      goods: String(item.goods ?? ""),
      volume: Number(item.volume ?? 0),
    };
  });
}
