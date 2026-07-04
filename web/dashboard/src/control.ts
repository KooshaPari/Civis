export async function postControl<TBody extends object>(
  path: string,
  body: TBody,
): Promise<void> {
  const response = await fetch(path, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(body),
  });

  if (!response.ok) {
    throw new Error(`POST ${path} failed with ${response.status}`);
  }
}

export type DashboardShortcut = {
  keys: string;
  action: string;
};

export const DASHBOARD_SHORTCUTS: DashboardShortcut[] = [
  { keys: "1", action: "Set speed to 1x" },
  { keys: "2", action: "Set speed to 2x" },
  { keys: "3", action: "Set speed to 4x" },
  { keys: "Space", action: "Pause / resume" },
  { keys: "g", action: "Toggle grid" },
  { keys: "m", action: "Toggle minimap" },
  { keys: "Esc", action: "Clear selection" },
];

