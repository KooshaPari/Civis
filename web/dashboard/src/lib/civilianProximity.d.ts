export type BuildingProximityIndex = {
  buckets: Map<string, Array<{ x: number; z: number }>>;
  cellSize: number;
  radiusSquared: number;
};

export function createBuildingProximityIndex(
  terrainSize: number,
  buildings: Array<{ x: number; y: number }>,
  cellSize?: number,
): BuildingProximityIndex;

export function isNearBuilding(
  index: BuildingProximityIndex,
  terrainSize: number,
  point: { x: number; y: number },
): boolean;
