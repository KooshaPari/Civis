import React, { createContext, useContext, useReducer } from "react";
import type { ServerMetrics } from "./lib/civisServer";
import { pushFrameSample } from "./lib/framePerf";
import { readStoredTheme, type ThemeMode } from "./lib/theme";

export type ToolKind = "PlaceVoxel" | "SpawnCivilian" | "DamageBomb" | "InspectAgent" | "Camera";

export type SpawnKind = "civilian" | "vehicle" | "airport";
export type CameraPreset = "wide" | "close" | "orbit";
export type TimeSpeed = 0 | 1 | 2 | 4 | 8;

export type JobLabel = "farmer" | "warrior" | "scholar" | "trader" | "priest" | "admin" | "unemployed";

export type Biome = "deepwater" | "water" | "sand" | "grass" | "forest" | "stone" | "snow";

export type Terrain = {
  size: number;
  heights: number[];
  biomes: Biome[];
};

export type SampleCivilian = {
  age: number;
  health: number;
  ideology: number;
  welfare: number;
  job: JobLabel | null;
};

export type CivPin = {
  idx: number;
  x: number;
  y: number;
  dx: number;
  dy: number;
  job: JobLabel | null;
};

export type MilitaryPin = {
  id: number;
  x: number;
  y: number;
  unit_type: string;
  faction: number;
  strength: number;
};

export type Faction = {
  id: number;
  color: [number, number, number];
  capital: [number, number];
  radius: number;
};

export type BuildingKind = "Residential" | "Commercial" | "Industrial" | "Civic";

export type Building = {
  id: number;
  x: number;
  y: number;
  kind: BuildingKind;
  era: number;
  faction_id: number;
  occupants: number;
  capacity: number;
};

export type HousingStats = {
  total_capacity: number;
  occupied: number;
  homeless: number;
  vacancy_rate: number;
};

export type RoadKind = "Trail" | "Dirt" | "Paved" | "Highway";

export type Road = {
  from: [number, number];
  to: [number, number];
  width: number;
  kind: RoadKind;
};

export type TradeRoute = {
  from_faction: number;
  to_faction: number;
  goods: string;
  volume: number;
};

export type GameEvent = {
  tick: number;
  kind: string;
  message: string;
  faction_id: number | null;
};

export type TechNode = {
  id: string;
  kind: string;
  era_min: number;
  unlocked: boolean;
};

export type FactionTreasury = {
  id: number;
  name: string;
  balance: number;
};

export type ProductionRates = {
  food_per_tick: number;
  wood_per_tick: number;
  metal_per_tick: number;
  energy_per_tick: number;
};

export type InstitutionRow = {
  id: number;
  kind: string;
  balance_joules: number;
};

export type EconomySnapshot = {
  energy_budget: number;
  faction_treasury: FactionTreasury[];
  production_rates: ProductionRates;
  institutions: InstitutionRow[];
  resources: ResourceSnapshot;
};

export type ResourceSnapshot = {
  food: number;
  wood: number;
  metal: number;
  energy: number;
};

export type PopulationPulse = {
  tick: number;
  entity_id: number;
  x: number;
  y: number;
};

export type DiplomacyKind = "TradeAgreement" | "Conflict" | "Peace";

export type DiplomacyEvent = {
  tick: number;
  faction_a: number;
  faction_b: number;
  kind: DiplomacyKind;
};

export type DamagePulse = {
  x: number;
  y: number;
};

export type WeatherSnapshot = {
  season: string;
  temperature: number;
  wind_speed: number;
  precipitation: "none" | "rain" | "snow";
};

export type Snapshot = {
  tick: number;
  tick_dt_ms: number;
  current_era: number;
  population: number;
  voxel_dirty_count: number;
  voxel_chunk_count: number;
  sample_civilians: SampleCivilian[];
  civ_pins: CivPin[];
  military_units: MilitaryPin[];
  factions: Faction[];
  buildings: Building[];
  housing_stats: HousingStats;
  roads?: Road[];
  trade_routes?: TradeRoute[];
  economy: EconomySnapshot;
  births_this_tick: number;
  deaths_this_tick: number;
  diplomacy_events: DiplomacyEvent[];
  damage_events: DamagePulse[];
  birth_events: PopulationPulse[];
  death_events: PopulationPulse[];
  tech_tree: TechNode[];
  events: GameEvent[];
  is_day: boolean;
  weather: WeatherSnapshot;
  speed: TimeSpeed;
};

