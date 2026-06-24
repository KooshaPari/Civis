import {
  Status,
  buildHealthProbe,
  createConnectionMonitor,
  httpBaseFromWsUrl,
  statusClass,
  statusLabel,
} from "../src/connectionStatus.mjs";
import {
  flipTheme,
  readStoredTheme,
  themeToggleLabel,
} from "../src/theme.mjs";
import { resolveWsUrlFromQuery } from "../src/wsUrl.mjs";

const themeToggle = document.getElementById("theme-toggle");
const themeToggleLabelEl = document.getElementById("theme-toggle-label");

let currentTheme = readStoredTheme({ search: window.location.search });
applyThemeUi(currentTheme);

themeToggle?.addEventListener("click", () => {
  currentTheme = flipTheme(currentTheme);
  applyThemeUi(currentTheme);
});

function applyThemeUi(theme) {
  if (themeToggle) {
    themeToggle.title = themeToggleLabel(theme);
    themeToggle.setAttribute("aria-pressed", theme === "light" ? "true" : "false");
  }
  if (themeToggleLabelEl) {
    themeToggleLabelEl.textContent = theme === "dark" ? "Light" : "Dark";
  }
}

const wsUrl = resolveWsUrlFromQuery(window.location.search);

const statusPill = document.getElementById("status-pill");
const wsUrlEl = document.getElementById("ws-url");
const healthLink = document.getElementById("health-link");
const statusDetail = document.getElementById("status-detail");
const connectBtn = document.getElementById("connect-btn");
const disconnectBtn = document.getElementById("disconnect-btn");
const probeBtn = document.getElementById("probe-btn");

wsUrlEl.textContent = wsUrl;
const httpBase = httpBaseFromWsUrl(wsUrl);
healthLink.href = `${httpBase}/healthz`;
healthLink.textContent = `${httpBase}/healthz`;

function renderStatus(status, detail = {}) {
  statusPill.textContent = statusLabel(status);
  statusPill.className = `status-pill ${statusClass(status)}`;

  if (detail.reason) {
    statusDetail.textContent = detail.reason;
  } else if (detail.code !== undefined) {
    statusDetail.textContent = `code ${detail.code}${detail.reason ? `: ${detail.reason}` : ""}`;
  } else if (status === Status.OPEN) {
    statusDetail.textContent = "WebSocket open";
  } else if (status === Status.CONNECTING) {
    statusDetail.textContent = "Opening connection…";
  } else if (status === Status.IDLE) {
    statusDetail.textContent = "Not connected";
  } else {
    statusDetail.textContent = statusLabel(status);
  }

  connectBtn.disabled = status === Status.CONNECTING || status === Status.OPEN;
  disconnectBtn.disabled = status === Status.IDLE || status === Status.CLOSED;
  probeBtn.disabled = status !== Status.OPEN;
}

const monitor = createConnectionMonitor(wsUrl, {
  onChange: (status, detail) => renderStatus(status, detail),
});

connectBtn.addEventListener("click", () => monitor.connect());
disconnectBtn.addEventListener("click", () => monitor.disconnect());
probeBtn.addEventListener("click", () => {
  if (monitor.send(buildHealthProbe(Date.now()))) {
    statusDetail.textContent = "Sent JSON-RPC health probe";
  }
});

renderStatus(Status.IDLE);
