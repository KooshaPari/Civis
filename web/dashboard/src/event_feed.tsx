import { useEffect, useMemo, useRef } from "react";
import type { GameEvent, Faction } from "./store";
import { useDashboardStore } from "./store";

const EVENT_ICONS: Record<string, string> = {
  birth: "/civis-icons/spawn-life.png",
  death: "/civis-icons/erase.png",
  trade: "/civis-icons/spawn-material.png",
  conflict: "/civis-icons/diplomacy.png",
  tech: "/civis-icons/infra.png",
  building: "/civis-icons/spawn-structure.png",
  peace: "/civis-icons/diplomacy.png",
  disaster: "/civis-icons/disaster.png",
  damage: "/civis-icons/disaster.png",
};

export function EventFeed() {
  const { state, dispatch } = useDashboardStore();
  const feedRef = useRef<HTMLDivElement | null>(null);
  const events = useMemo(() => {
    const snapshot = state.snapshot;
    if (!snapshot) return [];
    const combatEvents = snapshot.damage_events.map((event) => ({
      tick: snapshot.tick,
      kind: "damage",
      message: `Combat damage at ${event.x.toFixed(2)}, ${event.y.toFixed(2)}`,
      faction_id: null,
    }));
    return [...snapshot.events, ...combatEvents].sort((a, b) => a.tick - b.tick);
  }, [state.snapshot]);

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
      <span className="event-feed-icon">
        {EVENT_ICONS[event.kind]?.startsWith("/") ? (
          <img src={EVENT_ICONS[event.kind]} alt="" />
        ) : (
          EVENT_ICONS[event.kind] ?? "•"
        )}
      </span>
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
    case "damage":
      return "Combat damage location";
    case "building":
      return event.faction_id != null ? `Faction ${event.faction_id} building site` : "Building site";
    default:
      return event.message;
  }
}
