import React, { useEffect } from "react";
import ReactDOM from "react-dom/client";
import { createRootRoute, createRouter, RouterProvider } from "@tanstack/react-router";
import { BottomBar } from "./bottom_bar";
import { EconomyPanel } from "./economy_panel";
import { useCivisAttach } from "./hooks/useCivisAttach";
import { useFramePerfMock } from "./hooks/useFramePerf";
import { SceneView } from "./scene_view";
import { SidePanel } from "./side_panel";
import { StoreProvider, useDashboardStore } from "./store";
import { applyDocumentTheme } from "./lib/theme";
import { TopBar } from "./top_bar";
import "./styles.css";

const rootRoute = createRootRoute({
  component: App,
});

const router = createRouter({ routeTree: rootRoute });

function App() {
  const { state, dispatch } = useDashboardStore();
  useCivisAttach(dispatch);
  useFramePerfMock(state.connection, dispatch);

  useEffect(() => {
    if (!state.toast) return;
    const handle = window.setTimeout(() => dispatch({ type: "clear_toast" }), 3000);
    return () => window.clearTimeout(handle);
  }, [dispatch, state.toast]);

  useEffect(() => {
    applyDocumentTheme(state.theme);
  }, [state.theme]);

  return (
    <main className="app-shell">
      <TopBar />
      <div className="scene-shell">
        <SceneView />
      </div>
      <EconomyPanel />
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
