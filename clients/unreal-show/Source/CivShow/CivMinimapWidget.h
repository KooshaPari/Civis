#pragma once

#include "CoreMinimal.h"
#include "Blueprint/UserWidget.h"
#include "CivMinimapWidget.generated.h"

class UImage;
class UTextureRenderTarget2D;

DECLARE_DYNAMIC_MULTICAST_DELEGATE_TwoParams(FOnMinimapUvClicked, float, U, float, V);

UCLASS()
class CIVSHOW_API UCivMinimapWidget : public UUserWidget
{
    GENERATED_BODY()

public:
    virtual void NativeConstruct() override;

    void SetMinimapTexture(UTextureRenderTarget2D* Texture);

    /** Normalised click UV (0–1, top-left origin) → world XZ on the minimap ortho footprint. */
    UFUNCTION(BlueprintPure, Category = "Civis")
    static FVector MinimapUvToWorldLocation(float U, float V);

    UPROPERTY(BlueprintAssignable, Category = "Civis")
    FOnMinimapUvClicked OnMinimapClicked;

private:
    FReply HandleMinimapPointerEvent(FGeometry MyGeometry, const FPointerEvent& MouseEvent);

    UPROPERTY()
    TObjectPtr<UImage> MinimapImage;

    /** Must match `ACivMinimapCapture` (actor location XZ + OrthoWidth). */
    static constexpr float CaptureCenterX = 64.0f;
    static constexpr float CaptureCenterZ = 64.0f;
    static constexpr float CaptureOrthoWidth = 512.0f;
    static constexpr float WidgetSize = 256.0f;
};
