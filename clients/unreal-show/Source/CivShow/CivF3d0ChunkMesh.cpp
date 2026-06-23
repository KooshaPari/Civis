#include "CivF3d0ChunkMesh.h"

namespace CivF3d0ChunkMesh
{
namespace
{
int32 VoxelIndex(int32 X, int32 Y, int32 Z)
{
    return X + Y * ChunkEdge + Z * ChunkEdge * ChunkEdge;
}

bool IsSolid(const TArray<int32>& Voxels, int32 X, int32 Y, int32 Z)
{
    return Voxels[VoxelIndex(X, Y, Z)] != 0;
}

bool NeighborSolid(const TArray<int32>& Voxels, int32 X, int32 Y, int32 Z)
{
    if (X < 0 || Y < 0 || Z < 0 || X >= ChunkEdge || Y >= ChunkEdge || Z >= ChunkEdge)
    {
        return false;
    }
    return IsSolid(Voxels, X, Y, Z);
}

void PushQuad(
    TArray<FVector>& Vertices,
    TArray<int32>& Triangles,
    TArray<FVector>& Normals,
    const FVector& Base,
    const FVector& U,
    const FVector& V,
    const FVector& Normal)
{
    const int32 BaseIndex = Vertices.Num();
    Vertices.Add(Base);
    Vertices.Add(Base + U);
    Vertices.Add(Base + U + V);
    Vertices.Add(Base + V);
    for (int32 I = 0; I < 4; ++I)
    {
        Normals.Add(Normal);
    }
    Triangles.Append(
        {BaseIndex, BaseIndex + 1, BaseIndex + 2, BaseIndex, BaseIndex + 2, BaseIndex + 3});
}
} // namespace

FVector ChunkWorldOriginFromId(const uint64 ChunkRaw)
{
    int64 Cx = static_cast<int64>((ChunkRaw >> 40) & 0xFFFFFF);
    int64 Cy = static_cast<int64>((ChunkRaw >> 16) & 0xFFFFFF);
    int64 Cz = static_cast<int64>(ChunkRaw & 0xFFFF);
    if (Cx & 0x800000)
    {
        Cx |= ~0xFFFFFFLL;
    }
    if (Cy & 0x800000)
    {
        Cy |= ~0xFFFFFFLL;
    }
    if (Cz & 0x8000)
    {
        Cz |= ~0xFFFFLL;
    }
    const float Edge = static_cast<float>(ChunkEdge);
    return FVector(static_cast<float>(Cx) * Edge, static_cast<float>(Cy) * Edge, static_cast<float>(Cz) * Edge);
}

bool BuildDenseChunkMesh(
    const TArray<int32>& MaterialIds,
    TArray<FVector>& Vertices,
    TArray<int32>& Triangles,
    TArray<FVector>& Normals)
{
    Vertices.Reset();
    Triangles.Reset();
    Normals.Reset();
    if (MaterialIds.Num() != ChunkVoxels)
    {
        return false;
    }

    const float Edge = static_cast<float>(ChunkEdge);
    for (int32 Z = 0; Z < ChunkEdge; ++Z)
    {
        for (int32 Y = 0; Y < ChunkEdge; ++Y)
        {
            for (int32 X = 0; X < ChunkEdge; ++X)
            {
                if (!IsSolid(MaterialIds, X, Y, Z))
                {
                    continue;
                }
                const FVector P(static_cast<float>(X), static_cast<float>(Y), static_cast<float>(Z));
                if (!NeighborSolid(MaterialIds, X - 1, Y, Z))
                {
                    PushQuad(
                        Vertices,
                        Triangles,
                        Normals,
                        P,
                        FVector(0, Edge, 0),
                        FVector(0, 0, Edge),
                        FVector(-1, 0, 0));
                }
                if (!NeighborSolid(MaterialIds, X + 1, Y, Z))
                {
                    PushQuad(
                        Vertices,
                        Triangles,
                        Normals,
                        P + FVector(1, 0, 0),
                        FVector(0, 0, Edge),
                        FVector(0, Edge, 0),
                        FVector(1, 0, 0));
                }
                if (!NeighborSolid(MaterialIds, X, Y - 1, Z))
                {
                    PushQuad(
                        Vertices,
                        Triangles,
                        Normals,
                        P,
                        FVector(Edge, 0, 0),
                        FVector(0, 0, Edge),
                        FVector(0, -1, 0));
                }
                if (!NeighborSolid(MaterialIds, X, Y + 1, Z))
                {
                    PushQuad(
                        Vertices,
                        Triangles,
                        Normals,
                        P + FVector(0, 1, 0),
                        FVector(0, 0, Edge),
                        FVector(Edge, 0, 0),
                        FVector(0, 1, 0));
                }
                if (!NeighborSolid(MaterialIds, X, Y, Z - 1))
                {
                    PushQuad(
                        Vertices,
                        Triangles,
                        Normals,
                        P,
                        FVector(Edge, 0, 0),
                        FVector(0, Edge, 0),
                        FVector(0, 0, -1));
                }
                if (!NeighborSolid(MaterialIds, X, Y, Z + 1))
                {
                    PushQuad(
                        Vertices,
                        Triangles,
                        Normals,
                        P + FVector(0, 0, 1),
                        FVector(0, Edge, 0),
                        FVector(Edge, 0, 0),
                        FVector(0, 0, 1));
                }
            }
        }
    }
    return Triangles.Num() > 0;
}

} // namespace CivF3d0ChunkMesh
