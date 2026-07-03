import {
  useDashboardStore,
  type DiplomacyEvent,
  type DiplomaticTreaty,
} from "./store";

const TREATY_KIND_LABEL: Record<string, string> = {
  TradeAgreement: "Trade Agreement",
  NonAggressionPact: "Non-Aggression Pact",
  ResearchAgreement: "Research Agreement",
  Alliance: "Alliance",
};

const TREATY_COLORS: Record<string, string> = {
  TradeAgreement: "#4caf50",
  NonAggressionPact: "#ff9800",
  ResearchAgreement: "#2196f3",
  Alliance: "#9c27b0",
};

export function DiplomacyPanel() {
  const { state, dispatch } = useDashboardStore();
  const { snapshot, diplomacyPanelOpen } = state;
  const treaties = snapshot?.diplomatic_treaties ?? [];
  const events = snapshot?.diplomacy_events ?? [];

  return (
    <section className="inspector-section" aria-labelledby="diplomacy-heading">
      <h3 id="diplomacy-heading">Diplomacy</h3>

      <button
        type="button"
        className="panel-expand-btn"
        onClick={() =>
          dispatch({ type: "set_diplomacy_panel_open", open: !diplomacyPanelOpen })
        }
      >
        {diplomacyPanelOpen ? "Collapse" : "Expand"}
      </button>

      {diplomacyPanelOpen && (
        <>
          {/* Active Treaties */}
          <div className="diplomacy-section">
            <h4>Active Treaties</h4>
            {treaties.length === 0 ? (
              <p className="stats-empty">No active treaties.</p>
            ) : (
              <ul className="diplomacy-treaty-list">
                {treaties.map((t: DiplomaticTreaty, i: number) => (
                  <li key={i} className="diplomacy-treaty-row">
                    <span
                      className="treaty-dot"
                      style={{ background: TREATY_COLORS[t.treaty_kind] ?? "#888" }}
                    />
                    <span className="treaty-kind">
                      {TREATY_KIND_LABEL[t.treaty_kind] ?? t.treaty_kind}
                    </span>
                    <span className="treaty-parties">
                      F{t.faction_a} &harr; F{t.faction_b}
                    </span>
                    <span className="treaty-ticks">
                      {t.remaining_ticks > 0
                        ? `${t.remaining_ticks} ticks`
                        : "permanent"}
                    </span>
                  </li>
                ))}
              </ul>
            )}
          </div>

          {/* Recent Diplomacy Events */}
          <div className="diplomacy-section">
            <h4>Recent Events</h4>
            {events.length === 0 ? (
              <p className="stats-empty">No diplomatic events yet.</p>
            ) : (
              <ul className="diplomacy-event-list">
                {events
                  .slice(-10)
                  .reverse()
                  .map((ev: DiplomacyEvent, i: number) => (
                    <li key={i} className="diplomacy-event-row">
                      <span className="event-tick">T{ev.tick}</span>
                      <span className="event-kind">{formatEventKind(ev.kind)}</span>
                      <span className="event-parties">
                        F{ev.faction_a} &harr; F{ev.faction_b}
                      </span>
                    </li>
                  ))}
              </ul>
            )}
          </div>
        </>
      )}
    </section>
  );
}

function formatEventKind(kind: string): string {
  switch (kind) {
    case "TradeAgreement":
      return "\u{1F91D} Trade";
    case "Conflict":
      return "\u{2694} Conflict";
    case "Peace":
      return "\u{1F54A} Peace";
    default:
      return kind;
  }
}
