import { useEffect } from "react";
import { postControl } from "../control";
import { useDashboardStore, type TimeSpeed } from "../store";

const SPEED_KEYS: Record<string, TimeSpeed> = {
  "1": 1,
  "2": 2,
  "3": 4,
};

function isTextInput(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  const tag = target.tagName;
  return (
    target.isContentEditable ||
    tag === "INPUT" ||
    tag === "TEXTAREA" ||
    tag === "SELECT"
  );
}

async function setSpeed(speed: TimeSpeed) {
  await postControl("/control/speed", { speed });
}

export function useDashboardShortcuts() {
  const { state, dispatch } = useDashboardStore();

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.repeat || event.defaultPrevented) return;
      if (event.ctrlKey || event.metaKey || event.altKey) return;
      if (isTextInput(event.target)) return;

      const key = event.key.toLowerCase();

      if (event.code === "Space") {
        event.preventDefault();
        void setSpeed(state.speed === 0 ? 1 : 0)
          .then(() => dispatch({ type: "set_speed", speed: state.speed === 0 ? 1 : 0 }))
          .catch(() => dispatch({ type: "set_toast", message: "sim.set_speed failed" }));
        return;
      }

      const nextSpeed = SPEED_KEYS[key];
      if (nextSpeed != null) {
        event.preventDefault();
        void setSpeed(nextSpeed)
          .then(() => dispatch({ type: "set_speed", speed: nextSpeed }))
          .catch(() => dispatch({ type: "set_toast", message: "sim.set_speed failed" }));
        return;
      }

      if (key === "g") {
        event.preventDefault();
        void postControl("/control/grid", {})
          .catch(() => dispatch({ type: "set_toast", message: "control/grid failed" }));
        return;
      }

      if (key === "m") {
        event.preventDefault();
        void postControl("/control/minimap", {})
          .catch(() => dispatch({ type: "set_toast", message: "control/minimap failed" }));
        return;
      }

      if (key === "escape") {
        event.preventDefault();
        dispatch({ type: "set_selected_military_index", index: null });
        void postControl("/control/deselect", {})
          .catch(() => dispatch({ type: "set_toast", message: "control/deselect failed" }));
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [dispatch, state.speed]);
}
