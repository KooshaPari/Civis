import type {
  Biome,
  Building,
  CivPin,
  EconomySnapshot,
  Faction,
  InstitutionRow,
  Snapshot,
  TimeSpeed,
} from "../store";

/** Merge `sim.snapshot` JSON (with optional spectator fields) into dashboard `Snapshot`. */
export function mergeServerSnapshot(result: unknown, speed: TimeSpeed): Snapshot {
  const r = (result ?? {}) as Record<string, unknown>;
  const civPins = parseCivPins(r.civ_pins);
  const factions = parseFactions(r.factions);
  const buildings = parseBuildings(r.buildings);
  return {
    tick: Number(r.tick ?? 0),
    tick_dt_ms: 100,
    population: Number(r.population ?? 0),
    voxel_dirty_count: 0,
    voxel_chunk_count: 0,
    sample_civilians: [],
    civ_pins: civPins,
    factions,
    buildings,
    is_day: Boolean(r.is_day ?? true),
    economy: parseEconomyForServer(r),
    speed,
  };
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
    institutions: parseInstitutions(r.institutions),
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
