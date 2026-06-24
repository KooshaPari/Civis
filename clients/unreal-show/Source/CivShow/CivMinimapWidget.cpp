#include "CivMinimapWidget.h"

#include "Blueprint/WidgetTree.h"
#include "Components/Image.h"
#include "Engine/TextureRenderTarget2D.h"
#include "Input/Events.h"
#include "InputCoreTypes.h"

FVector UCivMinimapWidget::MinimapUvToWorldLocation(const float U, const float V)
{
    // Ortho footprint: centre (CaptureCenterX, CaptureCenterZ), width CaptureOrthoWidth.
    // UV (0,0) = top-left of the 256×256 widget / render target (see minimap-conventions.md).
    const float HalfExtent = CaptureOrthoWidth * 0.5f;
    const float WorldX = CaptureCenterX - HalfExtent + FMath::Clamp(U, 0.0f, 1.0f) * CaptureOrthoWidth;
    const float WorldZ = CaptureCenterZ - HalfExtent + FMath::Clamp(V, 0.0f, 1.0f) * CaptureOrthoWidth;
    return FVector(WorldX, 0.0f, WorldZ);
}

void UCivMinimapWidget::NativeConstruct()
{
    Super::NativeConstruct();
    if (!MinimapImage && WidgetTree)
    {
        MinimapImage = WidgetTree->ConstructWidget<UImage>(UImage::StaticClass(), TEXT("MinimapImage"));
        if (MinimapImage)
        {
            WidgetTree->RootWidget = MinimapImage;
            MinimapImage->SetDesiredSizeOverride(FVector2D(WidgetSize, WidgetSize));
        }
    }

    if (MinimapImage)
    {
        MinimapImage->SetVisibility(ESlateVisibility::Visible);
        MinimapImage->OnMouseButtonDown.BindUObject(this, &UCivMinimapWidget::HandleMinimapPointerEvent);
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
    Brush.ImageSize = FVector2D(WidgetSize, WidgetSize);
    MinimapImage->SetBrush(Brush);
}

FReply UCivMinimapWidget::HandleMinimapPointerEvent(
    const FGeometry MyGeometry,
    const FPointerEvent& MouseEvent)
{
    if (MouseEvent.GetEffectingButton() != EKeys::LeftMouseButton)
    {
        return FReply::Unhandled();
    }

    const FVector2D Local = MyGeometry.AbsoluteToLocal(MouseEvent.GetScreenSpacePosition());
    const FVector2D Size = MyGeometry.GetLocalSize();
    if (Size.X <= KINDA_SMALL_NUMBER || Size.Y <= KINDA_SMALL_NUMBER)
    {
        return FReply::Unhandled();
    }

    const float U = FMath::Clamp(Local.X / Size.X, 0.0f, 1.0f);
    const float V = FMath::Clamp(Local.Y / Size.Y, 0.0f, 1.0f);
    OnMinimapClicked.Broadcast(U, V);
    return FReply::Handled();
}
