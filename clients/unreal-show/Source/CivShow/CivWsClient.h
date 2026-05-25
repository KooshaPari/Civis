#pragma once

#include "CoreMinimal.h"
#include "UObject/Object.h"
#include "CivWsClient.generated.h"

class IWebSocket;

DECLARE_DYNAMIC_MULTICAST_DELEGATE_OneParam(FOnCivWsSnapshot, const FString&, SnapshotJson);
DECLARE_DYNAMIC_MULTICAST_DELEGATE_OneParam(FOnCivWsConnection, const FString&, State);

/**
 * JSON-RPC WebSocket client for civ-server (mirrors Godot CivisWsClient).
 * Terrain remains on civ-watch HTTP via UCivProtocolClient.
 */
UCLASS(BlueprintType)
class CIVSHOW_API UCivWsClient : public UObject
{
    GENERATED_BODY()

public:
    UFUNCTION(BlueprintCallable, Category = "Civis")
    void ConnectServer(const FString& WsUrl = TEXT("ws://127.0.0.1:3000/ws?tick_format=binary"));

    UFUNCTION(BlueprintCallable, Category = "Civis")
    void DisconnectServer();

    UFUNCTION(BlueprintCallable, Category = "Civis")
    void Poll();

    UFUNCTION(BlueprintCallable, Category = "Civis")
    void RequestSnapshot();

    UFUNCTION(BlueprintCallable, Category = "Civis")
    void SetSpeed(int32 Multiplier);

    UFUNCTION(BlueprintCallable, Category = "Civis")
    void SpawnEntity(const FString& Kind, float X, float Y, int32 Faction = 0);

    UFUNCTION(BlueprintCallable, Category = "Civis")
    void PlaceVoxel(int64 X, int64 Y, int64 Z, int32 Material);

    UFUNCTION(BlueprintCallable, Category = "Civis")
    void ApplyDamage(int64 X, int64 Y, int64 Z, int32 Radius, int32 Energy = 1000);

    UPROPERTY(BlueprintAssignable, Category = "Civis")
    FOnCivWsSnapshot OnSnapshotReceived;

    UPROPERTY(BlueprintAssignable, Category = "Civis")
    FOnCivWsConnection OnConnectionChanged;

private:
    void SendRpc(const FString& Method, const FString& ParamsJsonObject);
    void HandleMessage(const FString& Text);
    void HandleBinary(const TArray<uint8>& Data);
    void ScheduleReconnect();
    FString NormalizeSnapshot(const TSharedPtr<FJsonObject>& Result) const;

    TSharedPtr<IWebSocket> Socket;
    TMap<int32, FString> PendingMethods;
    int32 RpcId = 1;
    bool bConnected = false;
    bool bReconnectScheduled = false;
    double ReconnectAtSeconds = 0.0;
    double LastSnapshotRequestSeconds = 0.0;
    FString WsUrl;
    static constexpr double ReconnectDelaySec = 3.0;
    static constexpr double SnapshotThrottleSec = 0.25;
};
