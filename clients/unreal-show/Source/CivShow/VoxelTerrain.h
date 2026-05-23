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

protected:
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Civis")
    UProceduralMeshComponent* TerrainMesh;
};
