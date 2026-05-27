/** Browser JSON-RPC client for `civ-server` (FR-CIV-WEB-002..005). */

export type ModBrowserEntry = {
  id: string;
  name: string;
  version: string;
  mod_type: string;
  has_wasm: boolean;
  guest_memory_len: number;
  float_instruction_count?: number;
  float_contamination_site_count?: number;
};

export type ServerMetrics = {
  tick: number;
  population: number;
  building_count: number;
  energy_budget?: number;
  market_prices: Record<string, number>;
  hash_chain_root?: string;
  speed_multiplier: number;
  mods?: ModBrowserEntry[];
};

let rpcId = 1;

function nextId() {
  return rpcId++;
}

export async function jsonRpcCall<T>(
  ws: WebSocket,
  method: string,
  params: Record<string, unknown> = {},
  timeoutMs = 8000,
): Promise<T> {
  const id = nextId();
  const payload = JSON.stringify({ jsonrpc: "2.0", id, method, params });

  return new Promise((resolve, reject) => {
    const timer = window.setTimeout(() => {
      ws.removeEventListener("message", onMessage);
      reject(new Error(`jsonrpc timeout: ${method}`));
    }, timeoutMs);

    const onMessage = (event: MessageEvent) => {
      if (typeof event.data !== "string") return;
      let data: { id?: number; result?: T; error?: { message?: string } };
      try {
        data = JSON.parse(event.data);
      } catch {
        return;
      }
      if (data.id !== id) return;
      window.clearTimeout(timer);
      ws.removeEventListener("message", onMessage);
      if (data.error) {
        reject(new Error(data.error.message ?? "jsonrpc error"));
        return;
      }
      resolve(data.result as T);
    };

    ws.addEventListener("message", onMessage);
    ws.send(payload);
  });
}

export function normalizeServerSnapshot(result: unknown): ServerMetrics {
  const r = (result ?? {}) as Record<string, unknown>;
  const prices =
    r.market_prices && typeof r.market_prices === "object"
      ? (r.market_prices as Record<string, number>)
      : {};
  return {
    tick: Number(r.tick ?? 0),
    population: Number(r.population ?? 0),
    building_count: Number(r.building_count ?? 0),
    energy_budget: r.energy_budget != null ? Number(r.energy_budget) : undefined,
    market_prices: prices,
    hash_chain_root:
      typeof r.hash_chain_root === "string" ? r.hash_chain_root : undefined,
    speed_multiplier: Number(r.speed_multiplier ?? 1),
    mods: Array.isArray(r.mods) ? (r.mods as ModBrowserEntry[]) : undefined,
  };
}

export async function fetchHealthTick(): Promise<number> {
  const response = await fetch("/healthz");
  if (!response.ok) throw new Error(`healthz ${response.status}`);
  const data = (await response.json()) as { tick?: number };
  return Number(data.tick ?? 0);
}

export async function exportReplayBlob(): Promise<Blob> {
  const response = await fetch("/replay/export");
  if (!response.ok) throw new Error(`replay export ${response.status}`);
  return response.blob();
}

export async function importReplayBytes(body: ArrayBuffer): Promise<{ tick: number }> {
  const response = await fetch("/replay/import", {
    method: "POST",
    headers: { "Content-Type": "application/octet-stream" },
    body,
  });
  if (!response.ok) throw new Error(`replay import ${response.status}`);
  const data = (await response.json()) as { tick?: number };
  return { tick: Number(data.tick ?? 0) };
}