export type CivilianFields = SampleCivilian & {
  id?: string | number;
  x?: number;
  y?: number;
  z?: number;
  speed?: number;
  hunger?: number;
  happiness?: number;
  name?: string;
};

type Toast = {
  id: number;
  message: string;
};

export type SaveEntry = {
  name: string;
  size_bytes: number;
  modified: number | null;
};

export type { ThemeMode } from "./lib/theme";

export type AttachMode = "watch" | "server";

export type FrameSampleSource = "idle" | "attach" | "mock";

type State = {
  attachMode: AttachMode;
  /** When true, terrain tools are inspect-only (?spectator=1). */
  readOnly: boolean;
  selectedTool: ToolKind;
  speed: TimeSpeed;
  selectedMaterial: number;
  selectedEra: number;
  damageRadius: number;
  selectedFaction: number;
  spawnKind: SpawnKind;
  /** Bumped when the user picks a camera preset (FR-CIV-UX-005). */
  cameraPresetToken: number;
  cameraPreset: CameraPreset | null;
  selectedCivilian: CivilianFields | null;
  selectedMilitary: MilitaryPin | null;
  connection: "live" | "reconnecting" | "disconnected";
  snapshot: Snapshot | null;
  serverMetrics: ServerMetrics | null;
  frame3dTick: number | null;
  loadedChunkCount: number;
  loadedChunkIds: number[];
  recentChunkIds: number[];
  inspectedChunkId: number | null;
  seenAgentCount: number;
  recentAgentIds: number[];
  frameSamples: number[];
  frameSampleSource: FrameSampleSource;
  terrain: Terrain | null;
  inspectorOpen: boolean;
  activeSideTab: "inspector" | "events";
  economyPanelOpen: boolean;
  techTreeOpen: boolean;
  theme: ThemeMode;
  toast: Toast | null;
  lastSaveTick: number | null;
};

type Action =
  | { type: "set_tool"; tool: ToolKind }
  | { type: "set_speed"; speed: TimeSpeed }
  | { type: "set_theme"; theme: ThemeMode }
  | { type: "set_material"; material: number }
  | { type: "set_era"; era: number }
  | { type: "set_damage_radius"; radius: number }
  | { type: "set_selected_faction"; faction: number }
  | { type: "set_spawn_kind"; kind: SpawnKind }
  | { type: "set_camera_preset"; preset: CameraPreset }
  | { type: "set_selected_civilian"; civilian: CivilianFields | null }
  | { type: "set_selected_military"; military: MilitaryPin | null }
  | { type: "set_connection"; connection: State["connection"] }
  | { type: "set_snapshot"; snapshot: Snapshot | null }
  | { type: "set_server_metrics"; metrics: ServerMetrics | null }
  | { type: "set_frame3d_tick"; tick: number | null }
  | { type: "set_chunk_stats"; count: number; recentIds: number[]; loadedIds: number[] }
  | { type: "set_inspected_chunk"; chunkId: number | null }
  | { type: "set_agent_stats"; count: number; recentIds: number[] }
  | { type: "push_frame_sample"; ms: number; source?: FrameSampleSource }
  | { type: "set_frame_sample_source"; source: FrameSampleSource }
  | { type: "reset_frame_samples" }
  | { type: "set_attach_mode"; mode: AttachMode }
  | { type: "set_read_only"; readOnly: boolean }
  | { type: "set_terrain"; terrain: Terrain | null }
  | { type: "set_inspector_open"; open: boolean }
  | { type: "set_active_side_tab"; tab: State["activeSideTab"] }
  | { type: "set_economy_panel_open"; open: boolean }
  | { type: "set_tech_tree_open"; open: boolean }
  | { type: "set_toast"; message: string | null }
  | { type: "set_last_save_tick"; tick: number | null }
  | { type: "clear_toast" };

const initialState: State = {
  attachMode: "server",
  readOnly: false,
  selectedTool: "InspectAgent",
  speed: 1,
  selectedMaterial: 1,
  selectedEra: 0,
  damageRadius: 8,
  selectedFaction: 0,
  spawnKind: "civilian",
  cameraPresetToken: 0,
  cameraPreset: null,
  selectedCivilian: null,
  selectedMilitary: null,
  connection: "disconnected",
  snapshot: null,
  serverMetrics: null,
  frame3dTick: null,
  loadedChunkCount: 0,
  loadedChunkIds: [],
  recentChunkIds: [],
  inspectedChunkId: null,
  seenAgentCount: 0,
  recentAgentIds: [],
  frameSamples: [],
  frameSampleSource: "idle",
  terrain: null,
  inspectorOpen: true,
  activeSideTab: "inspector",
  economyPanelOpen: true,
  techTreeOpen: false,
  theme: readStoredTheme(
    typeof window !== "undefined"
      ? { search: window.location.search }
      : {},
  ),
  toast: null,
  lastSaveTick: null,
};

