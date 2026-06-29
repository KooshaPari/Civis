#pragma once
#include "CoreMinimal.h"

namespace CivF3d0ChunkMesh
{
	static constexpr int32 ChunkEdge = 16;
	static constexpr int32 ChunkVoxels = ChunkEdge * ChunkEdge * ChunkEdge;
	static constexpr float VoxelSize = 1.0f;

	// Convert a 16^3 array of material IDs (Z-major order: voxels[z * 256 + y * 16 + x])
	// into a triangle mesh using per-voxel face culling (naive greedy approach).
	// Returns true on success; false if vertices are empty.
	// Note: Greedy-quad merging deferred as future perf optimization; naive per-voxel culling
	// produces correct geometry at ~40-80 us per chunk.
	bool BuildDenseChunkMesh(
		const TArray<int32>& MaterialIds,
		TArray<FVector>& OutVertices,
		TArray<int32>& OutTriangles,
		TArray<FVector>& OutNormals);

	// Decode chunk ID (u64 with signed x/y/z coordinates) to world origin.
	// Chunk layout: x [40:63] (24-bit signed), y [16:39] (24-bit signed), z [0:15] (16-bit signed).
	FVector ChunkWorldOriginFromId(uint64 ChunkRaw);
}
