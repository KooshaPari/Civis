import React, { useEffect, useRef } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { executeTerrainAuthoring } from "./lib/authoring";
import {
  Biome,
  Building,
  CivPin,
  Faction,
  MilitaryPin,
  Road,
  Snapshot,
  Terrain,
  useDashboardStore,
} from "./store";

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
  decorationGroup: THREE.Group | null;
  treeInstances: THREE.InstancedMesh<THREE.BufferGeometry, THREE.MeshStandardMaterial> | null;
  rockInstances: THREE.InstancedMesh<THREE.BufferGeometry, THREE.MeshStandardMaterial> | null;
  snowPoints: THREE.Points<THREE.BufferGeometry, THREE.PointsMaterial> | null;
  rainPoints: THREE.Points<THREE.BufferGeometry, THREE.PointsMaterial> | null;
  civilians: THREE.Mesh<THREE.BoxGeometry, THREE.MeshStandardMaterial>[];
  military: THREE.Mesh<THREE.ConeGeometry, THREE.MeshStandardMaterial>[];
  territories: THREE.Mesh<THREE.CircleGeometry, THREE.MeshStandardMaterial>[];
  buildings: THREE.Mesh<THREE.BoxGeometry, THREE.MeshStandardMaterial>[];
  roads: THREE.Mesh<THREE.BoxGeometry, THREE.MeshStandardMaterial>[];
  tradeRoutes: THREE.Line<THREE.BufferGeometry, THREE.LineDashedMaterial>[];
  effects: THREE.Group | null;
  transientSprites: THREE.Sprite[];
  activeTerrain: Terrain | null;
  currentTerrainSize: number;
  terrainWorldSize: number;
  terrainBaseColors: Float32Array | null;
  terrainSeason: string;
  terrainWeather: Snapshot["weather"] | null;
  targetDayFactor: number;
  dayFactor: number;
  previousSnapshot: Snapshot | null;
  currentSnapshot: Snapshot | null;
  snapshotReceivedAt: number;
  spawnBurst?: (x: number, y: number, color: number, label?: string) => void;
};

