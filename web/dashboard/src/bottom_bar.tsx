import { postControl } from "./control";
import { useDashboardStore } from "./store";

const MATERIALS = [
  { id: 1, label: "Mud", color: "#7b5c47" },
  { id: 2, label: "Brick", color: "#b46a44" },
  { id: 3, label: "Stone", color: "#8a95a6" },
  { id: 4, label: "Wood", color: "#8b6a45" },
  { id: 5, label: "Sand", color: "#d7bf79" },
  { id: 6, label: "Grass", color: "#4ab866" },
  { id: 7, label: "Arc", color: "#6bbcff" },
];

const ERAS = ["Mud-brick", "Bronze", "Iron", "Steam", "Modern", "Arcology"];

export function BottomBar() {
  const { state, dispatch } = useDashboardStore();

  const runControl = async (path: string) => {
    try {
      await postControl(path, {});
    } catch {
      dispatch({ type: "set_toast", message: `Failed to ${path.replace("/control/", "")}` });
    }
  };

  return (
    <footer className="bottom-bar">
      <div className="tool-row">
        <ToolButton active={state.selectedTool === "PlaceVoxel"} title="Place Voxel" emoji="🧱" onClick={() => dispatch({ type: "set_tool", tool: "PlaceVoxel" })} />
        <ToolButton active={state.selectedTool === "SpawnCivilian"} title="Spawn Civilian" emoji="👤" onClick={() => dispatch({ type: "set_tool", tool: "SpawnCivilian" })} />
        <ToolButton active={state.selectedTool === "DamageBomb"} title="Damage" emoji="💥" onClick={() => dispatch({ type: "set_tool", tool: "DamageBomb" })} />
        <ToolButton active={state.selectedTool === "InspectAgent"} title="Inspect" emoji="🔍" onClick={() => dispatch({ type: "set_tool", tool: "InspectAgent" })} />
        <ToolButton active={state.selectedTool === "Camera"} title="Camera" emoji="🎥" onClick={() => dispatch({ type: "set_tool", tool: "Camera" })} />
        <ToolButton title="Save" emoji="💾" onClick={() => void runControl("/control/save")} />
        <ToolButton title="Load" emoji="📂" onClick={() => void runControl("/control/load")} />
      </div>

      <div className="time-row">
        {[0, 1, 2, 4, 8].map((speed) => (
          <button
            key={speed}
            className={`time-button ${state.speed === speed ? "active" : ""}`}
            title={speed === 0 ? "Pause" : `${speed}x speed`}
            onClick={() => {
              const s = speed as 0 | 1 | 2 | 4 | 8;
              dispatch({ type: "set_speed", speed: s });
              void postControl("/control/speed", { speed: s }).catch(() =>
                dispatch({ type: "set_toast", message: "speed update failed" }),
              );
            }}
          >
            {speed === 0 ? "⏸ Pause" : speed === 1 ? "▶ 1×" : speed === 2 ? "⏩ 2×" : speed === 4 ? "⏩⏩ 4×" : "⏩⏩⏩ 8×"}
          </button>
        ))}
      </div>

      <div className="picker-row">
        <label>
          <span>Material</span>
          <select value={state.selectedMaterial} onChange={(event) => dispatch({ type: "set_material", material: Number(event.target.value) })}>
            {MATERIALS.map((material) => (
              <option key={material.id} value={material.id}>
                {material.id} - {material.label}
              </option>
            ))}
          </select>
        </label>
        <div className="swatches">
          {MATERIALS.map((material) => (
            <button
              key={material.id}
              className={`swatch ${state.selectedMaterial === material.id ? "active" : ""}`}
              title={`Material ${material.id}: ${material.label}`}
              onClick={() => dispatch({ type: "set_material", material: material.id })}
            >
              <span style={{ background: material.color }} />
              {material.id}
            </button>
          ))}
        </div>

        <label>
          <span>Era</span>
          <select value={state.selectedEra} onChange={(event) => dispatch({ type: "set_era", era: Number(event.target.value) })}>
            {ERAS.map((era, index) => (
              <option key={era} value={index}>
                {index} - {era}
              </option>
            ))}
          </select>
        </label>
      </div>
    </footer>
  );
}

function ToolButton({
  title,
  emoji,
  active,
  onClick,
}: {
  title: string;
  emoji: string;
  active?: boolean;
  onClick: () => void;
}) {
  return (
    <button className={`tool-button ${active ? "active" : ""}`} title={title} onClick={onClick}>
      <span aria-hidden>{emoji}</span>
      <small>{title}</small>
    </button>
  );
}
