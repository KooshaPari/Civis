import { lazy, Suspense } from "react";
import { resolveRendererMode } from "./lib/rendererMode";

const BabylonScene3d = lazy(() =>
  import("./babylon_scene").then(({ BabylonScene3d: component }) => ({ default: component })),
);
const Scene3d = lazy(() => import("./scene3d").then(({ Scene3d: component }) => ({ default: component })));

/** Select Three (default) or optional Babylon renderer (FR-CIV-WEB-007). */
export function SceneView() {
  const mode = resolveRendererMode(window.location.search);
  return (
    <Suspense
      fallback={
        <div className="scene-renderer-loading" role="status" aria-live="polite" aria-atomic="true">
          Loading world renderer...
        </div>
      }
    >
      {mode === "babylon" ? <BabylonScene3d /> : <Scene3d />}
    </Suspense>
  );
}
