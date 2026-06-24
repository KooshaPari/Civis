import { useEffect, useRef } from "react";
import {
  resolveAttachMode,
  resolveAuthoringEnabled,
  resolveBrowserWsUrl,
  resolveWsPreferBinary,
} from "../lib/attachConfig";
import { frame3dTick, frame3dVoxelChunkIds, parseWsPayload } from "../lib/frame3d";
import { frame3dAgentIds, noteAgentIds } from "../lib/agents";
import { noteChunkIds } from "../lib/minimap";
import {
  jsonRpcCall,
  normalizeServerSnapshot,
  type ServerMetrics,
} from "../lib/civisServer";
import { mergeServerSnapshot } from "../lib/mergeSnapshot";
import { setActiveServerSocket } from "../lib/civisSocket";
import type { FrameSampleSource, Snapshot, Terrain, TimeSpeed } from "../store";

type Dispatch = React.Dispatch<
  | { type: "set_connection"; connection: "live" | "reconnecting" | "disconnected" }
  | { type: "set_snapshot"; snapshot: Snapshot | null }
  | { type: "set_terrain"; terrain: Terrain | null }
  | { type: "set_server_metrics"; metrics: ServerMetrics | null }
  | { type: "set_attach_mode"; mode: "watch" | "server" }
  | { type: "set_read_only"; readOnly: boolean }
  | { type: "set_frame3d_tick"; tick: number | null }
  | { type: "set_chunk_stats"; count: number; recentIds: number[]; loadedIds: number[] }
  | { type: "set_agent_stats"; count: number; recentIds: number[] }
  | { type: "set_speed"; speed: 0 | 1 | 2 | 4 | 8 }
  | { type: "push_frame_sample"; ms: number; source?: FrameSampleSource }
  | { type: "reset_frame_samples" }
>;

function recordVoxelChunks(
  loadedChunkIdsRef: React.MutableRefObject<Set<number>>,
  recentChunkIdsRef: React.MutableRefObject<number[]>,
  dispatch: Dispatch,
  frame: unknown,
) {
  const chunkIds = frame3dVoxelChunkIds(frame);
  if (chunkIds.length === 0) return false;

  const stats = noteChunkIds(loadedChunkIdsRef.current, recentChunkIdsRef.current, chunkIds);
  recentChunkIdsRef.current = stats.recentIds;
  dispatch({
    type: "set_chunk_stats",
    count: stats.count,
    recentIds: stats.recentIds,
    loadedIds: [...loadedChunkIdsRef.current],
  });
  return true;
}

function recordAgentAppearance(
  seenAgentIdsRef: React.MutableRefObject<Set<number>>,
  recentAgentIdsRef: React.MutableRefObject<number[]>,
  dispatch: Dispatch,
  frame: unknown,
) {
  const ids = frame3dAgentIds(frame);
  if (ids.length === 0) return false;

  const stats = noteAgentIds(seenAgentIdsRef.current, recentAgentIdsRef.current, ids);
  recentAgentIdsRef.current = stats.recentIds;
  dispatch({ type: "set_agent_stats", count: stats.count, recentIds: stats.recentIds });
  return true;
}

function recordAttachFrame(
  lastAtRef: React.MutableRefObject<number | null>,
  dispatch: Dispatch,
) {
  const now = performance.now();
  if (lastAtRef.current != null) {
    dispatch({
      type: "push_frame_sample",
      ms: Math.max(0, now - lastAtRef.current),
      source: "attach",
    });
  }
  lastAtRef.current = now;
}

async function loadWatchTerrain(): Promise<Terrain | null> {
  try {
    const response = await fetch("/terrain");
    if (!response.ok) return null;
    return (await response.json()) as Terrain;
  } catch {
    return null;
  }
}

