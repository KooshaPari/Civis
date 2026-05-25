import { useEffect, useMemo, useRef } from "react";
import type { GameEvent, Faction } from "./store";
import { useDashboardStore } from "./store";

const EVENT_ICONS: Record<string, string> = {
  birth: "👶",
  death: "💀",
  trade: "🤝",
  conflict: "⚔️",
  tech: "🔬",
  building: "🏗️",
  peace: "🤝",
};

export function EventFeed() {
  const { state, dispatch } = useDashboardStore();
  const feedRef = useRef<HTMLDivElement | null>(null);
  const events = state.snapshot?.events ?? [];

  useEffect(() => {
    const node = feedRef.current;
    if (!node) return;
    node.scrollTop = node.scrollHeight;
  }, [events.length]);

  const factionsById = useMemo(() => new Map((state.snapshot?.factions ?? []).map((faction) => [faction.id, faction])), [state.snapshot?.factions]);

  return (
    <div className="event-feed" ref={feedRef}>
      {events.length === 0 ? <p className="event-feed-empty">No event feed entries yet.</p> : null}
      <div className="event-feed-list">
        {events.map((event, index) => (
          <EventRow
            key={`${event.tick}-${event.kind}-${index}`}
            event={event}
            faction={event.faction_id != null ? factionsById.get(event.faction_id) ?? null : null}
            onClick={() => dispatch({ type: "set_toast", message: eventLocationLabel(event) })}
          />
        ))}
      </div>
    </div>
  );
}

function EventRow({
  event,
  faction,
  onClick,
}: {
  event: GameEvent;
  faction: Faction | null;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      className="event-feed-row"
      onClick={onClick}
      style={faction ? { borderLeftColor: `rgb(${faction.color[0]} ${faction.color[1]} ${faction.color[2]})` } : undefined}
    >
      <span className="event-feed-tick">tick {event.tick}</span>
      <span className="event-feed-icon">{EVENT_ICONS[event.kind] ?? "•"}</span>
      <span className="event-feed-message">{event.message}</span>
    </button>
  );
}

function eventLocationLabel(event: GameEvent) {
  switch (event.kind) {
    case "birth":
      return event.faction_id != null ? `Faction ${event.faction_id} citizen location` : "Birth location";
    case "death":
      return "Death location";
    case "trade":
      return event.faction_id != null ? `Faction ${event.faction_id} trade route` : "Trade route";
    case "conflict":
      return event.faction_id != null ? `Faction ${event.faction_id} conflict zone` : "Conflict zone";
    case "tech":
      return "Technology unlock location";
    case "building":
      return event.faction_id != null ? `Faction ${event.faction_id} building site` : "Building site";
    default:
      return event.message;
  }
}
