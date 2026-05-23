import React, { useEffect, useRef } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { postControl } from "./control";
import { Biome, CivPin, Terrain, useDashboardStore } from "./store";

const BIOME_COLORS: Record<Biome, number> = {
  deepwater: 0x0e2659,
  water: 0x2c64a8,
  sand: 0xded484,
  grass: 0x689a3c,
  forest: 0x2c6434,
  stone: 0x807c74,
  snow: 0xf0f0f0,
};

const JOB_COLORS: Record<NonNullable<CivPin["job"]>, number> = {
  farmer: 0x7ed957,
  warrior: 0xff6b6b,
  scholar: 0x6fb9ff,
  trader: 0xffd166,
  priest: 0xc084fc,
  admin: 0xb7c0cc,
  unemployed: 0xb7c0cc,
};

const CIVILIAN_POOL_SIZE = 256;
const TERRAIN_HEIGHT_SCALE = 12;

type SceneRefs = {
  terrainMesh: THREE.Mesh<THREE.PlaneGeometry, THREE.MeshStandardMaterial> | null;
  waterMesh: THREE.Mesh<THREE.PlaneGeometry, THREE.MeshStandardMaterial> | null;
  civilians: THREE.Mesh<THREE.BoxGeometry, THREE.MeshStandardMaterial>[];
  activeTerrain: Terrain | null;
  currentTerrainSize: number;
  terrainWorldSize: number;
  targetDayFactor: number;
  dayFactor: number;
};

