import React, { useEffect, useRef } from "react";
import ReactDOM from "react-dom/client";
import { createRootRoute, createRouter, RouterProvider } from "@tanstack/react-router";
import { BottomBar } from "./bottom_bar";
import { EconomyPanel } from "./economy_panel";
import { useCivisAttach } from "./hooks/useCivisAttach";
import { useFramePerfMock } from "./hooks/useFramePerf";
import { playBirth, playClick, playConflict, playDeath, playDisaster, playTech } from "./lib/sounds";
import { SceneView } from "./scene_view";
import { SidePanel } from "./side_panel";
import { Notifications } from "./notifications";
import { StoreProvider, useDashboardStore, type NotificationItem, type Snapshot } from "./store";
import { applyDocumentTheme } from "./lib/theme";
import { TopBar } from "./top_bar";
import { TechTreeModal } from "./tech_tree";
import "./styles.css";

const rootRoute = createRootRoute({
  component: App,
});

const router = createRouter({ routeTree: rootRoute });

function App() {
  const { state, dispatch } = useDashboardStore();
  const previousSnapshotRef = useRef<Snapshot | null>(null);
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

  useEffect(() => {
    const snapshot = state.snapshot;
    const previous = previousSnapshotRef.current;
    previousSnapshotRef.current = snapshot;
    if (!snapshot || !previous) return;
    const notifications = buildNotifications(snapshot, previous, state.notifications);
    notifications.forEach((notification) => {
      dispatch({ type: "push_notification", notification });
      playSoundForNotification(notification.kind, state.soundEnabled);
    });
  }, [dispatch, state.notifications, state.soundEnabled, state.snapshot]);

  return (
    <main className="app-shell">
      <TopBar />
      <Notifications />
      <div className="scene-shell">
        <SceneView />
      </div>
      <EconomyPanel />
      <SidePanel />
      <BottomBar />
      <TechTreeModal />
      {state.toast ? <div className="toast">{state.toast.message}</div> : null}
    </main>
  );
}

function playSoundForNotification(kind: NotificationItem["kind"], enabled: boolean) {
  if (!enabled) return;
  if (kind === "birth") void playBirth();
  else if (kind === "death") void playDeath();
  else if (kind === "diplomacy") void playConflict();
  else if (kind === "tech") void playTech();
  else if (kind === "disaster") void playDisaster();
  else if (kind === "trade") void playClick();
}

function buildNotifications(snapshot: Snapshot, previous: Snapshot, existing: NotificationItem[]) {
  const existingKeys = new Set(existing.map((item) => `${item.tick}:${item.kind}:${item.message}`));
  const factionsById = new Map(snapshot.factions.map((faction) => [faction.id, faction]));
  const items: NotificationItem[] = [];
  const add = (item: NotificationItem) => {
    const key = `${item.tick}:${item.kind}:${item.message}`;
    if (existingKeys.has(key)) return;
    existingKeys.add(key);
    items.push(item);
  };

  const prevBirthKeys = new Set(previous.birth_events.map((event) => `${event.tick}:${event.entity_id}:${event.x}:${event.y}`));
  const prevDeathKeys = new Set(previous.death_events.map((event) => `${event.tick}:${event.entity_id}:${event.x}:${event.y}`));
  const prevDamageKeys = new Set(previous.damage_events.map((event) => `${event.x}:${event.y}`));
  const prevDiplomacyKeys = new Set(previous.diplomacy_events.map((event) => `${event.tick}:${event.faction_a}:${event.faction_b}:${event.kind}`));
  const prevEventKeys = new Set(previous.events.map((event) => `${event.tick}:${event.kind}:${event.message}:${event.faction_id ?? "n"}`));

  snapshot.birth_events.forEach((event) =>
    {
      const key = `${event.tick}:${event.entity_id}:${event.x}:${event.y}`;
      if (prevBirthKeys.has(key)) return;
      add({
        id: Number(`${snapshot.tick}${event.entity_id}1`),
        tick: event.tick,
        kind: "birth",
        icon: "👶",
        message: `Birth at ${event.x.toFixed(2)}, ${event.y.toFixed(2)}`,
        focus: [event.x, event.y],
      });
    },
  );
  snapshot.death_events.forEach((event) => {
    const key = `${event.tick}:${event.entity_id}:${event.x}:${event.y}`;
    if (prevDeathKeys.has(key)) return;
    add({
      id: Number(`${snapshot.tick}${event.entity_id}2`),
      tick: event.tick,
      kind: "death",
      icon: "💀",
      message: `Death at ${event.x.toFixed(2)}, ${event.y.toFixed(2)}`,
      focus: [event.x, event.y],
    });
  });
  snapshot.damage_events.forEach((event, index) => {
    const key = `${event.x}:${event.y}`;
    if (prevDamageKeys.has(key)) return;
    add({
      id: Number(`${snapshot.tick}${Math.round(event.x * 1000)}${Math.round(event.y * 1000)}3${index}`),
      tick: snapshot.tick,
      kind: "disaster",
      icon: "⚡",
      message: `Disaster at ${event.x.toFixed(2)}, ${event.y.toFixed(2)}`,
      focus: [event.x, event.y],
    });
  });
  snapshot.diplomacy_events.forEach((event, index) => {
    const key = `${event.tick}:${event.faction_a}:${event.faction_b}:${event.kind}`;
    if (prevDiplomacyKeys.has(key)) return;
    const faction = factionsById.get(event.faction_a) ?? factionsById.values().next().value ?? null;
    const icon = event.kind === "Conflict" ? "⚔️" : "🤝";
    add({
      id: Number(`${snapshot.tick}${event.faction_a}${event.faction_b}4${index}`),
      tick: event.tick,
      kind: "diplomacy",
      icon,
      message: event.kind === "Conflict" ? "Conflict declared" : event.kind === "TradeAgreement" ? "Trade agreement signed" : "Peace brokered",
      focus: faction ? [faction.capital[0], faction.capital[1]] : null,
    });
  });
  snapshot.events.forEach((event, index) => {
    const key = `${event.tick}:${event.kind}:${event.message}:${event.faction_id ?? "n"}`;
    if (prevEventKeys.has(key)) return;
    if (event.kind === "tech") {
      add({
        id: Number(`${snapshot.tick}${index}5`),
        tick: event.tick,
        kind: "tech",
        icon: "🔬",
        message: event.message || "Technology unlocked",
        focus: snapshot.factions[0] ? [snapshot.factions[0].capital[0], snapshot.factions[0].capital[1]] : null,
      });
    }
    if (event.kind === "trade") {
      add({
        id: Number(`${snapshot.tick}${index}6`),
        tick: event.tick,
        kind: "trade",
        icon: "📦",
        message: event.message || "Trade update",
        focus: snapshot.factions[0] ? [snapshot.factions[0].capital[0], snapshot.factions[0].capital[1]] : null,
      });
    }
  });

  return items.slice(0, 5);
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <StoreProvider>
      <RouterProvider router={router} />
    </StoreProvider>
  </React.StrictMode>,
);
