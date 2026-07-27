import { useEffect, useRef } from "react";
import { useDashboardStore } from "./store";

const KIND_CLASS: Record<string, string> = {
  birth: "birth",
  death: "death",
  diplomacy: "diplomacy",
  tech: "tech",
  disaster: "disaster",
  damage: "damage",
  trade: "trade",
};

export function Notifications() {
  const { state, dispatch } = useDashboardStore();
  const timersRef = useRef<Map<number, number>>(new Map());
  const deadlinesRef = useRef<Map<number, number>>(new Map());

  useEffect(() => {
    const now = Date.now();
    const activeIds = new Set(state.notifications.map((notification) => notification.id));

    for (const notification of state.notifications) {
      const { id } = notification;
      if (!deadlinesRef.current.has(id)) {
        deadlinesRef.current.set(id, now + 5000);
      }
      if (!timersRef.current.has(id)) {
        const delay = Math.max(0, deadlinesRef.current.get(id)! - now);
        const timer = window.setTimeout(() => {
          timersRef.current.delete(id);
          deadlinesRef.current.delete(id);
          dispatch({ type: "dismiss_notification", id });
        }, delay);
        timersRef.current.set(id, timer);
      }
    }

    for (const [id, timer] of timersRef.current) {
      if (!activeIds.has(id)) {
        window.clearTimeout(timer);
        timersRef.current.delete(id);
        deadlinesRef.current.delete(id);
      }
    }
  }, [dispatch, state.notifications]);

  useEffect(() => {
    const timers = timersRef.current;
    return () => timers.forEach((timer) => window.clearTimeout(timer));
  }, []);

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
            {notification.icon.startsWith("/") ? (
              <img src={notification.icon} alt="" />
            ) : (
              notification.icon
            )}
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