export function Scene3d() {
  const mountRef = useRef<HTMLDivElement | null>(null);
  const { state, dispatch } = useDashboardStore();
  const stateRef = useRef(state);
  const refs = useRef<SceneRefs>({
    terrainMesh: null,
    waterMesh: null,
    civilians: [],
    activeTerrain: null,
    currentTerrainSize: 0,
    terrainWorldSize: 0,
    targetDayFactor: 1,
    dayFactor: 1,
  });

  useEffect(() => {
    stateRef.current = state;
  }, [state]);

  useEffect(() => {
    const mount = mountRef.current;
    if (!mount) return;

    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x87b7e0);

    const camera = new THREE.PerspectiveCamera(55, 1, 0.1, 1000);
    const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
    renderer.outputColorSpace = THREE.SRGBColorSpace;
    renderer.shadowMap.enabled = true;
    renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setSize(mount.clientWidth, mount.clientHeight, false);
    mount.appendChild(renderer.domElement);

    const controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;
    controls.dampingFactor = 0.08;
    controls.maxPolarAngle = Math.PI / 2 - 0.05;

    const hemisphere = new THREE.HemisphereLight(0x88ccff, 0x442200, 0.6);
    scene.add(hemisphere);
    const sun = new THREE.DirectionalLight(0xffffff, 1.2);
    sun.position.set(0.7, 1.3, 0.4);
    sun.castShadow = true;
    sun.shadow.mapSize.set(2048, 2048);
    sun.shadow.camera.near = 0.1;
    sun.shadow.camera.far = 300;
    sun.shadow.camera.left = -120;
    sun.shadow.camera.right = 120;
    sun.shadow.camera.top = 120;
    sun.shadow.camera.bottom = -120;
    scene.add(sun);
    scene.add(sun.target);

    const terrainGroup = new THREE.Group();
    scene.add(terrainGroup);

    const civilianGroup = new THREE.Group();
    scene.add(civilianGroup);

    const applyTerrain = (terrain: Terrain) => {
      refs.current.activeTerrain = terrain;
      refs.current.currentTerrainSize = terrain.size;
      refs.current.terrainWorldSize = terrain.size;

      if (refs.current.terrainMesh) {
        terrainGroup.remove(refs.current.terrainMesh);
        disposeMesh(refs.current.terrainMesh);
        refs.current.terrainMesh = null;
      }
      if (refs.current.waterMesh) {
        terrainGroup.remove(refs.current.waterMesh);
        disposeMesh(refs.current.waterMesh);
        refs.current.waterMesh = null;
      }

      const geometry = new THREE.PlaneGeometry(terrain.size, terrain.size, terrain.size - 1, terrain.size - 1);
      geometry.rotateX(-Math.PI / 2);
      const positions = geometry.attributes.position as THREE.BufferAttribute;
      const colors = new Float32Array(terrain.size * terrain.size * 3);

      for (let y = 0; y < terrain.size; y += 1) {
        for (let x = 0; x < terrain.size; x += 1) {
          const idx = y * terrain.size + x;
          const height = terrain.heights[idx] * TERRAIN_HEIGHT_SCALE;
          positions.setY(idx, height);
          const color = new THREE.Color(BIOME_COLORS[terrain.biomes[idx]]);
          color.toArray(colors, idx * 3);
        }
      }
      geometry.setAttribute("color", new THREE.BufferAttribute(colors, 3));
      geometry.computeVertexNormals();

      const terrainMaterial = new THREE.MeshStandardMaterial({
        vertexColors: true,
        flatShading: true,
        roughness: 1,
        metalness: 0.02,
      });
      const terrainMesh = new THREE.Mesh(geometry, terrainMaterial);
      terrainMesh.receiveShadow = true;
      terrainGroup.add(terrainMesh);
      refs.current.terrainMesh = terrainMesh;

      const waterGeometry = new THREE.PlaneGeometry(terrain.size, terrain.size);
      waterGeometry.rotateX(-Math.PI / 2);
      const waterMaterial = new THREE.MeshStandardMaterial({
        color: 0x244878,
        transparent: true,
        opacity: 0.6,
        metalness: 0.2,
        roughness: 0.5,
      });
      const waterMesh = new THREE.Mesh(waterGeometry, waterMaterial);
      waterMesh.position.y = 0;
      waterMesh.receiveShadow = true;
      terrainGroup.add(waterMesh);
      refs.current.waterMesh = waterMesh;
      controls.target.set(0, terrain.size * 0.12, 0);
      camera.position.set(terrain.size * 0.6, terrain.size * 0.9, terrain.size * 1.0);
      camera.lookAt(controls.target);
      controls.update();
      updateShadowBounds(sun, terrain.size);
      resizeRenderer();
    };

    const updateCivilians = () => {
      const terrain = refs.current.activeTerrain;
      if (!terrain) return;
      const civs = stateRef.current.snapshot?.civ_pins ?? [];
      while (refs.current.civilians.length < CIVILIAN_POOL_SIZE) {
        const mesh = new THREE.Mesh(
          new THREE.BoxGeometry(0.4, 1.4, 0.4),
          new THREE.MeshStandardMaterial({ roughness: 0.85, metalness: 0.03 }),
        );
        mesh.castShadow = true;
        mesh.receiveShadow = false;
        mesh.visible = false;
        civilianGroup.add(mesh);
        refs.current.civilians.push(mesh);
      }
      refs.current.civilians.forEach((mesh, index) => {
        const pin = civs[index];
        if (!pin) {
          mesh.visible = false;
          return;
        }
        mesh.visible = true;
        const wx = pin.x * terrain.size - terrain.size / 2;
        const wz = pin.y * terrain.size - terrain.size / 2;
        const wy = terrainHeightAt(terrain, pin.x, pin.y) + 0.7;
        mesh.position.set(wx, wy, wz);
        mesh.material.color.setHex(jobColor(pin.job));
      });
    };

    const applyDayNight = () => {
      const target = stateRef.current.snapshot?.is_day === false ? 0.3 : 1;
      refs.current.targetDayFactor = target;
    };

    const onPointerDown = async (event: PointerEvent) => {
      const terrain = refs.current.activeTerrain;
      const terrainMesh = refs.current.terrainMesh;
      if (!terrain || !terrainMesh) return;
      const rect = renderer.domElement.getBoundingClientRect();
      const pointer = new THREE.Vector2(
        ((event.clientX - rect.left) / rect.width) * 2 - 1,
        -(((event.clientY - rect.top) / rect.height) * 2 - 1),
      );
      const raycaster = new THREE.Raycaster();
      raycaster.setFromCamera(pointer, camera);
      const hit = raycaster.intersectObject(terrainMesh, false)[0];
      if (!hit) return;

      const local = terrainMesh.worldToLocal(hit.point.clone());
      const cellX = clampIndex(Math.floor(local.x + terrain.size / 2), terrain.size);
      const cellY = clampIndex(Math.floor(local.z + terrain.size / 2), terrain.size);
      const current = stateRef.current;
      const basePayload = { x: cellX, y: cellY };

      try {
        if (current.selectedTool === "SpawnCivilian") {
          await postControl("/control/spawn_civilian", {
            x: cellX,
            y: Math.max(0, Math.round(hit.point.y)),
            z: cellY,
            era: current.selectedEra,
          });
        } else if (current.selectedTool === "DamageBomb") {
          await postControl("/control/damage", {
            x: cellX,
            y: Math.max(0, Math.round(hit.point.y)),
            z: cellY,
            radius: current.damageRadius,
          });
        } else if (current.selectedTool === "InspectAgent") {
          dispatch({ type: "set_toast", message: `Terrain cell ${basePayload.x}, ${basePayload.y}` });
        } else {
          await postControl("/control/place_voxel", {
            x: cellX,
            y: Math.max(0, Math.round(hit.point.y)),
            z: cellY,
            material: current.selectedMaterial,
          });
        }
      } catch {
        dispatch({ type: "set_toast", message: `Failed to ${controlLabel(current.selectedTool)}` });
      }
    };

    const resizeRenderer = () => {
      const width = mount.clientWidth;
      const height = mount.clientHeight;
      if (width === 0 || height === 0) return;
      camera.aspect = width / height;
      camera.updateProjectionMatrix();
      renderer.setSize(width, height, false);
    };

    const observer = new ResizeObserver(resizeRenderer);
    observer.observe(mount);
    renderer.domElement.addEventListener("pointerdown", onPointerDown);

    scene.background = new THREE.Color(0x87b7e0);
    const fog = new THREE.Fog(0x87b7e0, 0, 1);
    scene.fog = fog;

    let raf = 0;
    const animate = () => {
      raf = window.requestAnimationFrame(animate);
      const terrain = refs.current.activeTerrain;
      if (terrain) {
        refs.current.dayFactor += (refs.current.targetDayFactor - refs.current.dayFactor) * 0.04;
        const d = refs.current.dayFactor;
        hemisphere.intensity = 0.6 * d;
        sun.intensity = 1.2 * d;
        sun.position.set(terrain.size * 0.7, terrain.size * 1.3, terrain.size * 0.55);
        const bg = new THREE.Color().lerpColors(new THREE.Color(0x0a1530), new THREE.Color(0x87b7e0), d);
        scene.background = bg;
        fog.color.copy(bg);
        fog.near = terrain.size * 0.8;
        fog.far = terrain.size * 2.5;
      }
      controls.update();
      renderer.render(scene, camera);
    };

    const initialize = async () => {
      const terrain = stateRef.current.terrain ?? (await terrainLoader());
      if (!stateRef.current.terrain) {
        dispatch({ type: "set_terrain", terrain });
      }
      applyTerrain(terrain);
      updateCivilians();
      applyDayNight();
      animate();
    };

    void initialize();

    return () => {
      observer.disconnect();
      renderer.domElement.removeEventListener("pointerdown", onPointerDown);
      window.cancelAnimationFrame(raf);
      controls.dispose();
      terrainGroup.clear();
      civilianGroup.clear();
      disposeScene(scene);
      renderer.dispose();
      mount.removeChild(renderer.domElement);
    };

  }, [dispatch]);

  useEffect(() => {
    const terrain = refs.current.activeTerrain;
    if (!terrain) return;
    updateCiviliansFromRefs(refs.current, state.snapshot);
    refs.current.targetDayFactor = state.snapshot?.is_day === false ? 0.3 : 1;
  }, [state.snapshot]);

  return <div ref={mountRef} className="scene3d" aria-label="Three.js heightmap scene" />;
}