export function Scene3d() {
  const mountRef = useRef<HTMLDivElement | null>(null);
  const cameraRef = useRef<THREE.PerspectiveCamera | null>(null);
  const controlsRef = useRef<OrbitControls | null>(null);
  const { state, dispatch } = useDashboardStore();
  const stateRef = useRef(state);
  const refs = useRef<SceneRefs>({
    terrainMesh: null,
    waterMesh: null,
    decorationGroup: null,
    treeInstances: null,
    rockInstances: null,
    snowPoints: null,
    rainPoints: null,
    civilians: [],
    military: [],
    territories: [],
    buildings: [],
    roads: [],
    tradeRoutes: [],
    effects: null,
    transientSprites: [],
    activeTerrain: null,
    currentTerrainSize: 0,
    terrainWorldSize: 0,
    terrainBaseColors: null,
    terrainSeason: "",
    terrainWeather: null,
    targetDayFactor: 1,
    dayFactor: 1,
    previousSnapshot: null,
    currentSnapshot: null,
    snapshotReceivedAt: performance.now(),
  });

  useEffect(() => {
    stateRef.current = state;
  }, [state]);

  useEffect(() => {
    const camera = cameraRef.current;
    const controls = controlsRef.current;
    const terrain = refs.current.activeTerrain;
    const preset = state.cameraPreset;
    if (!camera || !controls || !terrain || !preset) return;
    const size = terrain.size;
    const targetY = size * 0.12;
    controls.target.set(0, targetY, 0);
    switch (preset) {
      case "wide":
        camera.position.set(size * 0.5, size * 2.2, size * 1.6);
        break;
      case "close":
        camera.position.set(size * 0.15, size * 0.55, size * 0.4);
        break;
      case "orbit":
        camera.position.set(size * 0.25, size * 1.7, size * 1.6);
        break;
      default:
        break;
    }
    camera.lookAt(controls.target);
    controls.update();
  }, [state.cameraPresetToken, state.cameraPreset]);

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
    cameraRef.current = camera;
    controlsRef.current = controls;
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
    const territoryGroup = new THREE.Group();
    scene.add(territoryGroup);
    const roadGroup = new THREE.Group();
    scene.add(roadGroup);
    const tradeRouteGroup = new THREE.Group();
    scene.add(tradeRouteGroup);
    const effectGroup = new THREE.Group();
    scene.add(effectGroup);
    refs.current.effects = effectGroup;

    const civilianGroup = new THREE.Group();
    scene.add(civilianGroup);
    const militaryGroup = new THREE.Group();
    scene.add(militaryGroup);
    const buildingGroup = new THREE.Group();
    scene.add(buildingGroup);

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
      if (refs.current.decorationGroup) {
        terrainGroup.remove(refs.current.decorationGroup);
        disposeObject(refs.current.decorationGroup);
        refs.current.decorationGroup = null;
        refs.current.treeInstances = null;
        refs.current.rockInstances = null;
        refs.current.snowPoints = null;
        refs.current.rainPoints = null;
      }
      if (refs.current.treeInstances) {
        terrainGroup.remove(refs.current.treeInstances);
        disposeObject(refs.current.treeInstances);
        refs.current.treeInstances = null;
      }
      if (refs.current.rockInstances) {
        terrainGroup.remove(refs.current.rockInstances);
        disposeObject(refs.current.rockInstances);
        refs.current.rockInstances = null;
      }
      if (refs.current.snowPoints) {
        terrainGroup.remove(refs.current.snowPoints);
        disposeObject(refs.current.snowPoints);
        refs.current.snowPoints = null;
      }
      if (refs.current.rainPoints) {
        terrainGroup.remove(refs.current.rainPoints);
        disposeObject(refs.current.rainPoints);
        refs.current.rainPoints = null;
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
      refs.current.terrainBaseColors = colors.slice();
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
      buildDecorations(terrain, terrainGroup, refs.current);
      controls.target.set(0, terrain.size * 0.12, 0);
      camera.position.set(terrain.size * 0.25, terrain.size * 1.7, terrain.size * 1.6);
      camera.lookAt(controls.target);
      controls.update();
      updateShadowBounds(sun, terrain.size);
      resizeRenderer();
    };

    const updateCivilians = () => {
      const terrain = refs.current.activeTerrain;
      if (!terrain) return;
      const snapshot = stateRef.current.snapshot;
      const civs = snapshot?.civ_pins ?? [];
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
        const sample = interpolateCivPin(refs.current, index, performance.now());
        const wx = sample.x * terrain.size - terrain.size / 2;
        const wz = sample.y * terrain.size - terrain.size / 2;
        const wy = terrainHeightAt(terrain, sample.x, sample.y) + 0.7;
        mesh.position.set(wx, wy, wz);
        mesh.material.color.setHex(jobColor(pin.job));
      });
    };

    const updateMilitary = () => {
      const terrain = refs.current.activeTerrain;
      const snapshot = stateRef.current.snapshot;
      if (!terrain) return;
      const units = snapshot?.military_units ?? [];
      const combat = new Set<string>();
      while (refs.current.military.length < units.length) {
        const mesh = new THREE.Mesh(
          new THREE.ConeGeometry(0.42, 1.05, 6),
          new THREE.MeshStandardMaterial({
            roughness: 0.6,
            metalness: 0.2,
            emissive: new THREE.Color(0x000000),
            emissiveIntensity: 0,
          }),
        );
        mesh.castShadow = true;
        mesh.receiveShadow = false;
        mesh.visible = false;
        militaryGroup.add(mesh);
        refs.current.military.push(mesh);
      }
      refs.current.military.forEach((mesh, index) => {
        const unit = units[index];
        if (!unit) {
          mesh.visible = false;
          return;
        }
        mesh.visible = true;
        const wx = unit.x * terrain.size - terrain.size / 2;
        const wz = unit.y * terrain.size - terrain.size / 2;
        const wy = terrainHeightAt(terrain, unit.x, unit.y) + 0.85;
        mesh.position.set(wx, wy, wz);
        mesh.scale.setScalar(1 + unit.strength * 0.15);
        mesh.material.color.setHex(factionColor(snapshot?.factions ?? [], unit.faction));
        mesh.material.emissive.setHex(combat.has(`${unit.x.toFixed(3)}:${unit.y.toFixed(3)}`) ? 0xff2222 : 0x000000);
        mesh.material.emissiveIntensity = combat.has(`${unit.x.toFixed(3)}:${unit.y.toFixed(3)}`) ? 0.8 : 0;
      });
    };

    const updateFactions = () => {
      const terrain = refs.current.activeTerrain;
      const snapshot = stateRef.current.snapshot;
      if (!terrain) return;
      const factions = snapshot?.factions ?? [];
      const conflicted = new Set(
        snapshot?.diplomacy_events
          ?.filter((event) => event.kind === "Conflict")
          .flatMap((event) => [event.faction_a, event.faction_b]) ?? [],
      );
      while (refs.current.territories.length < factions.length) {
        const mesh = new THREE.Mesh(
          new THREE.CircleGeometry(1, 64),
          new THREE.MeshStandardMaterial({
            color: 0xffffff,
            transparent: true,
            opacity: 0.18,
            roughness: 1,
            metalness: 0,
            depthWrite: false,
            side: THREE.DoubleSide,
          }),
        );
        mesh.rotation.x = -Math.PI / 2;
        mesh.receiveShadow = false;
        mesh.castShadow = false;
        territoryGroup.add(mesh);
        refs.current.territories.push(mesh);
      }
      refs.current.territories.forEach((mesh, index) => {
        const faction = factions[index];
        if (!faction) {
          mesh.visible = false;
          return;
        }
        mesh.visible = true;
        mesh.position.set(
          faction.capital[0] * terrain.size - terrain.size / 2,
          terrainHeightAt(terrain, faction.capital[0], faction.capital[1]) + 0.25,
          faction.capital[1] * terrain.size - terrain.size / 2,
        );
        mesh.scale.setScalar(faction.radius);
        mesh.material.color.setRGB(faction.color[0] / 255, faction.color[1] / 255, faction.color[2] / 255);
        mesh.material.opacity = conflicted.has(faction.id) ? 0.3 : 0.12 + index * 0.03;
        mesh.material.color.setHex(conflicted.has(faction.id) ? 0xff4d4d : mesh.material.color.getHex());
      });
    };

    const updateBuildings = () => {
      const terrain = refs.current.activeTerrain;
      const snapshot = stateRef.current.snapshot;
      if (!terrain) return;
      const buildings = snapshot?.buildings ?? [];
      while (refs.current.buildings.length < buildings.length) {
        const mesh = new THREE.Mesh(
          new THREE.BoxGeometry(1, 1, 1),
          new THREE.MeshStandardMaterial({ roughness: 0.92, metalness: 0.04 }),
        );
        mesh.castShadow = true;
        mesh.receiveShadow = true;
        buildingGroup.add(mesh);
        refs.current.buildings.push(mesh);
      }
      refs.current.buildings.forEach((mesh, index) => {
        const building = buildings[index];
        if (!building) {
          mesh.visible = false;
          return;
        }
        mesh.visible = true;
        const faction = factionById(snapshot?.factions ?? [], building.faction_id);
        const dims = buildingDimensions(building);
        mesh.scale.set(dims[0], dims[1], dims[2]);
        const wx = building.x * terrain.size - terrain.size / 2;
        const wz = building.y * terrain.size - terrain.size / 2;
        const wy = terrainHeightAt(terrain, building.x, building.y) + dims[1] * 0.5;
        mesh.position.set(wx, wy, wz);
        if (faction) {
          mesh.material.color.setRGB(
            Math.min(1, faction.color[0] / 255 * 0.8 + 0.15),
            Math.min(1, faction.color[1] / 255 * 0.8 + 0.15),
            Math.min(1, faction.color[2] / 255 * 0.8 + 0.15),
          );
        }
      });
    };

    const updateRoads = () => {
      const terrain = refs.current.activeTerrain;
      const snapshot = stateRef.current.snapshot;
      if (!terrain) return;
      const roads = snapshot?.roads ?? [];
      while (refs.current.roads.length < roads.length) {
        const mesh = new THREE.Mesh(
          new THREE.BoxGeometry(1, 1, 1),
          new THREE.MeshStandardMaterial({ roughness: 0.96, metalness: 0.01 }),
        );
        mesh.castShadow = false;
        mesh.receiveShadow = true;
        roadGroup.add(mesh);
        refs.current.roads.push(mesh);
      }
      refs.current.roads.forEach((mesh, index) => {
        const road = roads[index];
        if (!road) {
          mesh.visible = false;
          return;
        }
        mesh.visible = true;
        const segment = roadSegment(terrain, road.from, road.to);
        const [startX, startZ, endX, endZ, length, angle] = segment;
        mesh.position.set((startX + endX) * 0.5, roadHeight(terrain, road.from, road.to), (startZ + endZ) * 0.5);
        mesh.rotation.set(0, angle, 0);
        mesh.scale.set(length, 0.08, road.width);
        mesh.material.color.setHex(roadColor(road.kind));
      });
    };

    const updateTradeRoutes = () => {
      const terrain = refs.current.activeTerrain;
      const snapshot = stateRef.current.snapshot;
      if (!terrain) return;
      const routes = snapshot?.trade_routes ?? [];
      while (refs.current.tradeRoutes.length < routes.length) {
        const geometry = new THREE.BufferGeometry();
        geometry.setFromPoints([new THREE.Vector3(0, 0, 0), new THREE.Vector3(1, 0, 0)]);
        const material = new THREE.LineDashedMaterial({
          color: 0xffffff,
          dashSize: 0.45,
          gapSize: 0.25,
          transparent: true,
          opacity: 0.85,
        });
        const line = new THREE.Line(geometry, material);
        line.computeLineDistances();
        tradeRouteGroup.add(line);
        refs.current.tradeRoutes.push(line);
      }
      refs.current.tradeRoutes.forEach((line, index) => {
        const route = routes[index];
        if (!route) {
          line.visible = false;
          return;
        }
        const from = factionById(snapshot?.factions ?? [], route.from_faction);
        const to = factionById(snapshot?.factions ?? [], route.to_faction);
        if (!from || !to) {
          line.visible = false;
          return;
        }
        line.visible = true;
        const fromX = from.capital[0] * terrain.size - terrain.size / 2;
        const fromZ = from.capital[1] * terrain.size - terrain.size / 2;
        const toX = to.capital[0] * terrain.size - terrain.size / 2;
        const toZ = to.capital[1] * terrain.size - terrain.size / 2;
        const fromY = terrainHeightAt(terrain, from.capital[0], from.capital[1]) + 0.65;
        const toY = terrainHeightAt(terrain, to.capital[0], to.capital[1]) + 0.65;
        const dx = toX - fromX;
        const dy = toY - fromY;
        const dz = toZ - fromZ;
        const length = Math.sqrt(dx * dx + dy * dy + dz * dz) || 1;
        line.position.set((fromX + toX) * 0.5, (fromY + toY) * 0.5, (fromZ + toZ) * 0.5);
        line.scale.set(length, 1, 1);
        line.rotation.set(Math.atan2(dy, Math.sqrt(dx * dx + dz * dz)), Math.atan2(-dz, dx), 0);
        line.material.color.setRGB(from.color[0] / 255, from.color[1] / 255, from.color[2] / 255);
        line.computeLineDistances();
      });
    };

    const applyWeather = () => {
      refs.current.terrainWeather = stateRef.current.snapshot?.weather ?? null;
      refs.current.terrainSeason = refs.current.terrainWeather?.season ?? "";
      const target = stateRef.current.snapshot?.is_day === false ? 0.3 : 1;
      refs.current.targetDayFactor = target;
    };

    refs.current.spawnBurst = (x: number, y: number, color: number, label?: string) => {
      const terrain = refs.current.activeTerrain;
      if (!terrain || !refs.current.effects) return;
      const sprite = createEffectSprite(label ?? "", color);
      sprite.position.set(x * terrain.size - terrain.size / 2, terrainHeightAt(terrain, x, y) + 1.4, y * terrain.size - terrain.size / 2);
      (sprite.userData as { bornAt: number; rise: number; fadeAt: number }).bornAt = performance.now();
      (sprite.userData as { bornAt: number; rise: number; fadeAt: number }).rise = 0.8;
      (sprite.userData as { bornAt: number; rise: number; fadeAt: number }).fadeAt = performance.now() + 3000;
      refs.current.effects.add(sprite);
      refs.current.transientSprites.push(sprite);
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

      // Convert grid cell (0..size) to fixed-point world coords used by the
      // Rust civ-watch endpoints. SpawnCivilianReq uses normalised (0..1)
      // f32 x/y so the server can place the agent against the terrain;
      // PlaceVoxelReq / DamageReq use i64 world coords at FIXED_SCALE (10^6).
      const SCALE = 1_000_000;
      const normX = cellX / terrain.size;
      const normY = cellY / terrain.size;
      const worldX = cellX * SCALE;
      const worldZ = cellY * SCALE;
      const worldY = Math.max(0, Math.round(hit.point.y)) * SCALE;

      if (current.selectedTool === "Camera") return;

      if (current.selectedTool === "InspectAgent") {
        const militaryHit = raycaster.intersectObjects(refs.current.military, false)[0];
        if (militaryHit) {
          const index = refs.current.military.indexOf(militaryHit.object as THREE.Mesh<THREE.ConeGeometry, THREE.MeshStandardMaterial>);
          const unit = stateRef.current.snapshot?.military_units?.[index];
          dispatch({
            type: "set_selected_military",
            military: unit ?? null,
          });
          dispatch({ type: "set_selected_civilian", civilian: null });
          return;
        }
        dispatch({
          type: "set_toast",
          message: `Terrain cell ${basePayload.x}, ${basePayload.y}`,
        });
        return;
      }

      if (current.readOnly) {
        dispatch({
          type: "set_toast",
          message: `Spectator mode — add ?authoring=1 or remove ?spectator=1`,
        });
        return;
      }

      try {
        const message = await executeTerrainAuthoring(
          {
            attachMode: current.attachMode,
            speed: current.speed,
            tool: current.selectedTool,
            cellX: basePayload.x,
            cellY: basePayload.y,
            terrainSize: terrain.size,
            heightY: hit.point.y,
            material: current.selectedMaterial,
            faction: current.selectedFaction,
            damageRadius: current.damageRadius,
            spawnKind: current.spawnKind,
          },
          {
            set_snapshot: (snapshot) =>
              dispatch({ type: "set_snapshot", snapshot: snapshot as Snapshot }),
            set_server_metrics: (metrics) =>
              dispatch({ type: "set_server_metrics", metrics }),
            set_speed: (speed) => dispatch({ type: "set_speed", speed }),
          },
        );
        dispatch({ type: "set_toast", message });
      } catch (err) {
        dispatch({
          type: "set_toast",
          message: err instanceof Error ? err.message : "Authoring failed",
        });
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
    const fog = new THREE.FogExp2(0x87b7e0, 0.0035);
    scene.fog = fog;

    let raf = 0;
    const animate = () => {
      raf = window.requestAnimationFrame(animate);
      const snapshot = refs.current.currentSnapshot;
      const terrain = refs.current.activeTerrain;
      if (terrain) {
        refs.current.dayFactor += (refs.current.targetDayFactor - refs.current.dayFactor) * 0.04;
        const d = refs.current.dayFactor;
        hemisphere.intensity = 0.6 * d;
        sun.intensity = 1.2 * d;
        sun.position.set(terrain.size * 0.7, terrain.size * 1.3, terrain.size * 0.55);
        const weather = refs.current.terrainWeather;
        const bg = new THREE.Color().lerpColors(new THREE.Color(0x0a1530), new THREE.Color(0x87b7e0), d);
        scene.background = bg;
        fog.color.copy(bg);
        fog.density = weather?.precipitation === "rain" ? 0.012 : weather?.precipitation === "snow" ? 0.008 : 0.0035;
        updateWeatherParticles(refs.current, terrain, weather, performance.now());
        animateDecorations(refs.current, terrain, performance.now());
        updateInterpolatedCivilians(refs.current, terrain, performance.now());
        animateTradeRoutes(refs.current, performance.now());
        updateEffects(refs.current, terrain, performance.now());
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
      updateMilitary();
      updateFactions();
      updateBuildings();
      updateRoads();
      updateTradeRoutes();
      applyWeather();
      animate();
    };

    void initialize();

    return () => {
      observer.disconnect();
      renderer.domElement.removeEventListener("pointerdown", onPointerDown);
      window.cancelAnimationFrame(raf);
      controls.dispose();
      terrainGroup.clear();
      territoryGroup.clear();
      roadGroup.clear();
      tradeRouteGroup.clear();
      civilianGroup.clear();
      buildingGroup.clear();
      refs.current.spawnBurst = undefined;
      disposeScene(scene);
      renderer.dispose();
      mount.removeChild(renderer.domElement);
    };

  }, [dispatch]);

  useEffect(() => {
    const terrain = refs.current.activeTerrain;
    if (!terrain) return;
    if (state.snapshot) {
      refs.current.previousSnapshot = refs.current.currentSnapshot;
      refs.current.currentSnapshot = state.snapshot;
      refs.current.snapshotReceivedAt = performance.now();
      const burst = refs.current.spawnBurst;
      if (burst) {
        state.snapshot.birth_events.forEach((event) => burst(event.x, event.y, 0x7ed957));
        state.snapshot.death_events.forEach((event) => burst(event.x, event.y, 0xff6b6b));
        state.snapshot.damage_events.forEach((event) => burst(event.x, event.y, 0xff4d4d, "Impact"));
        state.snapshot.diplomacy_events.forEach((event) => {
          const faction = state.snapshot?.factions.find((item) => item.id === event.faction_a) ?? state.snapshot?.factions[0];
          if (!faction) return;
          burst(
            faction.capital[0],
            faction.capital[1],
            event.kind === "Conflict" ? 0xff4d4d : 0xf5d76e,
            event.kind === "Conflict" ? "Conflict!" : event.kind === "TradeAgreement" ? "Trade Agreement!" : "Peace",
          );
        });
      }
    }
    updateCiviliansFromRefs(refs.current, performance.now());
    updateMilitaryFromRefs(refs.current, state.snapshot);
    updateFactionsFromRefs(refs.current, state.snapshot);
    updateBuildingsFromRefs(refs.current, state.snapshot);
    updateRoadsFromRefs(refs.current, state.snapshot);
    updateTradeRoutesFromRefs(refs.current, state.snapshot);
    refs.current.targetDayFactor = state.snapshot?.is_day === false ? 0.3 : 1;
  }, [state.snapshot]);

  return <div ref={mountRef} className="scene3d" aria-label="Three.js heightmap scene" />;
}

