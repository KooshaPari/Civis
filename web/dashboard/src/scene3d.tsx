import React, { useEffect, useRef } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { postControl } from "./control";
import { useDashboardStore } from "./store";

const MATERIAL_COLORS: Record<number, number> = {
  1: 0x7b5c47,
  2: 0xb46a44,
  3: 0x8a95a6,
  4: 0x8b6a45,
  5: 0xd7bf79,
  6: 0x4ab866,
  7: 0x6bbcff,
};

export function Scene3d() {
  const mountRef = useRef<HTMLDivElement | null>(null);
  const sceneRef = useRef<THREE.Scene | null>(null);
  const cameraRef = useRef<THREE.PerspectiveCamera | null>(null);
  const rendererRef = useRef<THREE.WebGLRenderer | null>(null);
  const controlsRef = useRef<OrbitControls | null>(null);
  const frameRef = useRef<number | null>(null);
  const pulseRef = useRef<THREE.Mesh | null>(null);
  const voxelGroupRef = useRef<THREE.Group | null>(null);
  const civilianMeshesRef = useRef<THREE.Mesh[]>([]);
  const raycasterRef = useRef(new THREE.Raycaster());
  const pointerRef = useRef(new THREE.Vector2());
  const keysRef = useRef(new Set<string>());
  const { state, dispatch } = useDashboardStore();

  useEffect(() => {
    const mount = mountRef.current;
    if (!mount) return;

    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x07111d);
    scene.fog = new THREE.Fog(0x07111d, 14, 40);

    const camera = new THREE.PerspectiveCamera(55, 1, 0.1, 100);
    camera.position.set(10, 9, 12);

    const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setSize(mount.clientWidth, mount.clientHeight, false);
    renderer.outputColorSpace = THREE.SRGBColorSpace;
    mount.appendChild(renderer.domElement);

    const controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;
    controls.dampingFactor = 0.08;
    controls.target.set(0, 0.8, 0);

    scene.add(new THREE.AmbientLight(0xb9d7ff, 1.2));
    const directional = new THREE.DirectionalLight(0xffffff, 2.2);
    directional.position.set(7, 11, 6);
    scene.add(directional);

    const gridHelper = new THREE.GridHelper(16, 16, 0x38506f, 0x1d2c40);
    gridHelper.position.y = -1.25;
    scene.add(gridHelper);

    const axes = new THREE.AxesHelper(3);
    axes.position.y = -1.2;
    scene.add(axes);

    const voxelGroup = new THREE.Group();
    const voxelGeometry = new THREE.BoxGeometry(0.82, 0.82, 0.82);
    for (let x = 0; x < 8; x += 1) {
      for (let y = 0; y < 8; y += 1) {
        for (let z = 0; z < 8; z += 1) {
          const voxel = new THREE.Mesh(
            voxelGeometry,
            new THREE.MeshStandardMaterial({
              color: 0x2f4d73,
              roughness: 0.85,
              metalness: 0.05,
              transparent: true,
              opacity: 0.88,
            }),
          );
          voxel.position.set((x - 3.5) * 0.95, (y - 3.5) * 0.95, (z - 3.5) * 0.95);
          voxel.userData.baseY = voxel.position.y;
          voxel.userData.grid = { x, y, z };
          voxelGroup.add(voxel);
        }
      }
    }
    scene.add(voxelGroup);

    const pulseCube = new THREE.Mesh(
      new THREE.BoxGeometry(1, 1, 1),
      new THREE.MeshStandardMaterial({
        color: 0x7ee0ff,
        emissive: 0x19465b,
        emissiveIntensity: 1.2,
        roughness: 0.35,
      }),
    );
    scene.add(pulseCube);

    const onPointerDown = async (event: PointerEvent) => {
      const rect = renderer.domElement.getBoundingClientRect();
      pointerRef.current.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
      pointerRef.current.y = -(((event.clientY - rect.top) / rect.height) * 2 - 1);
      raycasterRef.current.setFromCamera(pointerRef.current, camera);

      const hits = raycasterRef.current.intersectObjects(voxelGroup.children, false);
      const hitPoint = hits[0]?.point ?? intersectGround(raycasterRef.current, camera);
      if (!hitPoint) return;

      if (state.selectedTool === "PlaceVoxel") {
        const grid = snapToGrid(hitPoint);
        await sendControl(dispatch, "/control/place_voxel", {
          x: grid.x,
          y: grid.y,
          z: grid.z,
          material: state.selectedMaterial,
        });
      } else if (state.selectedTool === "SpawnCivilian") {
        await sendControl(dispatch, "/control/spawn_civilian", {
          x: Math.round(hitPoint.x),
          y: Math.max(0, Math.round(hitPoint.y)),
          z: Math.round(hitPoint.z),
          era: state.selectedEra,
        });
      } else if (state.selectedTool === "DamageBomb") {
        await sendControl(dispatch, "/control/damage", {
          x: Math.round(hitPoint.x),
          y: Math.max(0, Math.round(hitPoint.y)),
          z: Math.round(hitPoint.z),
          radius: state.damageRadius,
        });
      } else if (state.selectedTool === "InspectAgent") {
        const pickedCivilian = pickCivilian(hitPoint, civilianMeshesRef.current);
        dispatch({ type: "set_selected_civilian", civilian: pickedCivilian });
      }
    };

    const onKeyDown = (event: KeyboardEvent) => {
      keysRef.current.add(event.key.toLowerCase());
      if (event.key.toLowerCase() === "r") {
        controls.target.set(0, 0.8, 0);
        camera.position.set(10, 9, 12);
        controls.update();
      }
    };

    const onKeyUp = (event: KeyboardEvent) => {
      keysRef.current.delete(event.key.toLowerCase());
    };

    const resize = () => {
      const width = mount.clientWidth;
      const height = mount.clientHeight;
      camera.aspect = width / height;
      camera.updateProjectionMatrix();
      renderer.setSize(width, height, false);
    };

    const observer = new ResizeObserver(resize);
    observer.observe(mount);
    renderer.domElement.addEventListener("pointerdown", onPointerDown);
    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("keyup", onKeyUp);

    sceneRef.current = scene;
    cameraRef.current = camera;
    rendererRef.current = renderer;
    controlsRef.current = controls;
    voxelGroupRef.current = voxelGroup;
    pulseRef.current = pulseCube;

    const animate = () => {
      frameRef.current = window.requestAnimationFrame(animate);
      handlePan(camera, controls);
      controls.update();
      renderer.render(scene, camera);
    };

    resize();
    animate();

    return () => {
      observer.disconnect();
      renderer.domElement.removeEventListener("pointerdown", onPointerDown);
      window.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("keyup", onKeyUp);
      controls.dispose();
      renderer.dispose();
      voxelGeometry.dispose();
      pulseCube.geometry.dispose();
      (pulseCube.material as THREE.Material).dispose();
      voxelGroup.children.forEach((child) => {
        const mesh = child as THREE.Mesh;
        (mesh.geometry as THREE.BufferGeometry).dispose();
        (mesh.material as THREE.Material).dispose();
      });
      mount.removeChild(renderer.domElement);
      if (frameRef.current !== null) window.cancelAnimationFrame(frameRef.current);
      scene.clear();
    };
  }, [dispatch, state.damageRadius, state.selectedEra, state.selectedMaterial, state.selectedTool]);

  useEffect(() => {
    const pulse = pulseRef.current;
    const voxelGroup = voxelGroupRef.current;
    const scene = sceneRef.current;
    if (!pulse || !voxelGroup || !scene) return;

    const tick = state.snapshot?.tick ?? 0;
    pulse.scale.setScalar(1 + Math.sin(((tick % 24) / 24) * Math.PI) * 0.5);

    voxelGroup.children.forEach((child, index) => {
      const mesh = child as THREE.Mesh;
      const baseY = mesh.userData.baseY as number;
      mesh.position.y = baseY + Math.sin(tick * 0.08 + index * 0.18) * 0.08;
      (mesh.material as THREE.MeshStandardMaterial).color.setHex(MATERIAL_COLORS[state.selectedMaterial] ?? 0x2f4d73);
    });

    updateCivilianMeshes(scene, civilianMeshesRef, state.snapshot, dispatch);
  }, [dispatch, state.selectedMaterial, state.snapshot]);

  return <div ref={mountRef} className="scene3d" aria-label="Three.js voxel scene" />;
}

