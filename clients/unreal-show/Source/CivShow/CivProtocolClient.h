#pragma once

#include "CoreMinimal.h"
#include "Http.h"
#include "Containers/Ticker.h"
#include "UObject/Object.h"
#include "CivProtocolClient.generated.h"

DECLARE_DYNAMIC_MULTICAST_DELEGATE_OneParam(FOnSnapshotReceived, const FString&, SnapshotJson);
DECLARE_DYNAMIC_MULTICAST_DELEGATE_TwoParams(FOnCivTerrainStatus, const FString&, State, const FString&, Detail);

UCLASS(BlueprintType)
class CIVSHOW_API UCivProtocolClient : public UObject
{
    GENERATED_BODY()

public:
    UFUNCTION(BlueprintCallable, Category = "Civis")
    void Connect(const FString& InBaseUrl = TEXT("http://127.0.0.1:9090"));

    UFUNCTION(BlueprintCallable, Category = "Civis")
    void FetchTerrain();

    UFUNCTION(BlueprintCallable, Category = "Civis")
    void PollSnapshot();

    UFUNCTION(BlueprintCallable, Category = "Civis")
    void PlaceVoxel(int64 X, int64 Y, int64 Z, int32 Material);

    UFUNCTION(BlueprintCallable, Category = "Civis")
    void SpawnCivilian(float X, float Y, int32 Faction);

    /** POST /control/spawn_entity — civilian, vehicle, or airport (FR-CIV-UX-006). */
    UFUNCTION(BlueprintCallable, Category = "Civis")
    void SpawnEntity(const FString& Kind, float X, float Y, int32 Faction);

    /** POST /control/damage — immediate voxel damage (matches civ-watch + civ-server). */
    UFUNCTION(BlueprintCallable, Category = "Civis")
    void ApplyDamage(int64 X, int64 Y, int64 Z, int32 Radius, int32 Energy);

    UPROPERTY(BlueprintAssignable, Category = "Civis")
    FOnSnapshotReceived OnSnapshot;

    UPROPERTY(BlueprintAssignable, Category = "Civis")
    FOnCivTerrainStatus OnTerrainStatus;

    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Civis")
    TArray<float> Heights;

    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Civis")
    TArray<uint8> Biomes;

private:
    bool RequestJson(const FString& Verb, const FString& Path, const FString& Body, TFunction<void(const FString&)> OnOk);
    bool TickSnapshotPoll(float DeltaTime);
    FString BaseUrl = TEXT("http://127.0.0.1:9090");
    FTSTicker::FDelegateHandle SnapshotTickerHandle;
    double LastSnapshotPollSeconds = 0.0;
    bool bSnapshotTickerActive = false;
};