function updateCiviliansFromRefs(refs: SceneRefs, now: number) {
  updateInterpolatedCivilians(refs, refs.activeTerrain, now);
}

function updateMilitaryFromRefs(refs: SceneRefs, snapshot: Snapshot | null) {
  const terrain = refs.activeTerrain;
  if (!terrain) return;
  const units = snapshot?.military_units ?? [];
  refs.military.forEach((mesh, index) => {
    const unit = units[index];
    if (!unit) {
      mesh.visible = false;
      return;
    }
    mesh.visible = true;
    mesh.position.set(
      unit.x * terrain.size - terrain.size / 2,
      terrainHeightAt(terrain, unit.x, unit.y) + 0.85,
      unit.y * terrain.size - terrain.size / 2,
    );
    mesh.material.color.setHex(factionColor(snapshot?.factions ?? [], unit.faction));
    mesh.material.emissiveIntensity = 0;
  });
}

function updateInterpolatedCivilians(refs: SceneRefs, terrain: Terrain | null, now: number) {
  if (!terrain) return;
  const current = refs.currentSnapshot;
  const previous = refs.previousSnapshot ?? current;
  if (!current) return;
  const civs = current.civ_pins ?? [];
  const duration = Math.max(1, current.tick_dt_ms || 100);
  const t = clamp01((now - refs.snapshotReceivedAt) / duration);
  refs.civilians.forEach((mesh, index) => {
    const currentPin = civs[index];
    if (!currentPin) {
      mesh.visible = false;
      return;
    }
    mesh.visible = true;
    const previousPin = previous?.civ_pins?.[index] ?? currentPin;
    const x = THREE.MathUtils.lerp(previousPin.x, currentPin.x, t);
    const y = THREE.MathUtils.lerp(previousPin.y, currentPin.y, t);
    const wx = x * terrain.size - terrain.size / 2;
    const wz = y * terrain.size - terrain.size / 2;
    const wy = terrainHeightAt(terrain, x, y) + 0.7;
    mesh.position.set(wx, wy, wz);
    mesh.material.color.setHex(jobColor(currentPin.job));
  });
}

