#include "CivMinimapWidget.h"

#include "Blueprint/WidgetTree.h"
#include "Components/Image.h"
#include "Engine/TextureRenderTarget2D.h"

void UCivMinimapWidget::NativeConstruct()
{
    Super::NativeConstruct();
    if (!MinimapImage && WidgetTree)
    {
        MinimapImage = WidgetTree->ConstructWidget<UImage>(UImage::StaticClass(), TEXT("MinimapImage"));
        if (MinimapImage)
        {
            WidgetTree->RootWidget = MinimapImage;
            MinimapImage->SetDesiredSizeOverride(FVector2D(256.0f, 256.0f));
        }
    }
}

void UCivMinimapWidget::SetMinimapTexture(UTextureRenderTarget2D* Texture)
{
    if (!MinimapImage || !Texture)
    {
        return;
    }
    FSlateBrush Brush;
    Brush.SetResourceObject(Texture);
    Brush.ImageSize = FVector2D(256.0f, 256.0f);
    MinimapImage->SetBrush(Brush);
}
