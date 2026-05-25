import { agentColorCss } from "./lib/agents";
import { useDashboardStore } from "./store";

export function AgentsPanel() {
  const { state } = useDashboardStore();
  const { seenAgentCount, recentAgentIds } = state;

  return (
    <section className="inspector-section agents-panel" aria-labelledby="agents-heading">
      <h3 id="agents-heading">Agents</h3>
      <div className="agents-metric">
        <span>Seen</span>
        <strong>{seenAgentCount}</strong>
      </div>
      {recentAgentIds.length > 0 ? (
        <ul className="agent-id-list">
          {recentAgentIds.map((id) => (
            <li key={id} className="agent-id-row">
              <span
                className="agent-color-swatch"
                style={{ backgroundColor: agentColorCss(id) }}
                aria-hidden
              />
              <code>#{id}</code>
            </li>
          ))}
        </ul>
      ) : (
        <p className="agents-empty">No AgentAppearance updates yet</p>
      )}
    </section>
  );
}
