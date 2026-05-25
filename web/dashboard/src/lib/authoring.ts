import { postControl } from "../control";
import type { AttachMode, SpawnKind, TimeSpeed, ToolKind } from "../store";
import { jsonRpcCall, normalizeServerSnapshot } from "./civisServer";
import { getActiveServerSocket } from "./civisSocket";
import { mergeServerSnapshot } from "./mergeSnapshot";

const FIXED_SCALE = 1_000_000;

export type TerrainAuthoringInput = {
  attachMode: AttachMode;
  speed: TimeSpeed;
  tool: ToolKind;
  cellX: number;
  cellY: number;
  terrainSize: number;
  heightY: number;
  material: number;
  faction: number;
  damageRadius: number;
  spawnKind: SpawnKind;
};

type AuthoringDispatch = {
  set_snapshot: (snapshot: unknown) => void;
  set_server_metrics: (metrics: ReturnType<typeof normalizeServerSnapshot>) => void;
  set_speed: (speed: TimeSpeed) => void;
};

async function refreshAfterMutation(
  attachMode: AttachMode,
  speed: TimeSpeed,
  dispatch: AuthoringDispatch,
): Promise<void> {
  if (attachMode === "server") {
    const ws = getActiveServerSocket();
    if (!ws || ws.readyState !== WebSocket.OPEN) return;
    const snap = await jsonRpcCall<unknown>(ws, "sim.snapshot");
    const metrics = normalizeServerSnapshot(snap);
    dispatch.set_server_metrics(metrics);
    dispatch.set_snapshot(mergeServerSnapshot(snap, (metrics.speed_multiplier as TimeSpeed) ?? speed));
    dispatch.set_speed((metrics.speed_multiplier as TimeSpeed) ?? speed);
    return;
  }
  const snapRes = await fetch("/snapshot");
  if (snapRes.ok) {
    dispatch.set_snapshot(await snapRes.json());
  }
}

/** Run spawn / place / damage for the active attach backend (FR-CIV-WEB-008). */
export async function executeTerrainAuthoring(
  input: TerrainAuthoringInput,
  dispatch: AuthoringDispatch,
): Promise<string> {
  const normX = input.cellX / input.terrainSize;
  const normY = input.cellY / input.terrainSize;
  const worldX = input.cellX * FIXED_SCALE;
  const worldZ = input.cellY * FIXED_SCALE;
  const worldY = Math.max(0, Math.round(input.heightY)) * FIXED_SCALE;

  switch (input.tool) {
    case "SpawnCivilian": {
      if (input.attachMode === "server") {
        const ws = getActiveServerSocket();
        if (!ws || ws.readyState !== WebSocket.OPEN) {
          throw new Error("Not connected to civ-server");
        }
        const method =
          input.spawnKind === "civilian" ? "sim.spawn_civilian" : "sim.spawn_entity";
        const params =
          input.spawnKind === "civilian"
            ? { x: normX, y: normY, faction: input.faction }
            : { kind: input.spawnKind, x: normX, y: normY, faction: input.faction };
        const result = await jsonRpcCall<{ entity_id?: number }>(ws, method, params);
        await refreshAfterMutation(input.attachMode, input.speed, dispatch);
        return `Spawned ${input.spawnKind} #${result.entity_id ?? "?"}`;
      }
      if (input.spawnKind !== "civilian") {
        throw new Error(`Spawn kind "${input.spawnKind}" is server-only (use ?attach=server)`);
      }
      await postControl("/control/spawn_civilian", {
        x: normX,
        y: normY,
        faction: input.faction,
      });
      await refreshAfterMutation(input.attachMode, input.speed, dispatch);
      return `Spawned civilian at ${input.cellX}, ${input.cellY}`;
    }
    case "PlaceVoxel": {
      if (input.attachMode === "server") {
        const ws = getActiveServerSocket();
        if (!ws || ws.readyState !== WebSocket.OPEN) {
          throw new Error("Not connected to civ-server");
        }
        await jsonRpcCall(ws, "sim.place_voxel", {
          x: worldX,
          y: worldY,
          z: worldZ,
          material: input.material,
        });
        await refreshAfterMutation(input.attachMode, input.speed, dispatch);
        return `Voxel placed at ${worldX}, ${worldY}, ${worldZ}`;
      }
      await postControl("/control/place_voxel", {
        x: worldX,
        y: worldY,
        z: worldZ,
        material: input.material,
      });
      await refreshAfterMutation(input.attachMode, input.speed, dispatch);
      return `Voxel placed (watch)`;
    }
    case "DamageBomb": {
      const body = {
        x: worldX,
        y: worldY,
        z: worldZ,
        radius: input.damageRadius,
        energy: 1000,
      };
      if (input.attachMode === "server") {
        const ws = getActiveServerSocket();
        if (!ws || ws.readyState !== WebSocket.OPEN) {
          throw new Error("Not connected to civ-server");
        }
        await jsonRpcCall(ws, "sim.damage", body);
      } else {
        await postControl("/control/damage", body);
      }
      await refreshAfterMutation(input.attachMode, input.speed, dispatch);
      return `Damage queued at ${input.cellX}, ${input.cellY}`;
    }
    default:
      return `Cell ${input.cellX}, ${input.cellY}`;
  }
}