function updateFactionsFromRefs(refs: SceneRefs, snapshot: Snapshot | null) {
  const terrain = refs.activeTerrain;
  if (!terrain) return;
  const factions = snapshot?.factions ?? [];
  refs.territories.forEach((mesh, index) => {
    const faction = factions[index];
    if (!faction) {
      mesh.visible = false;
      return;
    }
    mesh.visible = true;
    mesh.position.set(
      faction.capital[0] * terrain.size - terrain.size / 2,
      terrainHeightAt(terrain, faction.capital[0], faction.capital[1]) + 0.25,
      faction.capital[1] * terrain.size - terrain.size / 2,
    );
    mesh.scale.setScalar(faction.radius);
    mesh.material.color.setRGB(faction.color[0] / 255, faction.color[1] / 255, faction.color[2] / 255);
  });
}

function updateBuildingsFromRefs(refs: SceneRefs, snapshot: Snapshot | null) {
  const terrain = refs.activeTerrain;
  if (!terrain) return;
  const buildings = snapshot?.buildings ?? [];
  refs.buildings.forEach((mesh, index) => {
    const building = buildings[index];
    if (!building) {
      mesh.visible = false;
      return;
    }
    mesh.visible = true;
    const dims = buildingDimensions(building);
    mesh.scale.set(dims[0], dims[1], dims[2]);
    const wx = building.x * terrain.size - terrain.size / 2;
    const wz = building.y * terrain.size - terrain.size / 2;
    const wy = terrainHeightAt(terrain, building.x, building.y) + dims[1] * 0.5;
    mesh.position.set(wx, wy, wz);
  });
}

