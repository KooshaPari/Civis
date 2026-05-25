import { useMemo, useState } from "react";
import { useDashboardStore } from "./store";

type LawDetail = {
  id: string;
  kind: string;
  era_min: number;
  inputs: string[];
  outputs: string[];
  dependencies: string[];
};

const LAW_DETAILS: LawDetail[] = [
  {
    id: "mass_conservation",
    kind: "Conservation",
    era_min: 0,
    inputs: [],
    outputs: [],
    dependencies: [],
  },
  {
    id: "steel",
    kind: "Material",
    era_min: 4,
    inputs: ["iron_ore", "coal"],
    outputs: ["steel_ingot"],
    dependencies: ["mass_conservation"],
  },
  {
    id: "fusion_power",
    kind: "FictionalExtension",
    era_min: 9,
    inputs: ["deuterium"],
    outputs: ["energy"],
    dependencies: ["mass_conservation"],
  },
];

const LAW_DETAIL_MAP = new Map(LAW_DETAILS.map((law) => [law.id, law] as const));

export function TechTreeModal() {
  const { state, dispatch } = useDashboardStore();
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const snapshot = state.snapshot;

  const eras = useMemo(() => {
    const groups = new Map<number, LawDetail[]>();
    for (const law of snapshot?.tech_tree ?? []) {
      if (law.era_min > 5) continue;
      const list = groups.get(law.era_min) ?? [];
      const detail = LAW_DETAIL_MAP.get(law.id);
      if (detail) {
        list.push(detail);
      }
      groups.set(law.era_min, list);
    }
    return Array.from({ length: 6 }, (_, era) => ({
      era,
      laws: groups.get(era) ?? [],
    }));
  }, [snapshot]);

  const selected = selectedId ? LAW_DETAIL_MAP.get(selectedId) : null;

  if (!state.techTreeOpen) {
    return null;
  }

  return (
    <div className="tech-tree-modal" role="dialog" aria-modal="true" aria-label="Tech tree">
      <button
        type="button"
        className="tech-tree-backdrop"
        aria-label="Close tech tree"
        onClick={() => dispatch({ type: "set_tech_tree_open", open: false })}
      />
      <section className="tech-tree-panel">
        <header className="tech-tree-head">
          <div>
            <p className="tech-tree-kicker">Civis research</p>
            <h2>Tech Tree</h2>
            <p className="tech-tree-summary">
              Era {snapshot?.current_era ?? 0} of 5. Laws unlock as the sim advances.
            </p>
          </div>
          <button
            type="button"
            className="panel-toggle"
            onClick={() => dispatch({ type: "set_tech_tree_open", open: false })}
          >
            Close
          </button>
        </header>

        <div className="tech-tree-current-era">
          <span>YOU ARE HERE</span>
          <strong>Era {snapshot?.current_era ?? 0}</strong>
        </div>

        <div className="tech-tree-grid">
          {eras.map(({ era, laws }) => (
            <section key={era} className={`tech-era ${snapshot?.current_era === era ? "current" : ""}`}>
              <div className="tech-era-mark">
                <span className="tech-era-dot" />
                <strong>Era {era}</strong>
              </div>
              <div className="tech-era-laws">
                {laws.length ? (
                  laws.map((law) => {
                    const node = snapshot?.tech_tree.find((item) => item.id === law.id);
                    const unlocked = node?.unlocked ?? false;
                    return (
                      <button
                        key={law.id}
                        type="button"
                        className={`tech-card ${unlocked ? "unlocked" : "locked"} ${selectedId === law.id ? "selected" : ""}`}
                        onClick={() => setSelectedId((current) => (current === law.id ? null : law.id))}
                        title={`${law.id} | deps: ${law.dependencies.join(", ") || "none"} | in: ${
                          law.inputs.join(", ") || "none"
                        } | out: ${law.outputs.join(", ") || "none"}`}
                      >
                        <span className="tech-card-id">{law.id}</span>
                        <span className="tech-card-meta">
                          {law.kind} · Era {law.era_min}
                        </span>
                      </button>
                    );
                  })
                ) : (
                  <p className="tech-era-empty">No laws unlocked in this era.</p>
                )}
              </div>
            </section>
          ))}
        </div>

        <aside className="tech-tooltip" aria-live="polite">
          {selected ? (
            <>
              <h3>{selected.id}</h3>
              <p>{selected.kind} · Era {selected.era_min}</p>
              <dl>
                <div>
                  <dt>Dependencies</dt>
                  <dd>{selected.dependencies.length ? selected.dependencies.join(", ") : "none"}</dd>
                </div>
                <div>
                  <dt>Inputs</dt>
                  <dd>{selected.inputs.length ? selected.inputs.join(", ") : "none"}</dd>
                </div>
                <div>
                  <dt>Outputs</dt>
                  <dd>{selected.outputs.length ? selected.outputs.join(", ") : "none"}</dd>
                </div>
              </dl>
            </>
          ) : (
            <p>Select a law card to inspect dependencies and I/O.</p>
          )}
        </aside>
      </section>
    </div>
  );
}
