#pragma once

#include "CoreMinimal.h"
#include "Http.h"
#include "Misc/Ticker.h"
#include "UObject/Object.h"
#include "CivProtocolClient.generated.h"

DECLARE_DYNAMIC_MULTICAST_DELEGATE_OneParam(FOnSnapshotReceived, const FString&, SnapshotJson);

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

    UPROPERTY(BlueprintAssignable, Category = "Civis")
    FOnSnapshotReceived OnSnapshot;

    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Civis")
    TArray<float> Heights;

    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Civis")
    TArray<uint8> Biomes;

private:
    bool RequestJson(const FString& Verb, const FString& Path, const FString& Body, TFunction<void(const FString&)> OnOk);
    bool TickSnapshotPoll(float DeltaTime);
    FString BaseUrl = TEXT("http://127.0.0.1:9090");
    FDelegateHandle SnapshotTickerHandle;
    double LastSnapshotPollSeconds = 0.0;
    bool bSnapshotTickerActive = false;
};
