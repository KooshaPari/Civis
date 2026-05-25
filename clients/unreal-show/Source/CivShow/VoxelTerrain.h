#pragma once

#include "CoreMinimal.h"
#include "GameFramework/Actor.h"
#include "VoxelTerrain.generated.h"

class UProceduralMeshComponent;

UCLASS()
class CIVSHOW_API AVoxelTerrain : public AActor
{
    GENERATED_BODY()

public:
    AVoxelTerrain();

    UFUNCTION(BlueprintCallable, Category = "Civis")
    void BuildFromHeightmap(const TArray<float>& Heights, const TArray<uint8>& Biomes, int32 Size);

    /** World-space Y (up) at normalized map coords; includes optional foot offset. */
    UFUNCTION(BlueprintCallable, Category = "Civis")
    float SampleWorldHeightAtNorm(float NormX, float NormY, float FootOffset = 0.0f) const;

    int32 GetGridSize() const { return GridSize; }

protected:
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Civis")
    UProceduralMeshComponent* TerrainMesh;

    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Civis")
    float HeightWorldScale = 100.0f;

    TArray<float> CachedHeights;
    int32 GridSize = 0;
};