export function useCivisAttach(dispatch: Dispatch) {
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectRef = useRef<number | null>(null);
  const attachFrameAtRef = useRef<number | null>(null);
  const loadedChunkIdsRef = useRef<Set<number>>(new Set());
  const recentChunkIdsRef = useRef<number[]>([]);
  const seenAgentIdsRef = useRef<Set<number>>(new Set());
  const recentAgentIdsRef = useRef<number[]>([]);

  useEffect(() => {
    attachFrameAtRef.current = null;
    loadedChunkIdsRef.current = new Set();
    recentChunkIdsRef.current = [];
    seenAgentIdsRef.current = new Set();
    recentAgentIdsRef.current = [];
    dispatch({ type: "reset_frame_samples" });

    const search = window.location.search;
    const mode = resolveAttachMode(search);
    dispatch({ type: "set_attach_mode", mode });
    dispatch({ type: "set_read_only", readOnly: !resolveAuthoringEnabled(search) });

    if (mode === "watch") {
      return connectWatch(dispatch, attachFrameAtRef);
    }
    const preferBinary = resolveWsPreferBinary(search);
    return connectServer(
      dispatch,
      resolveBrowserWsUrl(search),
      preferBinary,
      wsRef,
      reconnectRef,
      attachFrameAtRef,
      loadedChunkIdsRef,
      recentChunkIdsRef,
      seenAgentIdsRef,
      recentAgentIdsRef,
    );
  }, [dispatch]);
}

function connectWatch(
  dispatch: Dispatch,
  attachFrameAtRef: React.MutableRefObject<number | null>,
) {
  let closed = false;
  let source: EventSource | null = null;
  let reconnectTimer: number | null = null;

  const clearReconnectTimer = () => {
    if (reconnectTimer !== null) {
      window.clearTimeout(reconnectTimer);
      reconnectTimer = null;
    }
  };

  const scheduleReconnect = () => {
    if (closed || reconnectTimer !== null) return;
    dispatch({ type: "set_connection", connection: "reconnecting" });
    reconnectTimer = window.setTimeout(() => {
      reconnectTimer = null;
      connect();
    }, 3000);
  };

  const connect = () => {
    if (closed) return;
    source?.close();
    clearReconnectTimer();
    source = new EventSource("/events");
    source.onopen = () => {
      clearReconnectTimer();
      dispatch({ type: "set_connection", connection: "live" });
    };
    source.addEventListener("snapshot", (event) => {
      const payload = (event as MessageEvent<string>).data;
      dispatch({ type: "set_snapshot", snapshot: JSON.parse(payload) });
      clearReconnectTimer();
      dispatch({ type: "set_connection", connection: "live" });
      recordAttachFrame(attachFrameAtRef, dispatch);
    });
    source.onerror = () => {
      dispatch({ type: "set_connection", connection: "disconnected" });
      if (source?.readyState === EventSource.CLOSED) {
        scheduleReconnect();
      }
    };
  };

  void (async () => {
    try {
      const snapRes = await fetch("/snapshot");
      if (snapRes.ok) {
        dispatch({ type: "set_snapshot", snapshot: await snapRes.json() });
      }
    } catch {
      /* SSE primary */
    }
    const terrain = await loadWatchTerrain();
    if (terrain) dispatch({ type: "set_terrain", terrain });
  })();

  connect();

  return () => {
    closed = true;
    source?.close();
    clearReconnectTimer();
  };
}

