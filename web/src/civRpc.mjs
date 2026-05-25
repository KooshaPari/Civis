/**
 * Minimal JSON-RPC 2.0 client for `civ-server` WebSocket attach.
 */

let nextId = 1;

/**
 * @param {string} method
 * @param {Record<string, unknown>} [params]
 */
export function buildJsonRpcRequest(method, params = {}) {
  const id = nextId++;
  return { jsonrpc: "2.0", id, method, params };
}

/**
 * @param {import('ws') | { send: (data: string) => void }} socket
 * @param {string} method
 * @param {Record<string, unknown>} [params]
 * @param {{ role?: string; timeoutMs?: number }} [opts]
 * @returns {Promise<unknown>}
 */
export function jsonRpcCall(socket, method, params = {}, opts = {}) {
  const { role, timeoutMs = 8000 } = opts;
  const req = buildJsonRpcRequest(method, params);
  const payload = JSON.stringify(req);

  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      cleanup();
      reject(new Error(`jsonrpc timeout: ${method}`));
    }, timeoutMs);

    const onMessage = (event) => {
      let data;
      try {
        data = typeof event.data === "string" ? JSON.parse(event.data) : JSON.parse(String(event.data));
      } catch {
        return;
      }
      if (data.id !== req.id) return;
      cleanup();
      if (data.error) {
        reject(new Error(data.error.message ?? "jsonrpc error"));
        return;
      }
      resolve(data.result);
    };

    const cleanup = () => {
      clearTimeout(timer);
      socket.removeEventListener?.("message", onMessage);
    };

    socket.addEventListener("message", onMessage);
    if (role) {
      // Browser WebSocket has no custom headers; civ-server accepts role in params for some paths.
      socket.send(payload);
    } else {
      socket.send(payload);
    }
  });
}

/**
 * Parse a JSON-RPC response object (for tests).
 * @param {string} text
 * @param {number} expectedId
 */
export function parseJsonRpcResponse(text, expectedId) {
  const data = JSON.parse(text);
  if (data.id !== expectedId) {
    throw new Error("id mismatch");
  }
  if (data.error) {
    throw new Error(data.error.message ?? "error");
  }
  return data.result;
}

/**
 * @param {Record<string, unknown>} result — `sim.snapshot` result
 */
export function normalizeServerSnapshot(result) {
  if (!result || typeof result !== "object") {
    return { tick: 0, population: 0, building_count: 0 };
  }
  const r = /** @type {Record<string, unknown>} */ (result);
  return {
    tick: Number(r.tick ?? 0),
    population: Number(r.population ?? 0),
    building_count: Number(r.building_count ?? 0),
    energy_budget: r.energy_budget != null ? Number(r.energy_budget) : undefined,
    market_prices:
      r.market_prices && typeof r.market_prices === "object"
        ? /** @type {Record<string, number>} */ (r.market_prices)
        : {},
    hash_chain_root:
      typeof r.hash_chain_root === "string" ? r.hash_chain_root : undefined,
    speed_multiplier: Number(r.speed_multiplier ?? 1),
  };
}
