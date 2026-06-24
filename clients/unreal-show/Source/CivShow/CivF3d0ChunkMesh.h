#pragma once
#include "CoreMinimal.h"
namespace CivF3d0ChunkMesh {
static constexpr int32 ChunkEdge = 16;
static constexpr int32 ChunkVoxels = 4096;
bool BuildDenseChunkMesh(const TArray<int32>& MaterialIds, TArray<FVector>& Vertices, TArray<int32>& Triangles, TArray<FVector>& Normals);
FVector ChunkWorldOriginFromId(uint64 ChunkRaw);
}
