export type InfoviewOverlayId = "agents" | "mods" | "perf";

export type InfoviewOverlay = {
  id: InfoviewOverlayId;
  label: string;
  description: string;
  defaultEnabled: boolean;
};

export const INFOVIEW_OVERLAYS = [
  {
    id: "agents",
    label: "Agents",
    description: "Show agent-related status and debug overlays.",
    defaultEnabled: true,
  },
  {
    id: "mods",
    label: "Mods",
    description: "Show mod lifecycle and attachment overlays.",
    defaultEnabled: true,
  },
  {
    id: "perf",
    label: "Performance",
    description: "Show frame timing and render budget overlays.",
    defaultEnabled: false,
  },
] as const satisfies readonly InfoviewOverlay[];

const OVERLAY_BY_ID = new Map<InfoviewOverlayId, InfoviewOverlay>(
  INFOVIEW_OVERLAYS.map((overlay) => [overlay.id, overlay]),
);

export type InfoviewOverlayState = Record<InfoviewOverlayId, boolean>;

export function listInfoviewOverlayIds(): InfoviewOverlayId[] {
  return INFOVIEW_OVERLAYS.map((overlay) => overlay.id);
}

export function getInfoviewOverlay(id: string): InfoviewOverlay | null {
  return OVERLAY_BY_ID.get(id as InfoviewOverlayId) ?? null;
}

export function createInfoviewOverlayState(
  state?: Partial<InfoviewOverlayState>,
): InfoviewOverlayState {
  const next = {} as InfoviewOverlayState;
  for (const overlay of INFOVIEW_OVERLAYS) {
    next[overlay.id] = state?.[overlay.id] ?? overlay.defaultEnabled;
  }
  return next;
}

export function getInfoviewOverlayEnabled(
  state: Partial<InfoviewOverlayState>,
  id: string,
): boolean {
  const overlay = getInfoviewOverlay(id);
  return overlay ? Boolean(state[overlay.id] ?? overlay.defaultEnabled) : false;
}

export function setInfoviewOverlayEnabled(
  state: Partial<InfoviewOverlayState>,
  id: string,
  enabled?: boolean,
): InfoviewOverlayState {
  const overlay = getInfoviewOverlay(id);
  if (!overlay) return createInfoviewOverlayState(state);
  return {
    ...createInfoviewOverlayState(state),
    [overlay.id]: enabled ?? !getInfoviewOverlayEnabled(state, id),
  };
}

export function toggleInfoviewOverlay(
  state: Partial<InfoviewOverlayState>,
  id: string,
): InfoviewOverlayState {
  return setInfoviewOverlayEnabled(state, id);
}
