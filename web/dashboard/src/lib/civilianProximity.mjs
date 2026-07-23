const DEFAULT_CELL_SIZE = 2;

export function createBuildingProximityIndex(
  terrainSize,
  buildings,
  cellSize = DEFAULT_CELL_SIZE,
) {
  const buckets = new Map();
  const halfTerrain = terrainSize / 2;

  for (const building of buildings) {
    const x = building.x * terrainSize - halfTerrain;
    const z = building.y * terrainSize - halfTerrain;
    const key = bucketKey(Math.floor(x / cellSize), Math.floor(z / cellSize));
    const bucket = buckets.get(key);
    if (bucket) bucket.push({ x, z });
    else buckets.set(key, [{ x, z }]);
  }

  return { buckets, cellSize, radiusSquared: DEFAULT_CELL_SIZE ** 2 };
}

export function isNearBuilding(index, terrainSize, point) {
  const halfTerrain = terrainSize / 2;
  const x = point.x * terrainSize - halfTerrain;
  const z = point.y * terrainSize - halfTerrain;
  const bucketX = Math.floor(x / index.cellSize);
  const bucketZ = Math.floor(z / index.cellSize);

  for (let dx = -1; dx <= 1; dx += 1) {
    for (let dz = -1; dz <= 1; dz += 1) {
      const bucket = index.buckets.get(bucketKey(bucketX + dx, bucketZ + dz));
      if (!bucket) continue;
      for (const building of bucket) {
        const offsetX = x - building.x;
        const offsetZ = z - building.z;
        if (offsetX * offsetX + offsetZ * offsetZ < index.radiusSquared) {
          return true;
        }
      }
    }
  }
  return false;
}

function bucketKey(x, z) {
  return `${x}:${z}`;
}
