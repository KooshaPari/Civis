import React, { useEffect, useRef } from "react";
import ReactDOM from "react-dom/client";
import { createRootRoute, createRouter, RouterProvider } from "@tanstack/react-router";
import { BottomBar } from "./bottom_bar";
import { Scene3d } from "./scene3d";
import { SidePanel } from "./side_panel";
import { StoreProvider, useDashboardStore } from "./store";
import { TopBar } from "./top_bar";
import "./styles.css";

const rootRoute = createRootRoute({
  component: App,
});

const router = createRouter({ routeTree: rootRoute });

function App() {
  const { state, dispatch } = useDashboardStore();
  const reconnectTimerRef = useRef<number | null>(null);
  const sourceRef = useRef<EventSource | null>(null);
  const closedByCleanupRef = useRef(false);

  useEffect(() => {
    const loadSnapshot = async () => {
      try {
        const response = await fetch("/snapshot");
        if (!response.ok) return;
        const data = await response.json();
        dispatch({ type: "set_snapshot", snapshot: data });
      } catch {
        // SSE remains primary.
      }
    };
    void loadSnapshot();
  }, [dispatch]);

  useEffect(() => {
    const scheduleReconnect = () => {
      if (closedByCleanupRef.current || reconnectTimerRef.current !== null) return;
      dispatch({ type: "set_connection", connection: "reconnecting" });
      reconnectTimerRef.current = window.setTimeout(() => {
        reconnectTimerRef.current = null;
        connect();
      }, 3000);
    };

    const connect = () => {
      if (closedByCleanupRef.current) return;
      sourceRef.current?.close();
      const source = new EventSource("/events");
      sourceRef.current = source;
      source.onopen = () => dispatch({ type: "set_connection", connection: "live" });
      source.onmessage = () => dispatch({ type: "set_connection", connection: "live" });
      source.addEventListener("snapshot", (event) => {
        const payload = (event as MessageEvent<string>).data;
        dispatch({ type: "set_snapshot", snapshot: JSON.parse(payload) });
        dispatch({ type: "set_connection", connection: "live" });
      });
      source.onerror = () => {
        if (source.readyState === EventSource.CLOSED) {
          dispatch({ type: "set_connection", connection: "disconnected" });
          scheduleReconnect();
          return;
        }
        dispatch({ type: "set_connection", connection: "reconnecting" });
        scheduleReconnect();
      };
    };

    connect();
    return () => {
      closedByCleanupRef.current = true;
      sourceRef.current?.close();
      if (reconnectTimerRef.current !== null) window.clearTimeout(reconnectTimerRef.current);
    };
  }, [dispatch]);

  useEffect(() => {
    const handle = window.setInterval(async () => {
      if (state.snapshot) return;
      try {
        const response = await fetch("/snapshot");
        if (!response.ok) return;
        dispatch({ type: "set_snapshot", snapshot: await response.json() });
      } catch {
        // keep polling best-effort
      }
    }, 3000);
    return () => window.clearInterval(handle);
  }, [dispatch, state.snapshot]);

  useEffect(() => {
    if (!state.toast) return;
    const handle = window.setTimeout(() => dispatch({ type: "clear_toast" }), 3000);
    return () => window.clearTimeout(handle);
  }, [dispatch, state.toast]);

  return (
    <main className="app-shell">
      <TopBar />
      <div className="scene-shell">
        <Scene3d />
      </div>
      <SidePanel />
      <BottomBar />
      {state.toast ? <div className="toast">{state.toast.message}</div> : null}
    </main>
  );
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <StoreProvider>
      <RouterProvider router={router} />
    </StoreProvider>
  </React.StrictMode>,
);
