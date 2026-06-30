import { useDashboardStore } from "./store";

export function ReligionPanel() {
  const { state, dispatch } = useDashboardStore();
  const { snapshot, religionPanelOpen } = state;
  const profiles = snapshot?.religious_profiles ?? [];

  return (
    <section className="inspector-section" aria-labelledby="religion-heading">
      <h3 id="religion-heading">Religion &amp; Belief</h3>

      <button
        type="button"
        className="panel-expand-btn"
        onClick={() =>
          dispatch({ type: "set_religion_panel_open", open: !religionPanelOpen })
        }
      >
        {religionPanelOpen ? "Collapse" : "Expand"}
      </button>

      {religionPanelOpen && (
        <>
          {profiles.length === 0 ? (
            <p className="stats-empty">
              No religious profiles yet — belief emerges as factions develop.
            </p>
          ) : (
            <ul className="religion-profile-list">
              {profiles.map((p, i) => (
                <li key={i} className="religion-profile-row">
                  <div className="profile-header">
                    <span className="profile-faction">Faction {p.faction_id}</span>
                    <span className="profile-population">
                      {p.population}{" "}
                      {p.population === 1 ? "believer" : "believers"}
                    </span>
                  </div>
                  <div className="profile-bars">
                    <ProfileBar
                      label="Belief"
                      value={p.belief}
                      max={1000000}
                      color="#e0d060"
                    />
                    <ProfileBar
                      label="Coherence"
                      value={p.mythic_coherence}
                      max={1000}
                      color="#c084fc"
                    />
                    <ProfileBar
                      label="Certainty"
                      value={1.0 - p.uncertainty_reduction}
                      max={1.0}
                      color="#60a5fa"
                    />
                  </div>
                  <div className="profile-meta">
                    <span>Age: {p.age_ticks} ticks</span>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </>
      )}
    </section>
  );
}

function ProfileBar({
  label,
  value,
  max,
  color,
}: {
  label: string;
  value: number;
  max: number;
  color: string;
}) {
  const pct = Math.min((value / max) * 100, 100);
  return (
    <div className="profile-bar-row">
      <span className="profile-bar-label">{label}</span>
      <div className="profile-bar-track">
        <div
          className="profile-bar-fill"
          style={{ width: `${pct}%`, background: color }}
        />
      </div>
      <span className="profile-bar-value">{value.toFixed(0)}</span>
    </div>
  );
}
