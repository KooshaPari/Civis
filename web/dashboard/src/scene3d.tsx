import React, { useEffect, useRef } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";

type JobLabel = "Farmer" | "Warrior" | "Scholar" | "Trader" | "Priest" | "Admin" | "Unemployed";

type SampleCivilian = {
  age: number;
  health: number;
  ideology: number;
  welfare: number;
  job: JobLabel | null;
};

type Snapshot = {
  tick: number;
  population: number;
  voxel_dirty_count: number;
  voxel_chunk_count: number;
  sample_civilians: SampleCivilian[];
};

const JOB_COLORS: Record<JobLabel, number> = {
  Farmer: 0x53d36b,
  Warrior: 0xff6262,
  Scholar: 0x5db2ff,
  Trader: 0xffd65a,
  Priest: 0xc78bff,
  Admin: 0x8c96a8,
  Unemployed: 0x9fb3d1,
};

export function Scene3d({ snapshot }: { snapshot: Snapshot | null }) {
  const mountRef = useRef<HTMLDivElement | null>(null);
  const sceneRef = useRef<THREE.Scene | null>(null);
  const cameraRef = useRef<THREE.PerspectiveCamera | null>(null);
  const rendererRef = useRef<THREE.WebGLRenderer | null>(null);
  const controlsRef = useRef<OrbitControls | null>(null);
  const frameRef = useRef<number | null>(null);
  const pulseRef = useRef<THREE.Mesh | null>(null);
  const civilianMeshesRef = useRef<THREE.Mesh[]>([]);
  const voxelGroupRef = useRef<THREE.Group | null>(null);

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
          const voxelMaterial = new THREE.MeshStandardMaterial({
            color: 0x2f4d73,
            roughness: 0.85,
            metalness: 0.05,
            transparent: true,
            opacity: 0.88,
          });
          const voxel = new THREE.Mesh(voxelGeometry, voxelMaterial);
          voxel.position.set((x - 3.5) * 0.95, (y - 3.5) * 0.95, (z - 3.5) * 0.95);
          voxel.userData.baseY = voxel.position.y;
          voxelGroup.add(voxel);
        }
      }
    }
    scene.add(voxelGroup);

    const pulseGeometry = new THREE.BoxGeometry(1, 1, 1);
    const pulseMaterial = new THREE.MeshStandardMaterial({
      color: 0x7ee0ff,
      emissive: 0x19465b,
      emissiveIntensity: 1.2,
      roughness: 0.35,
    });
    const pulseCube = new THREE.Mesh(pulseGeometry, pulseMaterial);
    pulseCube.position.set(0, 0, 0);
    scene.add(pulseCube);

    const resize = () => {
      const width = mount.clientWidth;
      const height = mount.clientHeight;
      camera.aspect = width / height;
      camera.updateProjectionMatrix();
      renderer.setSize(width, height, false);
    };

    const observer = new ResizeObserver(resize);
    observer.observe(mount);

    sceneRef.current = scene;
    cameraRef.current = camera;
    rendererRef.current = renderer;
    controlsRef.current = controls;
    voxelGroupRef.current = voxelGroup;
    pulseRef.current = pulseCube;

    const animate = () => {
      frameRef.current = window.requestAnimationFrame(animate);
      controls.update();
      renderer.render(scene, camera);
    };

    resize();
    animate();

    return () => {
      observer.disconnect();
      controls.dispose();
      renderer.dispose();
      voxelGeometry.dispose();
      voxelGroup.children.forEach((child) => {
        const mesh = child as THREE.Mesh;
        (mesh.material as THREE.Material).dispose();
      });
      pulseGeometry.dispose();
      pulseMaterial.dispose();
      mount.removeChild(renderer.domElement);
      if (frameRef.current !== null) {
        window.cancelAnimationFrame(frameRef.current);
      }
      scene.clear();
      sceneRef.current = null;
      cameraRef.current = null;
      rendererRef.current = null;
      controlsRef.current = null;
      voxelGroupRef.current = null;
      pulseRef.current = null;
      civilianMeshesRef.current = [];
    };
  }, []);

  useEffect(() => {
    const pulse = pulseRef.current;
    const voxelGroup = voxelGroupRef.current;
    const scene = sceneRef.current;
    if (!pulse || !voxelGroup || !scene) return;

    const tick = snapshot?.tick ?? 0;
    const cycle = (tick % 24) / 24;
    const pulseScale = 1 + Math.sin(cycle * Math.PI) * 0.5;
    pulse.scale.setScalar(pulseScale);

    voxelGroup.children.forEach((child, index) => {
      const mesh = child as THREE.Mesh;
      const baseY = mesh.userData.baseY as number;
      mesh.position.y = baseY + Math.sin(tick * 0.08 + index * 0.18) * 0.08;
      const brightness = 0.72 + Math.sin(tick * 0.05 + index * 0.11) * 0.12;
      (mesh.material as THREE.MeshStandardMaterial).color.setHSL(0.59, 0.38, brightness);
    });

    updateCivilianMeshes(scene, civilianMeshesRef, snapshot);
  }, [snapshot]);

  return <div ref={mountRef} className="scene3d" aria-label="Three.js voxel scene" />;
}

function updateCivilianMeshes(
  scene: THREE.Scene,
  civilianMeshesRef: React.MutableRefObject<THREE.Mesh[]>,
  snapshot: Snapshot | null,
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
    const geometry = new THREE.SphereGeometry(0.22, 20, 20);
    const material = new THREE.MeshStandardMaterial({ roughness: 0.4, metalness: 0.05 });
    const mesh = new THREE.Mesh(geometry, material);
    mesh.castShadow = false;
    mesh.receiveShadow = false;
    existing.push(mesh);
    scene.add(mesh);
  }

  civilians.forEach((civilian, index) => {
    const mesh = existing[index];
    const position = deriveCivilianPosition(civilian, index, snapshot?.tick ?? 0);
    mesh.position.copy(position);
    (mesh.material as THREE.MeshStandardMaterial).color.setHex(JOB_COLORS[civilian.job ?? "Unemployed"]);
    mesh.scale.setScalar(0.9 + civilian.health * 0.35);
  });
}

function deriveCivilianPosition(civilian: SampleCivilian, index: number, tick: number) {
  const seedA = hashCivilian(civilian.age, civilian.health, civilian.ideology, civilian.welfare, index, tick);
  const seedB = hashCivilian(civilian.age + 11, civilian.health + 17, civilian.ideology + 23, civilian.welfare + 31, index + 7, tick + 13);
  const seedC = hashCivilian(civilian.age + 29, civilian.health + 37, civilian.ideology + 41, civilian.welfare + 43, index + 19, tick + 3);

  const radius = 2.2 + civilian.welfare * 1.8;
  const angle = seedA * Math.PI * 2;
  const height = (seedB - 0.5) * 4.2 + civilian.ideology * 0.8;
  const wobble = Math.sin(tick * 0.04 + seedC * Math.PI * 2) * 0.4;

  return new THREE.Vector3(
    Math.cos(angle) * (radius + wobble),
    height,
    Math.sin(angle) * (radius + wobble),
  );
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