function reducer(state: State, action: Action): State {
  switch (action.type) {
    case "set_tool":
      return { ...state, selectedTool: action.tool };
    case "set_speed":
      return { ...state, speed: action.speed };
    case "set_theme":
      return { ...state, theme: action.theme };
    case "set_material":
      return { ...state, selectedMaterial: action.material };
    case "set_era":
      return { ...state, selectedEra: action.era };
    case "set_damage_radius":
      return { ...state, damageRadius: action.radius };
    case "set_selected_faction":
      return { ...state, selectedFaction: action.faction };
    case "set_spawn_kind":
      return { ...state, spawnKind: action.kind };
    case "set_camera_preset":
      return {
        ...state,
        cameraPreset: action.preset,
        cameraPresetToken: state.cameraPresetToken + 1,
      };
    case "set_selected_civilian":
      return { ...state, selectedCivilian: action.civilian };
    case "set_selected_military":
      return { ...state, selectedMilitary: action.military };
    case "set_connection":
      return { ...state, connection: action.connection };
    case "set_snapshot":
      return { ...state, snapshot: action.snapshot };
    case "set_server_metrics":
      return { ...state, serverMetrics: action.metrics };
    case "set_frame3d_tick":
      return { ...state, frame3dTick: action.tick };
    case "set_chunk_stats":
      return {
        ...state,
        loadedChunkCount: action.count,
        recentChunkIds: action.recentIds,
        loadedChunkIds: action.loadedIds,
      };
    case "set_inspected_chunk":
      return { ...state, inspectedChunkId: action.chunkId };
    case "set_agent_stats":
      return {
        ...state,
        seenAgentCount: action.count,
        recentAgentIds: action.recentIds,
      };
    case "push_frame_sample":
      return {
        ...state,
        frameSamples: pushFrameSample(state.frameSamples, action.ms),
        frameSampleSource: action.source ?? state.frameSampleSource,
      };
    case "set_frame_sample_source":
      return { ...state, frameSampleSource: action.source };
    case "reset_frame_samples":
      return {
        ...state,
        frameSamples: [],
        frameSampleSource: "idle",
        frame3dTick: null,
        loadedChunkCount: 0,
        loadedChunkIds: [],
        recentChunkIds: [],
        inspectedChunkId: null,
        seenAgentCount: 0,
        recentAgentIds: [],
      };
    case "set_attach_mode":
      return { ...state, attachMode: action.mode };
    case "set_read_only":
      return { ...state, readOnly: action.readOnly };
    case "set_terrain":
      return { ...state, terrain: action.terrain };
    case "set_inspector_open":
      return { ...state, inspectorOpen: action.open };
    case "set_active_side_tab":
      return { ...state, activeSideTab: action.tab, inspectorOpen: true };
    case "set_economy_panel_open":
      return { ...state, economyPanelOpen: action.open };
    case "set_tech_tree_open":
      return { ...state, techTreeOpen: action.open };
    case "set_toast":
      return {
        ...state,
        toast: action.message ? { id: Date.now(), message: action.message } : null,
      };
    case "set_last_save_tick":
      return { ...state, lastSaveTick: action.tick };
    case "clear_toast":
      return { ...state, toast: null };
    default:
      return state;
  }
}

type StoreValue = {
  state: State;
  dispatch: React.Dispatch<Action>;
};

const StoreContext = createContext<StoreValue | null>(null);

export function StoreProvider({ children }: { children: React.ReactNode }) {
  const [state, dispatch] = useReducer(reducer, initialState);
  return <StoreContext.Provider value={{ state, dispatch }}>{children}</StoreContext.Provider>;
}

export function useDashboardStore() {
  const value = useContext(StoreContext);
  if (!value) {
    throw new Error("useDashboardStore must be used within StoreProvider");
  }
  return value;
}

// postControl lives in `./control` to avoid two competing implementations;
// import from there.