function updateRoadsFromRefs(refs: SceneRefs, snapshot: Snapshot | null) {
  const terrain = refs.activeTerrain;
  if (!terrain) return;
  const roads = snapshot?.roads ?? [];
  refs.roads.forEach((mesh, index) => {
    const road = roads[index];
    if (!road) {
      mesh.visible = false;
      return;
    }
    mesh.visible = true;
    const segment = roadSegment(terrain, road.from, road.to);
    const [startX, startZ, endX, endZ, length, angle] = segment;
    mesh.position.set((startX + endX) * 0.5, roadHeight(terrain, road.from, road.to), (startZ + endZ) * 0.5);
    mesh.rotation.set(0, angle, 0);
    mesh.scale.set(length, 0.08, road.width);
    mesh.material.color.setHex(roadColor(road.kind));
  });
}

function updateTradeRoutesFromRefs(refs: SceneRefs, snapshot: Snapshot | null) {
  const terrain = refs.activeTerrain;
  if (!terrain) return;
  const routes = snapshot?.trade_routes ?? [];
  refs.tradeRoutes.forEach((line, index) => {
    const route = routes[index];
    if (!route) {
      line.visible = false;
      return;
    }
    const from = factionById(snapshot?.factions ?? [], route.from_faction);
    const to = factionById(snapshot?.factions ?? [], route.to_faction);
    if (!from || !to) {
      line.visible = false;
      return;
    }
    line.visible = true;
    const fromX = from.capital[0] * terrain.size - terrain.size / 2;
    const fromZ = from.capital[1] * terrain.size - terrain.size / 2;
    const toX = to.capital[0] * terrain.size - terrain.size / 2;
    const toZ = to.capital[1] * terrain.size - terrain.size / 2;
    const fromY = terrainHeightAt(terrain, from.capital[0], from.capital[1]) + 0.65;
    const toY = terrainHeightAt(terrain, to.capital[0], to.capital[1]) + 0.65;
    const dx = toX - fromX;
    const dy = toY - fromY;
    const dz = toZ - fromZ;
    const length = Math.sqrt(dx * dx + dy * dy + dz * dz) || 1;
    line.position.set((fromX + toX) * 0.5, (fromY + toY) * 0.5, (fromZ + toZ) * 0.5);
    line.scale.set(length, 1, 1);
    line.rotation.set(Math.atan2(dy, Math.sqrt(dx * dx + dz * dz)), Math.atan2(-dz, dx), 0);
    line.material.color.setRGB(from.color[0] / 255, from.color[1] / 255, from.color[2] / 255);
    line.computeLineDistances();
  });
}

function animateTradeRoutes(refs: SceneRefs, now: number) {
  const time = now * 0.001;
  refs.tradeRoutes.forEach((line, index) => {
    if (!line.visible) return;
    (line.material as any).dashOffset = -(time * 0.8 + index * 0.15);
  });
}

function updateEffects(refs: SceneRefs, terrain: Terrain, now: number) {
  if (!refs.effects) return;
  const live: THREE.Sprite[] = [];
  refs.transientSprites.forEach((sprite) => {
    const data = sprite.userData as { bornAt: number; fadeAt: number; rise: number };
    const age = now - data.bornAt;
    const life = 3000;
    if (age >= life) {
      refs.effects?.remove(sprite);
      sprite.material.dispose();
      sprite.material.map?.dispose();
      return;
    }
    sprite.position.y += data.rise * 0.01;
    const remaining = 1 - age / life;
    (sprite.material as THREE.SpriteMaterial).opacity = remaining;
    sprite.scale.setScalar(0.8 + remaining * 0.8);
    live.push(sprite);
  });
  refs.transientSprites = live;
}

function createEffectSprite(label: string, color: number) {
  const canvas = document.createElement("canvas");
  canvas.width = 256;
  canvas.height = 128;
  const ctx = canvas.getContext("2d");
  if (ctx) {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = "rgba(10, 14, 20, 0.0)";
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.strokeStyle = `#${color.toString(16).padStart(6, "0")}`;
    ctx.fillStyle = `#${color.toString(16).padStart(6, "0")}`;
    ctx.font = "bold 28px sans-serif";
    ctx.textAlign = "center";
    ctx.fillText(label, canvas.width / 2, canvas.height / 2);
  }
  const texture = new THREE.CanvasTexture(canvas);
  texture.needsUpdate = true;
  const material = new THREE.SpriteMaterial({
    map: texture,
    color: 0xffffff,
    transparent: true,
    opacity: 0.95,
    depthWrite: false,
  });
  return new THREE.Sprite(material);
}