function connectServer(
  dispatch: Dispatch,
  wsUrl: string,
  preferBinary: boolean,
  wsRef: React.MutableRefObject<WebSocket | null>,
  reconnectRef: React.MutableRefObject<number | null>,
  attachFrameAtRef: React.MutableRefObject<number | null>,
  loadedChunkIdsRef: React.MutableRefObject<Set<number>>,
  recentChunkIdsRef: React.MutableRefObject<number[]>,
  seenAgentIdsRef: React.MutableRefObject<Set<number>>,
  recentAgentIdsRef: React.MutableRefObject<number[]>,
) {
  let closed = false;
  let lastSnapshotRefreshAt = 0;
  const SNAPSHOT_REFRESH_MS = 250;

  const refreshSnapshot = async (ws: WebSocket) => {
    await jsonRpcCall(ws, "health");
    const result = await jsonRpcCall<unknown>(ws, "sim.snapshot");
    const metrics = normalizeServerSnapshot(result);
    const speed = (metrics.speed_multiplier as TimeSpeed) ?? 1;
    dispatch({ type: "set_server_metrics", metrics });
    dispatch({ type: "set_snapshot", snapshot: mergeServerSnapshot(result, speed) });
    dispatch({ type: "set_speed", speed });
    if (speed === 0) {
      const started = await jsonRpcCall<{ multiplier?: number }>(ws, "sim.set_speed", {
        multiplier: 1,
      });
      const runSpeed = (started?.multiplier ?? 1) as TimeSpeed;
      dispatch({ type: "set_speed", speed: runSpeed });
    }
  };

  const connect = () => {
    if (closed) return;
    dispatch({ type: "set_connection", connection: "reconnecting" });
    wsRef.current?.close();
    const ws = new WebSocket(wsUrl);
    wsRef.current = ws;

    ws.onopen = () => {
      setActiveServerSocket(ws);
      void (async () => {
        try {
          await refreshSnapshot(ws);
          dispatch({ type: "set_connection", connection: "live" });
          void loadWatchTerrain().then((terrain) => {
            if (terrain) dispatch({ type: "set_terrain", terrain });
          });
        } catch {
          dispatch({ type: "set_connection", connection: "disconnected" });
          scheduleReconnect();
        }
      })();
    };

    ws.onmessage = (event) => {
      if (preferBinary && typeof event.data === "string") return;

      void (async () => {
        let handled = false;
        try {
          let payload: string | Uint8Array;
          if (typeof event.data === "string") {
            payload = event.data;
          } else if (event.data instanceof ArrayBuffer) {
            payload = new Uint8Array(event.data);
          } else if (event.data instanceof Blob) {
            payload = new Uint8Array(await event.data.arrayBuffer());
          } else {
            return;
          }
          const frame = parseWsPayload(payload);
          const tick = frame3dTick(frame);
          const hasChunks = recordVoxelChunks(
            loadedChunkIdsRef,
            recentChunkIdsRef,
            dispatch,
            frame,
          );
          const hasAgents = recordAgentAppearance(
            seenAgentIdsRef,
            recentAgentIdsRef,
            dispatch,
            frame,
          );
          if (tick != null) {
            dispatch({ type: "set_frame3d_tick", tick });
            handled = true;
            const now = performance.now();
            if (now - lastSnapshotRefreshAt >= SNAPSHOT_REFRESH_MS) {
              lastSnapshotRefreshAt = now;
              void refreshSnapshot(ws).catch(() => {
                /* keep last snapshot on transient RPC errors */
              });
            }
          } else if (hasChunks || hasAgents) {
            handled = true;
          }
        } catch {
          /* ignore non-frame payloads */
        }
        if (handled) recordAttachFrame(attachFrameAtRef, dispatch);
      })();
    };

    ws.onerror = () => dispatch({ type: "set_connection", connection: "disconnected" });
    ws.onclose = () => {
      setActiveServerSocket(null);
      if (!closed) scheduleReconnect();
    };
  };

  const scheduleReconnect = () => {
    if (closed || reconnectRef.current !== null) return;
    dispatch({ type: "set_connection", connection: "reconnecting" });
    reconnectRef.current = window.setTimeout(() => {
      reconnectRef.current = null;
      connect();
    }, 3000);
  };

  connect();

  return () => {
    closed = true;
    setActiveServerSocket(null);
    wsRef.current?.close();
    if (reconnectRef.current !== null) {
      window.clearTimeout(reconnectRef.current);
    }
  };
}