function updateCiviliansFromRefs(refs: SceneRefs, snapshot: { civ_pins: CivPin[] } | null) {
  const terrain = refs.activeTerrain;
  if (!terrain) return;
  const civs = snapshot?.civ_pins ?? [];
  refs.civilians.forEach((mesh, index) => {
    const pin = civs[index];
    if (!pin) {
      mesh.visible = false;
      return;
    }
    mesh.visible = true;
    const wx = pin.x * terrain.size - terrain.size / 2;
    const wz = pin.y * terrain.size - terrain.size / 2;
    const wy = terrainHeightAt(terrain, pin.x, pin.y) + 0.7;
    mesh.position.set(wx, wy, wz);
    mesh.material.color.setHex(jobColor(pin.job));
  });
}

async function terrainLoader(): Promise<Terrain> {
  const response = await fetch("/terrain");
  if (!response.ok) {
    throw new Error(`GET /terrain failed with ${response.status}`);
  }
  return (await response.json()) as Terrain;
}

function terrainHeightAt(terrain: Terrain, x: number, y: number) {
  const ix = clampIndex(Math.floor(x), terrain.size);
  const iy = clampIndex(Math.floor(y), terrain.size);
  return terrain.heights[iy * terrain.size + ix] * TERRAIN_HEIGHT_SCALE;
}

function clampIndex(value: number, size: number) {
  return Math.max(0, Math.min(size - 1, value));
}

function jobColor(job: CivPin["job"]) {
  if (!job) return 0xb7c0cc;
  return JOB_COLORS[job] ?? 0xb7c0cc;
}

function controlLabel(tool: string) {
  switch (tool) {
    case "SpawnCivilian":
      return "spawn civilian";
    case "DamageBomb":
      return "damage";
    case "InspectAgent":
      return "inspect";
    default:
      return "place voxel";
  }
}

function disposeMesh(mesh: THREE.Mesh<THREE.BufferGeometry, THREE.Material | THREE.Material[]>) {
  mesh.geometry.dispose();
  if (Array.isArray(mesh.material)) {
    mesh.material.forEach((material) => material.dispose());
  } else {
    mesh.material.dispose();
  }
}

function disposeScene(scene: THREE.Scene) {
  scene.traverse((object) => {
    if (!(object instanceof THREE.Mesh)) return;
    const mesh = object as THREE.Mesh<THREE.BufferGeometry, THREE.Material | THREE.Material[]>;
    if (mesh.geometry) mesh.geometry.dispose();
    if (Array.isArray(mesh.material)) {
      mesh.material.forEach((material) => material.dispose());
    } else if (mesh.material) {
      mesh.material.dispose();
    }
  });
}

function updateShadowBounds(light: THREE.DirectionalLight, terrainSize: number) {
  const camera = light.shadow.camera as THREE.OrthographicCamera;
  const extent = terrainSize * 0.8;
  camera.left = -extent;
  camera.right = extent;
  camera.top = extent;
  camera.bottom = -extent;
  camera.updateProjectionMatrix();
}
