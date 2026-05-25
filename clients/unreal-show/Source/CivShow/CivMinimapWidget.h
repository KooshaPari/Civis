#pragma once

#include "CoreMinimal.h"
#include "Blueprint/UserWidget.h"
#include "CivMinimapWidget.generated.h"

class UImage;
class UTextureRenderTarget2D;

UCLASS()
class CIVSHOW_API UCivMinimapWidget : public UUserWidget
{
    GENERATED_BODY()

public:
    virtual void NativeConstruct() override;

    void SetMinimapTexture(UTextureRenderTarget2D* Texture);

private:
    UPROPERTY()
    TObjectPtr<UImage> MinimapImage;
};
