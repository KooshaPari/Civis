import { resolveRendererMode } from "./lib/rendererMode";
import { BabylonScene3d } from "./babylon_scene";
import { Scene3d } from "./scene3d";

/** Select Three (default) or optional Babylon renderer (FR-CIV-WEB-007). */
export function SceneView() {
  const mode = resolveRendererMode(window.location.search);
  if (mode === "babylon") return <BabylonScene3d />;
  return <Scene3d />;
}
