#pragma once

#include "CoreMinimal.h"
#include "GameFramework/Actor.h"
#include "CivilianActor.generated.h"

class UMaterialInstanceDynamic;
class UStaticMeshComponent;

UCLASS()
class CIVSHOW_API ACivilianActor : public AActor
{
    GENERATED_BODY()

public:
    ACivilianActor();

    UFUNCTION(BlueprintCallable, Category = "Civis")
    void SetJobColor(const FLinearColor& Color);

protected:
    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Civis")
    UStaticMeshComponent* Mesh;

    UPROPERTY(Transient)
    UMaterialInstanceDynamic* TintMaterial;
};
