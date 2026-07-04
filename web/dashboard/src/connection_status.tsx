import { useEffect, useMemo, useRef, useState } from "react";
import {
  attachEndpointLabel,
  attachEndpointUrl,
  attachModeLabel,
  resolveBrowserWsUrl,
} from "./lib/attachConfig";
import {
  buildHealthProbe,
  connectionDetail,
  dashboardConnectionToStatus,
  httpBaseFromWsUrl,
  Status,
  statusClass,
  statusLabel,
} from "./lib/connectionStatus";
import { getActiveServerSocket } from "./lib/civisSocket";
import { useDashboardStore } from "./store";

export function ConnectionStatusCard() {
  const { state, dispatch } = useDashboardStore();
  const [probeNote, setProbeNote] = useState<string | null>(null);
  const probeTimerRef = useRef<number | null>(null);

  const wsUrl = useMemo(() => resolveBrowserWsUrl(window.location.search), []);
  const endpointLabel = attachEndpointLabel(state.attachMode);
  const endpointValue = attachEndpointUrl(state.attachMode, wsUrl, window.location.origin);
  const targetLabel = attachModeLabel(state.attachMode);

  const status = dashboardConnectionToStatus(state.connection);
  const healthHref =
    state.attachMode === "server"
      ? `${httpBaseFromWsUrl(wsUrl)}/healthz`
      : `${window.location.origin}/healthz`;

  const detail = probeNote ?? connectionDetail(status, state.attachMode);

  const sendProbe = () => {
    const ws = getActiveServerSocket();
    if (ws?.readyState === WebSocket.OPEN) {
      ws.send(buildHealthProbe(Date.now()));
      setProbeNote("Sent JSON-RPC health probe");
      if (probeTimerRef.current !== null) {
        window.clearTimeout(probeTimerRef.current);
      }
      probeTimerRef.current = window.setTimeout(() => {
        probeTimerRef.current = null;
        setProbeNote(null);
      }, 2500);
      return;
    }
    dispatch({ type: "set_toast", message: "Not connected to civ-server" });
  };

  useEffect(() => {
    return () => {
      if (probeTimerRef.current !== null) {
        window.clearTimeout(probeTimerRef.current);
        probeTimerRef.current = null;
      }
    };
  }, []);

  return (
    <section className="inspector-section connection-status" aria-labelledby="connection-heading">
      <ConnectionSectionHeader id="connection-heading" title="Connection" href="./status.html" />
      <dl className="connection-meta">
        <div>
          <dt>Status</dt>
          <dd>
            <span className={`status-pill ${statusClass(status)}`}>{statusLabel(status)}</span>
          </dd>
        </div>
        <div>
          <dt>Attach target</dt>
          <dd>{targetLabel}</dd>
        </div>
        <div>
          <dt>{endpointLabel}</dt>
          <dd>{endpointValue}</dd>
        </div>
        <div>
          <dt>HTTP health</dt>
          <dd>
            <a href={healthHref} target="_blank" rel="noopener noreferrer">
              {healthHref}
            </a>
          </dd>
        </div>
        <div>
          <dt>Last detail</dt>
          <dd>{detail}</dd>
        </div>
      </dl>
      {state.attachMode === "server" ? (
        <div className="connection-actions">
          <button type="button" disabled={status !== Status.OPEN} onClick={sendProbe}>
            Send health probe
          </button>
          <a className="connection-link" href="./status.html">
            Full status page
          </a>
        </div>
      ) : (
        <p className="connection-hint">
          Watch mode attaches to civ-watch over SSE.{" "}
          <a href="./status.html">Open status page</a> to inspect the civ-server WebSocket endpoint.
        </p>
      )}
    </section>
  );
}

function ConnectionSectionHeader({
  id,
  title,
  href,
}: {
  id: string;
  title: string;
  href?: string;
}) {
  return (
    <div className="connection-head">
      <h3 id={id}>{title}</h3>
      {href ? (
        <a className="connection-link" href={href}>
          Details
        </a>
      ) : null}
    </div>
  );
}
