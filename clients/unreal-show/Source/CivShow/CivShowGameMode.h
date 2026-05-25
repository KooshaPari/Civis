#pragma once

#include "CoreMinimal.h"
#include "GameFramework/GameModeBase.h"
#include "CivShowGameMode.generated.h"

class ACivilianActor;
class AVoxelTerrain;
class UCivProtocolClient;
class UCivWsClient;

UCLASS()
class CIVSHOW_API ACivShowGameMode : public AGameModeBase
{
    GENERATED_BODY()

public:
    ACivShowGameMode();

    virtual void BeginPlay() override;
    virtual void Tick(float DeltaSeconds) override;

protected:
    UPROPERTY(EditAnywhere, Category = "Civis")
    FString WatchHttpUrl = TEXT("http://127.0.0.1:9090");

    UPROPERTY(EditAnywhere, Category = "Civis")
    FString ServerWsUrl = TEXT("ws://127.0.0.1:3000/ws?tick_format=binary");

    UPROPERTY(EditAnywhere, Category = "Civis")
    TSubclassOf<AVoxelTerrain> TerrainClass;

    UPROPERTY(EditAnywhere, Category = "Civis")
    TSubclassOf<ACivilianActor> CivilianClass;

private:
    UFUNCTION()
    void OnTerrainFetched();

    UFUNCTION()
    void OnWsSnapshot(const FString& SnapshotJson);

    void SyncCiviliansFromSnapshot(const FString& SnapshotJson);

    void ApplyDayNight(bool bIsDay);

    UPROPERTY()
    UCivProtocolClient* HttpClient = nullptr;

    UPROPERTY()
    UCivWsClient* WsClient = nullptr;

    UPROPERTY()
    AVoxelTerrain* TerrainActor = nullptr;

    UPROPERTY()
    TMap<int32, ACivilianActor*> CivilianActors;
};
