#pragma once

#include "CoreMinimal.h"
#include "GameFramework/GameModeBase.h"
#include "CivShowGameMode.generated.h"

class ACivilianActor;
class ACivMinimapCapture;
class AVoxelTerrain;
class UCivMinimapWidget;
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

    UFUNCTION()
    void OnF3d0Frame(const FString& Kind, const FString& FrameJson);

    void SyncCiviliansFromSnapshot(const FString& SnapshotJson);

    void ApplyVoxelDeltaOverlay(const FString& FrameJson);

    void ApplyDayNight(bool bIsDay);

    UPROPERTY()
    UCivProtocolClient* HttpClient = nullptr;

    UPROPERTY()
    UCivWsClient* WsClient = nullptr;

    UPROPERTY()
    AVoxelTerrain* TerrainActor = nullptr;

    UPROPERTY()
    TMap<int32, ACivilianActor*> CivilianActors;

    UPROPERTY()
    TMap<uint64, AActor*> ChunkOverlayActors;

    UPROPERTY()
    ACivMinimapCapture* MinimapCapture = nullptr;

    UPROPERTY()
    UCivMinimapWidget* MinimapWidget = nullptr;

    void SpawnMinimapHud();

    static constexpr int32 MaxChunkOverlays = 64;
    static constexpr float ChunkEdge = 16.0f;
};
