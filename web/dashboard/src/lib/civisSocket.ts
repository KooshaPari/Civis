/** Module-level handle to the active `civ-server` WebSocket (operator controls). */

let activeWs: WebSocket | null = null;

export function setActiveServerSocket(ws: WebSocket | null) {
  activeWs = ws;
}

export function getActiveServerSocket(): WebSocket | null {
  return activeWs;
}
