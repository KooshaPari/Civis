#include "VoxelTerrain.h"

#include "ProceduralMeshComponent.h"

AVoxelTerrain::AVoxelTerrain()
{
    TerrainMesh = CreateDefaultSubobject<UProceduralMeshComponent>(TEXT("TerrainMesh"));
    RootComponent = TerrainMesh;
}

static FColor BiomeColor(uint8 Biome)
{
    switch (Biome)
    {
    case 0: return FColor(0x0e, 0x26, 0x59);
    case 1: return FColor(0x2c, 0x64, 0xa8);
    case 2: return FColor(0xde, 0xc8, 0x84);
    case 3: return FColor(0x68, 0x9a, 0x3c);
    case 4: return FColor(0x2c, 0x64, 0x34);
    case 5: return FColor(0x80, 0x7c, 0x74);
    default: return FColor(0xf0, 0xf0, 0xf0);
    }
}

void AVoxelTerrain::BuildFromHeightmap(const TArray<float>& Heights, const TArray<uint8>& Biomes, int32 Size)
{
    TArray<FVector> Vertices;
    TArray<int32> Triangles;
    TArray<FVector> Normals;
    TArray<FVector2D> UV0;
    TArray<FProcMeshTangent> Tangents;
    TArray<FLinearColor> Colors;

    Vertices.Reserve((Size - 1) * (Size - 1) * 4);
    Triangles.Reserve((Size - 1) * (Size - 1) * 6);

    for (int32 Y = 0; Y < Size - 1; ++Y)
    {
        for (int32 X = 0; X < Size - 1; ++X)
        {
            const int32 Idx00 = Y * Size + X;
            const int32 Idx10 = Y * Size + (X + 1);
            const int32 Idx01 = (Y + 1) * Size + X;
            const int32 Idx11 = (Y + 1) * Size + (X + 1);

            const FVector A(X, Y, Heights[Idx00] * 100.0f);
            const FVector B(X + 1, Y, Heights[Idx10] * 100.0f);
            const FVector C(X, Y + 1, Heights[Idx01] * 100.0f);
            const FVector D(X + 1, Y + 1, Heights[Idx11] * 100.0f);

            const int32 Base = Vertices.Num();
            Vertices.Add(A);
            Vertices.Add(B);
            Vertices.Add(C);
            Vertices.Add(D);

            Triangles.Append({Base, Base + 2, Base + 1, Base + 1, Base + 2, Base + 3});

            const FColor CellColor = BiomeColor(Biomes.IsValidIndex(Idx00) ? Biomes[Idx00] : 6);
            Colors.Add(CellColor);
            Colors.Add(CellColor);
            Colors.Add(CellColor);
            Colors.Add(CellColor);

            Normals.Add(FVector::UpVector);
            Normals.Add(FVector::UpVector);
            Normals.Add(FVector::UpVector);
            Normals.Add(FVector::UpVector);

            UV0.Add(FVector2D(0, 0));
            UV0.Add(FVector2D(1, 0));
            UV0.Add(FVector2D(0, 1));
            UV0.Add(FVector2D(1, 1));

            Tangents.Add(FProcMeshTangent(1, 0, 0));
            Tangents.Add(FProcMeshTangent(1, 0, 0));
            Tangents.Add(FProcMeshTangent(1, 0, 0));
            Tangents.Add(FProcMeshTangent(1, 0, 0));
        }
    }

    TerrainMesh->CreateMeshSection_LinearColor(0, Vertices, Triangles, Normals, UV0, Colors, Tangents, true);
}