function interpolateCivPin(refs: SceneRefs, index: number, now: number) {
  const current = refs.currentSnapshot;
  if (!current) return { x: 0, y: 0 };
  const currentPin = current.civ_pins[index];
  if (!currentPin) return { x: 0, y: 0 };
  const previous = refs.previousSnapshot ?? current;
  const previousPin = previous.civ_pins[index] ?? currentPin;
  const duration = Math.max(1, current.tick_dt_ms || 100);
  const t = clamp01((now - refs.snapshotReceivedAt) / duration);
  return {
    x: THREE.MathUtils.lerp(previousPin.x, currentPin.x, t),
    y: THREE.MathUtils.lerp(previousPin.y, currentPin.y, t),
  };
}

async function terrainLoader(): Promise<Terrain> {
  const cachedEtag = localStorage.getItem("civis-terrain-etag");
  const headers: HeadersInit = {};
  if (cachedEtag) {
    headers["If-None-Match"] = cachedEtag;
  }
  const response = await fetch("/terrain", { headers });
  if (response.status === 304 && cachedEtag) {
    const cachedBody = localStorage.getItem("civis-terrain-body");
    if (cachedBody) {
      return JSON.parse(cachedBody) as Terrain;
    }
  }
  if (!response.ok) {
    throw new Error(`GET /terrain failed with ${response.status}`);
  }
  const etag = response.headers.get("ETag");
  const body = await response.text();
  if (etag) {
    localStorage.setItem("civis-terrain-etag", etag);
    localStorage.setItem("civis-terrain-body", body);
  }
  return JSON.parse(body) as Terrain;
}

function terrainHeightAt(terrain: Terrain, x: number, y: number) {
  const ix = clampIndex(Math.floor(x), terrain.size);
  const iy = clampIndex(Math.floor(y), terrain.size);
  return terrain.heights[iy * terrain.size + ix] * TERRAIN_HEIGHT_SCALE;
}

function roadHeight(terrain: Terrain, from: [number, number], to: [number, number]) {
  return Math.max(terrainHeightAt(terrain, from[0], from[1]), terrainHeightAt(terrain, to[0], to[1])) + 0.06;
}

function roadSegment(terrain: Terrain, from: [number, number], to: [number, number]) {
  const startX = from[0] * terrain.size - terrain.size / 2;
  const startZ = from[1] * terrain.size - terrain.size / 2;
  const endX = to[0] * terrain.size - terrain.size / 2;
  const endZ = to[1] * terrain.size - terrain.size / 2;
  const dx = endX - startX;
  const dz = endZ - startZ;
  const length = Math.sqrt(dx * dx + dz * dz) || 1;
  const angle = Math.atan2(dz, dx);
  return [startX, startZ, endX, endZ, length, angle] as const;
}

function roadColor(kind: Road["kind"]) {
  switch (kind) {
    case "Trail":
    case "Dirt":
      return 0x8b6a42;
    case "Paved":
      return 0x8d9299;
    case "Highway":
      return 0x45484c;
  }
}

function clampIndex(value: number, size: number) {
  return Math.max(0, Math.min(size - 1, value));
}

function clamp01(value: number) {
  return Math.max(0, Math.min(1, value));
}

function jobColor(job: CivPin["job"]) {
  if (!job) return 0xb7c0cc;
  return JOB_COLORS[job] ?? 0xb7c0cc;
}

function factionById(factions: Faction[], id: number) {
  return factions.find((faction) => faction.id === id) ?? null;
}

function factionColor(factions: Faction[], id: number) {
  const faction = factionById(factions, id);
  if (!faction) return 0xaaaaaa;
  return (faction.color[0] << 16) | (faction.color[1] << 8) | faction.color[2];
}

