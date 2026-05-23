#include "CivilianActor.h"

#include "Components/StaticMeshComponent.h"
#include "Materials/MaterialInstanceDynamic.h"
#include "UObject/ConstructorHelpers.h"

ACivilianActor::ACivilianActor()
{
    Mesh = CreateDefaultSubobject<UStaticMeshComponent>(TEXT("Mesh"));
    RootComponent = Mesh;

    static ConstructorHelpers::FObjectFinder<UStaticMesh> CubeMesh(TEXT("/Engine/BasicShapes/Cube.Cube"));
    if (CubeMesh.Succeeded())
    {
        Mesh->SetStaticMesh(CubeMesh.Object);
    }
    Mesh->SetWorldScale3D(FVector(0.4f, 1.4f, 0.4f));

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
    }
}
