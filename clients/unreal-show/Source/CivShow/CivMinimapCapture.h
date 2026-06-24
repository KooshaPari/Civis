#pragma once

#include "CoreMinimal.h"
#include "GameFramework/Actor.h"
#include "CivMinimapCapture.generated.h"

class USceneCaptureComponent2D;
class UTextureRenderTarget2D;

/// Top-down orthographic capture for the CivShow minimap HUD.
UCLASS()
class CIVSHOW_API ACivMinimapCapture : public AActor
{
    GENERATED_BODY()

public:
    ACivMinimapCapture();

    UPROPERTY(BlueprintReadOnly, Category = "Civis")
    TObjectPtr<UTextureRenderTarget2D> MinimapTexture;

protected:
    virtual void BeginPlay() override;

    UPROPERTY(VisibleAnywhere, Category = "Civis")
    TObjectPtr<USceneCaptureComponent2D> SceneCapture;

    static constexpr int32 TextureSize = 256;
};