function buildingDimensions(building: Building): [number, number, number] {
  switch (building.kind) {
    case "Commercial":
      return [1.3, 1.0, 0.9];
    case "Industrial":
      return [1.1, 2.0, 1.1];
    case "Civic":
      return [1.6, 1.9, 1.2];
    case "Residential":
    default:
      return [0.9, 0.8, 0.9];
  }
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

function disposeObject(object: THREE.Object3D) {
  object.traverse((child) => {
    if (child instanceof THREE.Mesh || child instanceof THREE.InstancedMesh || child instanceof THREE.Points) {
      child.geometry.dispose();
      const material = child.material;
      if (Array.isArray(material)) {
        material.forEach((m) => m.dispose());
      } else if (material) {
        material.dispose();
      }
    }
  });
}

function disposeScene(scene: THREE.Scene) {
  scene.traverse((object) => {
    const anyObject = object as THREE.Object3D & {
      geometry?: THREE.BufferGeometry;
      material?: THREE.Material | THREE.Material[];
    };
    if (anyObject.geometry) anyObject.geometry.dispose();
    if (Array.isArray(anyObject.material)) {
      anyObject.material.forEach((material) => material.dispose());
    } else if (anyObject.material) {
      anyObject.material.dispose();
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

function buildDecorations(terrain: Terrain, terrainGroup: THREE.Group, refs: SceneRefs) {
  const rng = createTerrainRng(terrain);
  const treeCandidates: Array<{ x: number; y: number; height: number }> = [];
  const rockCandidates: Array<{ x: number; y: number; height: number }> = [];
  const snowCells: Array<{ x: number; y: number; height: number }> = [];

  for (let y = 0; y < terrain.size; y += 1) {
    for (let x = 0; x < terrain.size; x += 1) {
      const idx = y * terrain.size + x;
      const biome = terrain.biomes[idx];
      const height = terrain.heights[idx] * TERRAIN_HEIGHT_SCALE;
      if (biome === "forest") treeCandidates.push({ x, y, height });
      if (biome === "stone") rockCandidates.push({ x, y, height });
      if (biome === "snow") snowCells.push({ x, y, height });
    }
  }

  const treeCount = Math.min(2000, Math.floor(treeCandidates.length * 0.55));
  const rockCount = Math.min(500, Math.floor(rockCandidates.length * 0.45));
  if (treeCount > 0) {
    const trunkGeo = new THREE.CylinderGeometry(0.08, 0.11, 0.65, 6);
    const canopyGeo = new THREE.ConeGeometry(0.55, 1.4, 7);
    const treeGroup = new THREE.Group();
    const canopyMaterial = new THREE.MeshStandardMaterial({ vertexColors: true, roughness: 1, metalness: 0 });
    const trunkMaterial = new THREE.MeshStandardMaterial({ color: 0x6a4a2f, roughness: 1, metalness: 0 });
    const trunk = new THREE.InstancedMesh(trunkGeo, trunkMaterial, treeCount);
    const canopy = new THREE.InstancedMesh(canopyGeo, canopyMaterial, treeCount);
    trunk.castShadow = true;
    canopy.castShadow = true;
    const trunkMatrix = new THREE.Matrix4();
    const canopyMatrix = new THREE.Matrix4();
    const color = new THREE.Color();
    for (let i = 0; i < treeCount; i += 1) {
      const cell = treeCandidates[Math.floor(rng() * treeCandidates.length)];
      const scale = 0.8 + rng() * 0.4;
      const wx = cell.x - terrain.size / 2 + (rng() - 0.5) * 0.35;
      const wz = cell.y - terrain.size / 2 + (rng() - 0.5) * 0.35;
      const wy = cell.height + 0.08;
      trunkMatrix.compose(
        new THREE.Vector3(wx, wy + 0.3 * scale, wz),
        new THREE.Quaternion().setFromEuler(new THREE.Euler(0, rng() * Math.PI * 2, 0)),
        new THREE.Vector3(scale * 0.8, scale, scale * 0.8),
      );
      canopyMatrix.compose(
        new THREE.Vector3(wx, wy + 1.1 * scale, wz),
        new THREE.Quaternion().setFromEuler(new THREE.Euler(0, rng() * Math.PI * 2, 0)),
        new THREE.Vector3(scale, scale, scale),
      );
      trunk.setMatrixAt(i, trunkMatrix);
      const green = 0x2d5a1e + Math.floor(rng() * (0x4a8c32 - 0x2d5a1e));
      color.setHex(green);
      canopy.setMatrixAt(i, canopyMatrix);
      canopy.setColorAt(i, color);
    }
    canopy.instanceColor!.needsUpdate = true;
    trunk.instanceMatrix.needsUpdate = true;
    canopy.instanceMatrix.needsUpdate = true;
    treeGroup.add(trunk, canopy);
    terrainGroup.add(treeGroup);
    refs.decorationGroup = treeGroup;
    refs.treeInstances = trunk;
  }

  if (rockCount > 0) {
    const rockGeo = new THREE.DodecahedronGeometry(0.5, 0);
    const rockMat = new THREE.MeshStandardMaterial({ vertexColors: true, roughness: 1, metalness: 0.02 });
    const rocks = new THREE.InstancedMesh(rockGeo, rockMat, rockCount);
    rocks.castShadow = true;
    const matrix = new THREE.Matrix4();
    const color = new THREE.Color();
    for (let i = 0; i < rockCount; i += 1) {
      const cell = rockCandidates[Math.floor(rng() * rockCandidates.length)];
      const radius = 0.3 + rng() * 0.5;
      const wx = cell.x - terrain.size / 2 + (rng() - 0.5) * 0.25;
      const wz = cell.y - terrain.size / 2 + (rng() - 0.5) * 0.25;
      const wy = cell.height + radius * 0.5;
      matrix.compose(
        new THREE.Vector3(wx, wy, wz),
        new THREE.Quaternion().setFromEuler(new THREE.Euler(rng() * Math.PI, rng() * Math.PI * 2, rng() * Math.PI)),
        new THREE.Vector3(radius, radius, radius),
      );
      rocks.setMatrixAt(i, matrix);
      const tone = 0x707070 + Math.floor(rng() * (0x909090 - 0x707070));
      color.setHex(tone);
      rocks.setColorAt(i, color);
    }
    rocks.instanceColor!.needsUpdate = true;
    rocks.instanceMatrix.needsUpdate = true;
    terrainGroup.add(rocks);
    refs.rockInstances = rocks;
  }

}

function animateDecorations(refs: SceneRefs, terrain: Terrain, now: number) {
  const time = now * 0.001;
  if (refs.waterMesh) {
    refs.waterMesh.material.opacity = 0.5 + 0.1 * (0.5 + 0.5 * Math.sin(time * 0.8));
  }

  const terrainMesh = refs.terrainMesh;
  const baseColors = refs.terrainBaseColors;
  if (terrainMesh && baseColors) {
    const colorAttr = terrainMesh.geometry.getAttribute("color") as THREE.BufferAttribute;
    const season = refs.terrainSeason || "Summer";
    const seasonBlend = terrainSeasonBlend(season);
    const weather = refs.terrainWeather;
    const positions = terrainMesh.geometry.getAttribute("position") as THREE.BufferAttribute;
    const base = new THREE.Color();
    const tint = new THREE.Color(seasonBlend.tint);
    for (let i = 0; i < colorAttr.count; i += 1) {
      const biome = terrain.biomes[i];
      base.setRGB(baseColors[i * 3], baseColors[i * 3 + 1], baseColors[i * 3 + 2]);
      const x = positions.getX(i);
      const z = positions.getZ(i);
      const sway = 0.04 * Math.sin(time * 1.2 + x * 0.25 + z * 0.17);
      if (biome === "grass" || biome === "forest") {
        const seasonal = base.clone().lerp(tint, seasonBlend.amount);
        if (weather?.precipitation === "snow") {
          seasonal.lerp(new THREE.Color(0xf2f6fb), 0.32);
        }
        seasonal.offsetHSL(0, 0, sway * 0.12);
        colorAttr.setXYZ(i, seasonal.r, seasonal.g, seasonal.b);
      } else {
        colorAttr.setXYZ(
          i,
          clamp01(base.r + sway * 0.05),
          clamp01(base.g + sway * 0.05),
          clamp01(base.b + sway * 0.05),
        );
      }
    }
    colorAttr.needsUpdate = true;
  }
}

function updateWeatherParticles(
  refs: SceneRefs,
  terrain: Terrain,
  weather: Snapshot["weather"] | null,
  now: number,
) {
  if (weather?.precipitation === "snow") {
    if (!refs.snowPoints) {
      refs.snowPoints = createSnowSystem(terrain);
      refs.decorationGroup?.add(refs.snowPoints);
    }
    if (refs.rainPoints) {
      refs.rainPoints.visible = false;
    }
    refs.snowPoints.visible = true;
    animateSnow(refs.snowPoints, terrain);
    return;
  }

  if (weather?.precipitation === "rain") {
    if (!refs.rainPoints) {
      refs.rainPoints = createRainSystem(terrain);
      refs.decorationGroup?.add(refs.rainPoints);
    }
    if (refs.snowPoints) {
      refs.snowPoints.visible = false;
    }
    refs.rainPoints.visible = true;
    animateRain(refs.rainPoints, terrain, now);
    return;
  }

  if (refs.snowPoints) refs.snowPoints.visible = false;
  if (refs.rainPoints) refs.rainPoints.visible = false;
}

function createSnowSystem(terrain: Terrain) {
  const count = Math.min(260, terrain.size * 2);
  const positions = new Float32Array(count * 3);
  const speeds = new Float32Array(count);
  for (let i = 0; i < count; i += 1) {
    positions[i * 3] = (Math.random() - 0.5) * terrain.size;
    positions[i * 3 + 1] = terrain.size * 0.45 + Math.random() * terrain.size * 0.2;
    positions[i * 3 + 2] = (Math.random() - 0.5) * terrain.size;
    speeds[i] = 0.35 + Math.random() * 0.45;
  }
  const geo = new THREE.BufferGeometry();
  geo.setAttribute("position", new THREE.BufferAttribute(positions, 3));
  geo.setAttribute("speed", new THREE.BufferAttribute(speeds, 1));
  return new THREE.Points(geo, new THREE.PointsMaterial({ color: 0xffffff, size: 0.12, transparent: true, opacity: 0.8 }));
}

function createRainSystem(terrain: Terrain) {
  const count = Math.min(500, terrain.size * 4);
  const positions = new Float32Array(count * 3);
  const speeds = new Float32Array(count);
  for (let i = 0; i < count; i += 1) {
    positions[i * 3] = (Math.random() - 0.5) * terrain.size;
    positions[i * 3 + 1] = terrain.size * 0.65 + Math.random() * terrain.size * 0.15;
    positions[i * 3 + 2] = (Math.random() - 0.5) * terrain.size;
    speeds[i] = 1.2 + Math.random() * 0.9;
  }
  const geo = new THREE.BufferGeometry();
  geo.setAttribute("position", new THREE.BufferAttribute(positions, 3));
  geo.setAttribute("speed", new THREE.BufferAttribute(speeds, 1));
  return new THREE.Points(geo, new THREE.PointsMaterial({ color: 0x66aaff, size: 0.05, transparent: true, opacity: 0.75 }));
}

function animateSnow(points: THREE.Points<THREE.BufferGeometry, THREE.PointsMaterial>, terrain: Terrain) {
  const geometry = points.geometry;
  const position = geometry.getAttribute("position") as THREE.BufferAttribute;
  const speed = geometry.getAttribute("speed") as THREE.BufferAttribute | undefined;
  for (let i = 0; i < position.count; i += 1) {
    const vy = position.getY(i) - ((speed?.getX(i) ?? 0.4) * 0.02);
    if (vy < -terrain.size * 0.15) {
      position.setXYZ(i, position.getX(i), terrain.size * 0.6 + (i % 7) * 0.2, position.getZ(i));
    } else {
      position.setY(i, vy);
    }
  }
  position.needsUpdate = true;
}

function animateRain(points: THREE.Points<THREE.BufferGeometry, THREE.PointsMaterial>, terrain: Terrain, now: number) {
  const geometry = points.geometry;
  const position = geometry.getAttribute("position") as THREE.BufferAttribute;
  const speed = geometry.getAttribute("speed") as THREE.BufferAttribute | undefined;
  const wind = 0.03 * Math.sin(now * 0.0015);
  for (let i = 0; i < position.count; i += 1) {
    const vy = position.getY(i) - ((speed?.getX(i) ?? 1.0) * 0.09);
    const vx = position.getX(i) + wind;
    if (vy < -terrain.size * 0.1) {
      position.setXYZ(i, (Math.random() - 0.5) * terrain.size, terrain.size * 0.7 + Math.random() * terrain.size * 0.1, (Math.random() - 0.5) * terrain.size);
    } else {
      position.setXYZ(i, vx, vy, position.getZ(i));
    }
  }
  position.needsUpdate = true;
}

function terrainSeasonBlend(season: string) {
  switch (season) {
    case "Spring":
      return { tint: 0x7dbf63, amount: 0.15 };
    case "Summer":
      return { tint: 0xc9b45b, amount: 0.12 };
    case "Autumn":
      return { tint: 0xb77036, amount: 0.34 };
    case "Winter":
      return { tint: 0xf2f6fb, amount: 0.42 };
    default:
      return { tint: 0x7dbf63, amount: 0.1 };
  }
}

function createTerrainRng(terrain: Terrain) {
  let seed = 2166136261;
  for (let i = 0; i < terrain.heights.length; i += 1) {
    seed = fnv1a(seed, Math.floor(terrain.heights[i] * 1000));
  }
  for (let i = 0; i < terrain.biomes.length; i += 1) {
    seed = fnv1a(seed, biomeId(terrain.biomes[i]));
  }
  return mulberry32(seed >>> 0);
}

function fnv1a(seed: number, value: number) {
  let hash = seed ^ value;
  hash = Math.imul(hash, 16777619);
  return hash >>> 0;
}

function biomeId(biome: Biome) {
  switch (biome) {
    case "deepwater":
      return 1;
    case "water":
      return 2;
    case "sand":
      return 3;
    case "grass":
      return 4;
    case "forest":
      return 5;
    case "stone":
      return 6;
    case "snow":
      return 7;
  }
}

function mulberry32(seed: number) {
  return function rng() {
    let t = (seed += 0x6d2b79f5);
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}
