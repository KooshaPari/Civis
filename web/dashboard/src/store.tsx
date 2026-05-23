import React, { createContext, useContext, useReducer } from "react";

export type ToolKind = "PlaceVoxel" | "SpawnCivilian" | "DamageBomb" | "InspectAgent" | "Camera";
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
  job: JobLabel | null;
};

export type Snapshot = {
  tick: number;
  population: number;
  voxel_dirty_count: number;
  voxel_chunk_count: number;
  sample_civilians: SampleCivilian[];
  civ_pins: CivPin[];
  is_day: boolean;
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

type State = {
  selectedTool: ToolKind;
  speed: TimeSpeed;
  selectedMaterial: number;
  selectedEra: number;
  damageRadius: number;
  selectedFaction: number;
  selectedCivilian: CivilianFields | null;
  connection: "live" | "reconnecting" | "disconnected";
  snapshot: Snapshot | null;
  terrain: Terrain | null;
  inspectorOpen: boolean;
  toast: Toast | null;
};

type Action =
  | { type: "set_tool"; tool: ToolKind }
  | { type: "set_speed"; speed: TimeSpeed }
  | { type: "set_material"; material: number }
  | { type: "set_era"; era: number }
  | { type: "set_damage_radius"; radius: number }
  | { type: "set_selected_faction"; faction: number }
  | { type: "set_selected_civilian"; civilian: CivilianFields | null }
  | { type: "set_connection"; connection: State["connection"] }
  | { type: "set_snapshot"; snapshot: Snapshot | null }
  | { type: "set_terrain"; terrain: Terrain | null }
  | { type: "set_inspector_open"; open: boolean }
  | { type: "set_toast"; message: string | null }
  | { type: "clear_toast" };

const initialState: State = {
  selectedTool: "PlaceVoxel",
  speed: 1,
  selectedMaterial: 1,
  selectedEra: 0,
  damageRadius: 8,
  selectedFaction: 0,
  selectedCivilian: null,
  connection: "disconnected",
  snapshot: null,
  terrain: null,
  inspectorOpen: true,
  toast: null,
};

function reducer(state: State, action: Action): State {
  switch (action.type) {
    case "set_tool":
      return { ...state, selectedTool: action.tool };
    case "set_speed":
      return { ...state, speed: action.speed };
    case "set_material":
      return { ...state, selectedMaterial: action.material };
    case "set_era":
      return { ...state, selectedEra: action.era };
    case "set_damage_radius":
      return { ...state, damageRadius: action.radius };
    case "set_selected_faction":
      return { ...state, selectedFaction: action.faction };
    case "set_selected_civilian":
      return { ...state, selectedCivilian: action.civilian };
    case "set_connection":
      return { ...state, connection: action.connection };
    case "set_snapshot":
      return { ...state, snapshot: action.snapshot };
    case "set_terrain":
      return { ...state, terrain: action.terrain };
    case "set_inspector_open":
      return { ...state, inspectorOpen: action.open };
    case "set_toast":
      return {
        ...state,
        toast: action.message ? { id: Date.now(), message: action.message } : null,
      };
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
