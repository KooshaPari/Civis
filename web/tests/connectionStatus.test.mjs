import assert from "node:assert/strict";
import { test } from "node:test";
import {
  Status,
  attachConnectionDetail,
  buildHealthProbe,
  createConnectionMonitor,
  httpBaseFromWsUrl,
  readyStateToStatus,
  statusClass,
  statusLabel,
} from "../src/connectionStatus.mjs";

test("readyStateToStatus maps WebSocket ready states", () => {
  assert.equal(readyStateToStatus(0), Status.CONNECTING);
  assert.equal(readyStateToStatus(1), Status.OPEN);
  assert.equal(readyStateToStatus(2), Status.CLOSED);
  assert.equal(readyStateToStatus(3), Status.CLOSED);
  assert.equal(readyStateToStatus(undefined), Status.ERROR);
});

test("statusLabel and statusClass expose UI-friendly values", () => {
  assert.equal(statusLabel(Status.OPEN), "Connected");
  assert.equal(statusClass(Status.OPEN), "status-open");
  assert.equal(statusClass("mystery"), "status-unknown");
});

test("attachConnectionDetail keeps idle and closed states visually aligned", () => {
  assert.equal(attachConnectionDetail(Status.IDLE, "server"), "Not connected");
  assert.equal(attachConnectionDetail(Status.CLOSED, "watch"), "Not connected");
});

test("buildHealthProbe emits JSON-RPC health request", () => {
  assert.deepEqual(JSON.parse(buildHealthProbe(7)), {
    jsonrpc: "2.0",
    id: 7,
    method: "health",
    params: {},
  });
});

test("httpBaseFromWsUrl derives HTTP origin from ws URL", () => {
  assert.equal(
    httpBaseFromWsUrl("ws://127.0.0.1:3000/ws"),
    "http://127.0.0.1:3000",
  );
  assert.equal(
    httpBaseFromWsUrl("wss://civis.example/ws"),
    "https://civis.example",
  );
});

test("createConnectionMonitor reports open and closed lifecycle", () => {
  /** @type {Array<{ type: string; listeners: Map<string, Set<Function>> }>} */
  const sockets = [];

  class MockWebSocket {
  /** @param {string} url */
    constructor(url) {
      this.url = url;
      this.readyState = 0;
      this.listeners = new Map();
      sockets.push(this);
    }

    /** @param {string} type @param {(event?: unknown) => void} handler */
    addEventListener(type, handler) {
      if (!this.listeners.has(type)) {
        this.listeners.set(type, new Set());
      }
      this.listeners.get(type).add(handler);
    }

    /** @param {string} type */
    emit(type, event = {}) {
      for (const handler of this.listeners.get(type) ?? []) {
        handler(event);
      }
    }

    close() {
      this.readyState = 3;
      this.emit("close", { code: 1000, reason: "bye" });
    }
  }

  const changes = [];
  const monitor = createConnectionMonitor("ws://127.0.0.1:3000/ws", {
    WebSocketImpl: MockWebSocket,
    onChange: (status, detail) => changes.push({ status, detail }),
  });

  monitor.connect();
  assert.equal(monitor.getStatus(), Status.CONNECTING);
  assert.deepEqual(changes.map((entry) => entry.status), [Status.CONNECTING]);
  assert.equal(sockets.length, 1);

  sockets[0].emit("open");
  assert.equal(monitor.getStatus(), Status.OPEN);

  monitor.disconnect();
  assert.equal(monitor.getStatus(), Status.IDLE);
  assert.ok(changes.some((entry) => entry.status === Status.OPEN));
});

