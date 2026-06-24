#include "CivChunkOverlayActor.h"

#include "CivF3d0ChunkMesh.h"
#include "ProceduralMeshComponent.h"
#include "Materials/MaterialInterface.h"

ACivChunkOverlayActor::ACivChunkOverlayActor()
{
    PrimaryActorTick.bCanEverTick = false;
    MeshComponent = CreateDefaultSubobject<UProceduralMeshComponent>(TEXT("ChunkMesh"));
    RootComponent = MeshComponent;
}

void ACivChunkOverlayActor::SetChunkOrigin(const FVector& Origin)
{
    SetActorLocation(Origin);
}

bool ACivChunkOverlayActor::SetDenseVoxels(const TArray<int32>& MaterialIds)
{
    TArray<FVector> Vertices;
    TArray<int32> Triangles;
    TArray<FVector> Normals;
    if (!CivF3d0ChunkMesh::BuildDenseChunkMesh(MaterialIds, Vertices, Triangles, Normals))
    {
        return false;
    }

    TArray<FVector2D> UV0;
    TArray<FProcMeshTangent> Tangents;
    UV0.SetNum(Vertices.Num());
    Tangents.SetNum(Vertices.Num());
    for (int32 I = 0; I < Vertices.Num(); ++I)
    {
        UV0[I] = FVector2D::ZeroVector;
        Tangents[I] = FProcMeshTangent(1.0f, 0.0f, 0.0f);
    }

    MeshComponent->ClearAllMeshSections();
    MeshComponent->CreateMeshSection(
        0,
        Vertices,
        Triangles,
        Normals,
        UV0,
        TArray<FColor>(),
        Tangents,
        true);
    return true;
}

void ACivChunkOverlayActor::SetMarkerFallback()
{
    const float Edge = static_cast<float>(CivF3d0ChunkMesh::ChunkEdge);
    const float Half = Edge * 0.5f;
    const FVector O(Half, Edge * 0.2f, Half);
    const FVector A = O + FVector(-Half * 0.9f, -Half * 0.35f, -Half * 0.9f);
    const FVector B = O + FVector(Half * 0.9f, -Half * 0.35f, -Half * 0.9f);
    const FVector C = O + FVector(Half * 0.9f, Half * 0.35f, -Half * 0.9f);
    const FVector D = O + FVector(-Half * 0.9f, Half * 0.35f, -Half * 0.9f);
    const FVector E = O + FVector(-Half * 0.9f, -Half * 0.35f, Half * 0.9f);
    const FVector F = O + FVector(Half * 0.9f, -Half * 0.35f, Half * 0.9f);
    const FVector G = O + FVector(Half * 0.9f, Half * 0.35f, Half * 0.9f);
    const FVector H = O + FVector(-Half * 0.9f, Half * 0.35f, Half * 0.9f);

    TArray<FVector> Vertices = {A, B, C, D, E, F, G, H};
    TArray<int32> Triangles = {
        0, 1, 2, 0, 2, 3, // bottom
        4, 6, 5, 4, 7, 6, // top
        0, 4, 5, 0, 5, 1,
        1, 5, 6, 1, 6, 2,
        2, 6, 7, 2, 7, 3,
        3, 7, 4, 3, 4, 0,
    };
    TArray<FVector> Normals;
    Normals.Init(FVector::UpVector, Vertices.Num());
    TArray<FVector2D> UV0;
    UV0.Init(FVector2D::ZeroVector, Vertices.Num());
    TArray<FProcMeshTangent> Tangents;
    Tangents.Init(FProcMeshTangent(1, 0, 0), Vertices.Num());

    MeshComponent->ClearAllMeshSections();
    MeshComponent->CreateMeshSection(
        0,
        Vertices,
        Triangles,
        Normals,
        UV0,
        TArray<FColor>(),
        Tangents,
        true);
}
