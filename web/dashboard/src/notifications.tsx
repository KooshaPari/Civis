import { useEffect } from "react";
import { useDashboardStore } from "./store";

const KIND_CLASS: Record<string, string> = {
  birth: "birth",
  death: "death",
  diplomacy: "diplomacy",
  tech: "tech",
  disaster: "disaster",
  trade: "trade",
};

export function Notifications() {
  const { state, dispatch } = useDashboardStore();

  useEffect(() => {
    if (state.notifications.length === 0) return;
    const timers = state.notifications.map((notification) =>
      window.setTimeout(() => {
        dispatch({ type: "dismiss_notification", id: notification.id });
      }, 5000),
    );
    return () => timers.forEach((timer) => window.clearTimeout(timer));
  }, [dispatch, state.notifications]);

  return (
    <aside className="notification-panel" aria-label="Recent game events">
      {state.notifications.map((notification) => (
        <button
          key={notification.id}
          type="button"
          className={`notification-card ${KIND_CLASS[notification.kind]}`}
          onClick={() => {
            if (notification.focus) {
              dispatch({ type: "set_camera_focus", focus: notification.focus });
            }
            dispatch({ type: "dismiss_notification", id: notification.id });
          }}
        >
          <span className="notification-icon" aria-hidden>
            {notification.icon}
          </span>
          <span className="notification-body">
            <strong>{notification.message}</strong>
            <span>tick {notification.tick}</span>
          </span>
        </button>
      ))}
    </aside>
  );
}
