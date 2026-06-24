#include "CivMinimapCapture.h"

#include "Components/SceneCaptureComponent2D.h"
#include "Engine/TextureRenderTarget2D.h"

ACivMinimapCapture::ACivMinimapCapture()
{
    PrimaryActorTick.bCanEverTick = false;

    SceneCapture = CreateDefaultSubobject<USceneCaptureComponent2D>(TEXT("SceneCapture"));
    RootComponent = SceneCapture;
    SceneCapture->ProjectionType = ECameraProjectionMode::Orthographic;
    SceneCapture->OrthoWidth = 512.0f;
    SceneCapture->CaptureSource = SCS_SceneColorHDR;
    SceneCapture->bCaptureEveryFrame = true;
    SceneCapture->bCaptureOnMovement = false;
}

void ACivMinimapCapture::BeginPlay()
{
    Super::BeginPlay();

    MinimapTexture = NewObject<UTextureRenderTarget2D>(this);
    if (!MinimapTexture)
    {
        return;
    }

    MinimapTexture->InitAutoFormat(TextureSize, TextureSize);
    MinimapTexture->UpdateResourceImmediate(true);
    SceneCapture->TextureTarget = MinimapTexture;

    SetActorLocation(FVector(64.0f, 800.0f, 64.0f));
    SetActorRotation(FRotator(-90.0f, 0.0f, 0.0f));
}