async function sendControl(
  dispatch: React.Dispatch<{ type: "set_toast"; message: string | null } | { type: "clear_toast" }>,
  path: string,
  body: Record<string, unknown>,
) {
  try {
    await postControl(path, body);
  } catch {
    dispatch({ type: "set_toast", message: `Failed to ${path.replace("/control/", "")}` });
  }
}

function intersectGround(raycaster: THREE.Raycaster, camera: THREE.PerspectiveCamera) {
  const plane = new THREE.Plane(new THREE.Vector3(0, 1, 0), 0);
  const point = new THREE.Vector3();
  return raycaster.ray.intersectPlane(plane, point) ? point : camera.position.clone();
}

function snapToGrid(point: THREE.Vector3) {
  return {
    x: Math.round(point.x),
    y: Math.max(0, Math.round(point.y)),
    z: Math.round(point.z),
  };
}

function pickCivilian(point: THREE.Vector3, meshes: THREE.Mesh[]) {
  const nearest = meshes.reduce<{ mesh: THREE.Mesh | null; distance: number }>(
    (best, mesh) => {
      const distance = mesh.position.distanceTo(point);
      return distance < best.distance ? { mesh, distance } : best;
    },
    { mesh: null, distance: Number.POSITIVE_INFINITY },
  );
  return (nearest.mesh?.userData.civilian as unknown as Record<string, unknown>) ?? null;
}

