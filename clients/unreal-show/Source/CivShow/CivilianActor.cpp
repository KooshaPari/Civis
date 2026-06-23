#include "CivilianActor.h"

#include "Components/StaticMeshComponent.h"
#include "Materials/MaterialInstanceDynamic.h"
#include "UObject/ConstructorHelpers.h"

ACivilianActor::ACivilianActor()
{
    Mesh = CreateDefaultSubobject<UStaticMeshComponent>(TEXT("Mesh"));
    RootComponent = Mesh;

    static ConstructorHelpers::FObjectFinder<UStaticMesh> CylinderMesh(
        TEXT("/Engine/BasicShapes/Cylinder.Cylinder"));
    if (CylinderMesh.Succeeded())
    {
        Mesh->SetStaticMesh(CylinderMesh.Object);
    }
    // Capsule-like proportions (Godot capsule radius 0.22, height 1.05).
    Mesh->SetRelativeScale3D(FVector(0.44f, 1.05f, 0.44f));

    TintMaterial = nullptr;
}

void ACivilianActor::SetJobColor(const FLinearColor& Color)
{
    if (!TintMaterial)
    {
        TintMaterial = Mesh->CreateAndSetMaterialInstanceDynamic(0);
    }
    if (TintMaterial)
    {
        TintMaterial->SetVectorParameterValue(TEXT("Tint"), Color);
        TintMaterial->SetVectorParameterValue(TEXT("BaseColor"), Color);
    }
}
