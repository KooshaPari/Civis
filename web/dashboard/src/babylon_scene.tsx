import React, { useEffect, useRef } from "react";
import { useDashboardStore } from "./store";
import { Scene3d } from "./scene3d";

const TERRAIN_WORLD = 128;
const HEIGHT_SCALE = 12;

type BabylonRuntime = {
  dispose: () => void;
  syncPins: () => void;
  applyTerrain: () => void;
};

/**
 * FR-CIV-WEB-007 (P2): Babylon.js read-only terrain + pin proxies.
 * Falls back to Three.js when `@babylonjs/core` fails to load.
 */
export function BabylonScene3d() {
  const mountRef = useRef<HTMLDivElement | null>(null);
  const runtimeRef = useRef<BabylonRuntime | null>(null);
  const { state } = useDashboardStore();
  const stateRef = useRef(state);
  const [failed, setFailed] = React.useState(false);

  useEffect(() => {
    stateRef.current = state;
  }, [state]);

  useEffect(() => {
    const mount = mountRef.current;
    if (!mount || failed) return;

    let disposed = false;

    void (async () => {
      try {
        const babylon = await import("@babylonjs/core");
        if (disposed) return;

        const canvas = document.createElement("canvas");
        canvas.style.width = "100%";
        canvas.style.height = "100%";
        mount.appendChild(canvas);

        const engine = new babylon.Engine(canvas, true);
        const scene = new babylon.Scene(engine);
        scene.clearColor = new babylon.Color4(0.53, 0.72, 0.88, 1);

        const camera = new babylon.ArcRotateCamera(
          "cam",
          -Math.PI / 4,
          1.1,
          90,
          new babylon.Vector3(TERRAIN_WORLD / 2, 8, TERRAIN_WORLD / 2),
          scene,
        );
        camera.attachControl(canvas, true);
        new babylon.HemisphericLight("sun", new babylon.Vector3(0.2, 1, 0.1), scene);

        const ground = babylon.MeshBuilder.CreateGround(
          "ground",
          { width: TERRAIN_WORLD, height: TERRAIN_WORLD, subdivisions: 32 },
          scene,
        );
        const groundMat = new babylon.StandardMaterial("groundMat", scene);
        groundMat.diffuseColor = new babylon.Color3(0.35, 0.55, 0.32);
        ground.material = groundMat;

        const pinMat = new babylon.StandardMaterial("pinMat", scene);
        pinMat.diffuseColor = new babylon.Color3(0.9, 0.35, 0.35);
        const pinMeshes: import("@babylonjs/core").Mesh[] = [];

        const syncPins = () => {
          for (const mesh of pinMeshes) mesh.dispose();
          pinMeshes.length = 0;
          const snapshot = stateRef.current.snapshot;
          if (!snapshot) return;
          for (const pin of snapshot.civ_pins) {
            const box = babylon.MeshBuilder.CreateBox(
              `pin-${pin.idx}`,
              { size: 1.2, height: 2.4 },
              scene,
            );
            box.material = pinMat;
            box.position = new babylon.Vector3(
              pin.x * TERRAIN_WORLD,
              1.2,
              pin.y * TERRAIN_WORLD,
            );
            pinMeshes.push(box);
          }
        };

        const applyTerrain = () => {
          const terrain = stateRef.current.terrain;
          if (!terrain?.heights?.length) return;
          const positions = ground.getVerticesData(babylon.VertexBuffer.PositionKind);
          if (!positions) return;
          const verts = positions as Float32Array;
          const size = terrain.size;
          for (let i = 0; i < verts.length; i += 3) {
            const u = (verts[i]! + TERRAIN_WORLD / 2) / TERRAIN_WORLD;
            const v = (verts[i + 2]! + TERRAIN_WORLD / 2) / TERRAIN_WORLD;
            const x = Math.min(size - 1, Math.max(0, Math.floor(u * size)));
            const z = Math.min(size - 1, Math.max(0, Math.floor(v * size)));
            const h = terrain.heights[z * size + x] ?? 0;
            verts[i + 1] = h * HEIGHT_SCALE;
          }
          ground.updateVerticesData(babylon.VertexBuffer.PositionKind, verts);
          ground.refreshBoundingInfo(true);
        };

        syncPins();
        applyTerrain();

        engine.runRenderLoop(() => scene.render());
        const onResize = () => engine.resize();
        window.addEventListener("resize", onResize);

        runtimeRef.current = {
          syncPins,
          applyTerrain,
          dispose: () => {
            window.removeEventListener("resize", onResize);
            engine.stopRenderLoop();
            scene.dispose();
            engine.dispose();
            mount.replaceChildren();
          },
        };
      } catch {
        setFailed(true);
      }
    })();

    return () => {
      disposed = true;
      runtimeRef.current?.dispose();
      runtimeRef.current = null;
    };
  }, [failed]);

  useEffect(() => {
    runtimeRef.current?.syncPins();
    runtimeRef.current?.applyTerrain();
  }, [state.snapshot, state.terrain]);

  if (failed) {
    return (
      <div className="babylon-fallback">
        <p className="inspector-hint">
          Babylon viewer unavailable. Using Three.js fallback.
        </p>
        <Scene3d />
      </div>
    );
  }

  return <div ref={mountRef} className="scene-canvas babylon-canvas" />;
}