function handlePan(camera: THREE.PerspectiveCamera, controls: OrbitControls) {
  const step = 0.18;
  const keys = Array.from((window as unknown as { __civisKeys?: Set<string> }).__civisKeys ?? []);
  if (keys.includes("w")) camera.position.z -= step;
  if (keys.includes("s")) camera.position.z += step;
  if (keys.includes("a")) camera.position.x -= step;
  if (keys.includes("d")) camera.position.x += step;
  controls.target.x = camera.position.x * 0.15;
}

function updateCivilianMeshes(
  scene: THREE.Scene,
  civilianMeshesRef: React.MutableRefObject<THREE.Mesh[]>,
  snapshot: { sample_civilians: { age: number; health: number; ideology: number; welfare: number; job: string | null }[] } | null,
  dispatch: React.Dispatch<{ type: "set_selected_civilian"; civilian: unknown }>,
) {
  const civilians = snapshot?.sample_civilians ?? [];
  const existing = civilianMeshesRef.current;

  while (existing.length > civilians.length) {
    const mesh = existing.pop();
    if (!mesh) continue;
    scene.remove(mesh);
    (mesh.geometry as THREE.SphereGeometry).dispose();
    (mesh.material as THREE.MeshStandardMaterial).dispose();
  }

  while (existing.length < civilians.length) {
    const mesh = new THREE.Mesh(
      new THREE.SphereGeometry(0.22, 20, 20),
      new THREE.MeshStandardMaterial({ roughness: 0.4, metalness: 0.05 }),
    );
    existing.push(mesh);
    scene.add(mesh);
  }

  civilians.forEach((civilian, index) => {
    const mesh = existing[index];
    mesh.position.copy(deriveCivilianPosition(civilian, index, snapshot?.tick ?? 0));
    (mesh.material as THREE.MeshStandardMaterial).color.setHex(jobColor(civilian.job));
    mesh.scale.setScalar(0.9 + civilian.health * 0.35);
    mesh.userData.civilian = civilian;
    mesh.userData.type = "civilian";
    if (index === 0) dispatch({ type: "set_selected_civilian", civilian });
  });
}

function jobColor(job: string | null) {
  switch (job) {
    case "Farmer":
      return 0x53d36b;
    case "Warrior":
      return 0xff6262;
    case "Scholar":
      return 0x5db2ff;
    case "Trader":
      return 0xffd65a;
    case "Priest":
      return 0xc78bff;
    case "Admin":
      return 0x8c96a8;
    default:
      return 0x9fb3d1;
  }
}

function deriveCivilianPosition(
  civilian: { age: number; health: number; ideology: number; welfare: number },
  index: number,
  tick: number,
) {
  const seedA = hashCivilian(civilian.age, civilian.health, civilian.ideology, civilian.welfare, index, tick);
  const seedB = hashCivilian(civilian.age + 11, civilian.health + 17, civilian.ideology + 23, civilian.welfare + 31, index + 7, tick + 13);
  const seedC = hashCivilian(civilian.age + 29, civilian.health + 37, civilian.ideology + 41, civilian.welfare + 43, index + 19, tick + 3);
  const radius = 2.2 + civilian.welfare * 1.8;
  const angle = seedA * Math.PI * 2;
  const height = (seedB - 0.5) * 4.2 + civilian.ideology * 0.8;
  const wobble = Math.sin(tick * 0.04 + seedC * Math.PI * 2) * 0.4;
  return new THREE.Vector3(Math.cos(angle) * (radius + wobble), height, Math.sin(angle) * (radius + wobble));
}

function hashCivilian(...values: number[]) {
  let state = 2166136261;
  for (const value of values) {
    const mixed = Math.floor((value + 1) * 1_000_000);
    state ^= mixed;
    state = Math.imul(state, 16777619);
  }
  return (state >>> 0) / 4294967295;
}

