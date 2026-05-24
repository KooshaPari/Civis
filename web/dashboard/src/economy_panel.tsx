import { useEffect, useMemo, useRef } from "react";
import { useDashboardStore } from "./store";

type EconomyHistory = {
  tick: number;
  population: number;
  treasury: Map<number, number>;
  energyBudget: number;
};

export function EconomyPanel() {
  const { state, dispatch } = useDashboardStore();
  const historyRef = useRef<EconomyHistory | null>(null);
  const snapshot = state.snapshot;
  const economy = snapshot?.economy ?? null;
  const initialEnergyBudgetRef = useRef<number | null>(null);

  const derived = useMemo(() => {
    const previous = historyRef.current;
    const currentPopulation = snapshot?.population ?? 0;
    const previousPopulation = previous?.population ?? currentPopulation;
    const populationDelta = currentPopulation - previousPopulation;
    const populationGrowthRate = previous ? populationDelta / Math.max(previous.population, 1) : 0;

    const treasuryRows =
      economy?.faction_treasury.map((row) => ({
        ...row,
        delta: row.balance - (previous?.treasury.get(row.id) ?? row.balance),
      })) ?? [];

    const production = economy?.production_rates ?? null;
    const productionBars = production
      ? [
          { key: "food", label: "Food", value: production.food_per_tick },
          { key: "wood", label: "Wood", value: production.wood_per_tick },
          { key: "metal", label: "Metal", value: production.metal_per_tick },
          { key: "energy", label: "Energy", value: production.energy_per_tick },
        ]
      : [];
    const maxProduction = Math.max(1, ...productionBars.map((row) => Math.abs(row.value)));
    const startingEnergyBudget = initialEnergyBudgetRef.current ?? economy?.energy_budget ?? 0;

    return {
      energyBudget: economy?.energy_budget ?? 0,
      treasuryRows,
      productionBars: productionBars.map((row) => ({
        ...row,
        width: `${Math.min(100, (Math.abs(row.value) / maxProduction) * 100)}%`,
      })),
      populationGrowthRate,
      populationDelta,
      startingEnergyBudget,
    };
  }, [economy, snapshot?.population]);

  useEffect(() => {
    if (!snapshot || !economy) return;
    if (initialEnergyBudgetRef.current == null) {
      initialEnergyBudgetRef.current = economy.energy_budget;
    }
    historyRef.current = {
      tick: snapshot.tick,
      population: snapshot.population,
      treasury: new Map(economy.faction_treasury.map((row) => [row.id, row.balance])),
      energyBudget: economy.energy_budget,
    };
  }, [snapshot, economy]);

  const energyRemaining = derived.energyBudget;
  const energyFill = economy
    ? Math.max(
        0,
        Math.min(100, (energyRemaining / Math.max(derived.startingEnergyBudget, 1)) * 100),
      )
    : 0;

  return (
    <aside className={`economy-panel ${state.economyPanelOpen ? "open" : "closed"}`}>
      <button
        type="button"
        className="panel-toggle"
        onClick={() =>
          dispatch({ type: "set_economy_panel_open", open: !state.economyPanelOpen })
        }
      >
        {state.economyPanelOpen ? "Hide" : "Show"}
      </button>
      {state.economyPanelOpen && (
        <>
          <h2>Economy</h2>
          <p className="inspector-hint">
            Energy, treasury, and production are tracked from the current sim snapshot.
          </p>

          <section className="inspector-section economy-section">
            <h3>Energy budget</h3>
            <div className="economy-budget">
              <div className="economy-budget-track" aria-hidden="true">
                <div className="economy-budget-fill" style={{ width: `${energyFill}%` }} />
              </div>
              <div className="economy-budget-meta">
                <span>Remaining</span>
                <strong>{formatNumber(energyRemaining)}</strong>
              </div>
            </div>
          </section>

          <section className="inspector-section economy-section">
            <h3>Institutions</h3>
            {economy?.institutions?.length ? (
              <div className="economy-table">
                <div className="economy-table-head">
                  <span>ID</span>
                  <span>Kind</span>
                  <span>Joules</span>
                </div>
                {economy.institutions.map((row) => (
                  <div key={row.id} className="economy-table-row">
                    <span>#{row.id}</span>
                    <span>{row.kind}</span>
                    <span>{formatNumber(row.balance_joules)}</span>
                  </div>
                ))}
              </div>
            ) : (
              <p className="economy-empty">No institution ledger on snapshot yet</p>
            )}
          </section>

          <section className="inspector-section economy-section">
            <h3>Faction treasury</h3>
            {derived.treasuryRows.length > 0 ? (
              <div className="economy-table">
                <div className="economy-table-head">
                  <span>ID</span>
                  <span>Name</span>
                  <span>Balance</span>
                </div>
                {derived.treasuryRows.map((row) => (
                  <div key={row.id} className="economy-table-row">
                    <span>#{row.id}</span>
                    <span>{row.name}</span>
                    <span className={row.delta >= 0 ? "trend-up" : "trend-down"}>
                      {row.delta >= 0 ? "▲" : "▼"} {formatNumber(row.balance)}
                    </span>
                  </div>
                ))}
              </div>
            ) : (
              <p className="economy-empty">No treasury data yet</p>
            )}
          </section>

          <section className="inspector-section economy-section">
            <h3>Production rates</h3>
            <div className="economy-bars" role="list" aria-label="Production rates">
              {derived.productionBars.length > 0 ? (
                derived.productionBars.map((bar) => (
                  <div key={bar.key} className="economy-bar-row" role="listitem">
                    <div className="economy-bar-head">
                      <span>{bar.label}</span>
                      <strong>{bar.value.toFixed(1)}/tick</strong>
                    </div>
                    <div className="economy-bar-track">
                      <div className="economy-bar-fill" style={{ width: bar.width }} />
                    </div>
                  </div>
                ))
              ) : (
                <p className="economy-empty">No production data yet</p>
              )}
            </div>
          </section>

          <section className="inspector-section economy-section">
            <h3>Population growth</h3>
            <div className="economy-growth">
              <span>{derived.populationDelta >= 0 ? "▲" : "▼"}</span>
              <strong>{derived.populationGrowthRate.toFixed(3)} / tick</strong>
            </div>
          </section>
        </>
      )}
    </aside>
  );
}

function formatNumber(value: number) {
  return new Intl.NumberFormat("en-US", { maximumFractionDigits: 1 }).format(value);
}
