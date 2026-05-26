#pragma once

#include "CoreMinimal.h"
#include "GameFramework/Actor.h"
#include "CivChunkOverlayActor.generated.h"

class UProceduralMeshComponent;

/// F3D0 chunk overlay: procedural mesh when dense voxels are present, else a cube marker.
UCLASS()
class CIVSHOW_API ACivChunkOverlayActor : public AActor
{
    GENERATED_BODY()

public:
    ACivChunkOverlayActor();

    void SetChunkOrigin(const FVector& Origin);

    bool SetDenseVoxels(const TArray<int32>& MaterialIds);

    void SetMarkerFallback();

private:
    UPROPERTY(VisibleAnywhere)
    TObjectPtr<UProceduralMeshComponent> MeshComponent;
};
