import { useEffect, useRef } from "react";
import { postControl } from "../control";
import { useDashboardStore, type TimeSpeed } from "../store";
import { isDashboardShortcutTarget } from "../../../src/shortcutTarget.mjs";

export { isDashboardShortcutTarget };

const SPEED_KEYS: Record<string, TimeSpeed> = {
  "1": 1,
  "2": 2,
  "3": 4,
};

async function setSpeed(speed: TimeSpeed) {
  await postControl("/control/speed", { speed });
}

export function useDashboardShortcuts() {
  const { state, dispatch } = useDashboardStore();
  const speedRef = useRef(state.speed);

  useEffect(() => {
    speedRef.current = state.speed;
  }, [state.speed]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.repeat || event.defaultPrevented || event.isComposing) return;
      if (event.ctrlKey || event.metaKey || event.altKey) return;
      if (isDashboardShortcutTarget(event.target)) return;

      const key = event.key.toLowerCase();
      const isSpace = event.code === "Space" || key === " ";

      if (isSpace) {
        event.preventDefault();
        const nextSpeed = speedRef.current === 0 ? 1 : 0;
        void setSpeed(nextSpeed)
          .then(() => dispatch({ type: "set_speed", speed: nextSpeed }))
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
  }, [dispatch]);
}