test("createConnectionMonitor ignores stale close events after disconnect", () => {
  /** @type {MockWebSocket | null} */
  let socket = null;

  class MockWebSocket {
    constructor() {
      this.readyState = 0;
      this.listeners = new Map();
      socket = this;
    }

    /** @param {string} type @param {(event?: unknown) => void} handler */
    addEventListener(type, handler) {
      if (!this.listeners.has(type)) {
        this.listeners.set(type, new Set());
      }
      this.listeners.get(type).add(handler);
    }

    /** @param {string} type */
    emit(type, event = {}) {
      for (const handler of this.listeners.get(type) ?? []) {
        handler(event);
      }
    }

    close() {
      this.readyState = 3;
    }
  }

  const monitor = createConnectionMonitor("ws://127.0.0.1:3000/ws", {
    WebSocketImpl: MockWebSocket,
  });

  monitor.connect();
  socket?.emit("open");
  assert.equal(monitor.getStatus(), Status.OPEN);

  monitor.disconnect();
  assert.equal(monitor.getStatus(), Status.IDLE);

  socket?.emit("close", { code: 1000, reason: "late" });
  assert.equal(monitor.getStatus(), Status.IDLE);
});

test("createConnectionMonitor ignores stale events from a replaced socket", () => {
  /** @type {Array<MockWebSocket>} */
  const sockets = [];

  class MockWebSocket {
    constructor() {
      this.readyState = 0;
      this.listeners = new Map();
      sockets.push(this);
    }

    /** @param {string} type @param {(event?: unknown) => void} handler */
    addEventListener(type, handler) {
      if (!this.listeners.has(type)) {
        this.listeners.set(type, new Set());
      }
      this.listeners.get(type).add(handler);
    }

    /** @param {string} type */
    emit(type, event = {}) {
      for (const handler of this.listeners.get(type) ?? []) {
        handler(event);
      }
    }

    close() {
      this.readyState = 3;
    }
  }

  const changes = [];
  const monitor = createConnectionMonitor("ws://127.0.0.1:3000/ws", {
    WebSocketImpl: MockWebSocket,
    onChange: (status, detail) => changes.push({ status, detail }),
  });

  monitor.connect();
  sockets[0].emit("open");
  assert.equal(monitor.getStatus(), Status.OPEN);

  monitor.connect();
  sockets[0].emit("close", { code: 1006, reason: "stale" });
  assert.equal(monitor.getStatus(), Status.CONNECTING);

  sockets[1].emit("open");
  assert.equal(monitor.getStatus(), Status.OPEN);
  assert.ok(changes.some((entry) => entry.status === Status.CONNECTING));
});

test("createConnectionMonitor send delivers messages when open", () => {
  /** @type {MockWebSocket | null} */
  let socket = null;

  class MockWebSocket {
    constructor() {
      this.readyState = 0;
      this.sent = [];
      this.listeners = new Map();
      socket = this;
    }

    /** @param {string} type @param {(event?: unknown) => void} handler */
    addEventListener(type, handler) {
      if (!this.listeners.has(type)) {
        this.listeners.set(type, new Set());
      }
      this.listeners.get(type).add(handler);
    }

    /** @param {string} type */
    emit(type, event = {}) {
      if (type === "open") {
        this.readyState = 1;
      }
      for (const handler of this.listeners.get(type) ?? []) {
        handler(event);
      }
    }

    send(message) {
      this.sent.push(message);
    }

    close() {}
  }

  const monitor = createConnectionMonitor("ws://127.0.0.1:3000/ws", {
    WebSocketImpl: MockWebSocket,
  });

  monitor.connect();
  socket?.emit("open");
  assert.equal(monitor.send("ping"), true);
  assert.deepEqual(socket?.sent, ["ping"]);
});

test("createConnectionMonitor handles constructor failures", () => {
  class BrokenWebSocket {
    constructor() {
      throw new Error("blocked");
    }
  }

  const monitor = createConnectionMonitor("ws://127.0.0.1:3000/ws", {
    WebSocketImpl: BrokenWebSocket,
  });

  monitor.connect();
  assert.equal(monitor.getStatus(), Status.ERROR);
  assert.equal(monitor.getDetail().reason, "blocked");
});
