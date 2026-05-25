#include "CivWsClient.h"

#include "Dom/JsonObject.h"
#include "IWebSocket.h"
#include "JsonObjectConverter.h"
#include "Serialization/JsonReader.h"
#include "Serialization/JsonSerializer.h"
#include "WebSocketsModule.h"

void UCivWsClient::ConnectServer(const FString& InWsUrl)
{
    if (!InWsUrl.IsEmpty())
    {
        WsUrl = InWsUrl;
    }
    if (WsUrl.IsEmpty())
    {
        WsUrl = TEXT("ws://127.0.0.1:3000/ws?tick_format=binary");
    }

    DisconnectServer();

    if (!FModuleManager::Get().IsModuleLoaded("WebSockets"))
    {
        FModuleManager::Get().LoadModule("WebSockets");
    }

    Socket = FWebSocketsModule::Get().CreateWebSocket(WsUrl, TEXT("civis-jsonrpc"));
    if (!Socket.IsValid())
    {
        OnConnectionChanged.Broadcast(TEXT("disconnected"));
        ScheduleReconnect();
        return;
    }

    Socket->OnConnected().AddLambda([this]() {
        bConnected = true;
        bReconnectScheduled = false;
        OnConnectionChanged.Broadcast(TEXT("live"));
        SendRpc(TEXT("health"), TEXT("{}"));
        RequestSnapshot();
        SetSpeed(1);
    });

    Socket->OnConnectionError().AddLambda([this](const FString&) {
        bConnected = false;
        OnConnectionChanged.Broadcast(TEXT("disconnected"));
        ScheduleReconnect();
    });

    Socket->OnClosed().AddLambda([this](int32, const FString&, bool) {
        bConnected = false;
        OnConnectionChanged.Broadcast(TEXT("disconnected"));
        ScheduleReconnect();
    });

    Socket->OnMessage().AddLambda([this](const FString& Message) { HandleMessage(Message); });

    Socket->OnRawMessage().AddLambda([this](const void* Data, SIZE_T Size, SIZE_T) {
        TArray<uint8> Bytes;
        Bytes.Append(static_cast<const uint8*>(Data), static_cast<int32>(Size));
        HandleBinary(Bytes);
    });

    OnConnectionChanged.Broadcast(TEXT("reconnecting"));
    Socket->Connect();
}

void UCivWsClient::DisconnectServer()
{
    bReconnectScheduled = false;
    if (Socket.IsValid())
    {
        Socket->Close();
        Socket.Reset();
    }
    PendingMethods.Empty();
    bConnected = false;
    OnConnectionChanged.Broadcast(TEXT("disconnected"));
}

void UCivWsClient::Poll()
{
    if (bReconnectScheduled)
    {
        const double Now = FPlatformTime::Seconds();
        if (Now >= ReconnectAtSeconds)
        {
            bReconnectScheduled = false;
            ConnectServer(WsUrl);
        }
    }
}

void UCivWsClient::RequestSnapshot()
{
    SendRpc(TEXT("sim.snapshot"), TEXT("{}"));
}

void UCivWsClient::SetSpeed(int32 Multiplier)
{
    SendRpc(TEXT("sim.set_speed"), FString::Printf(TEXT("{\"multiplier\":%d}"), Multiplier));
}

void UCivWsClient::SpawnEntity(const FString& Kind, float X, float Y, int32 Faction)
{
    if (Kind.Equals(TEXT("civilian"), ESearchCase::IgnoreCase))
    {
        SendRpc(
            TEXT("sim.spawn_civilian"),
            FString::Printf(TEXT("{\"x\":%f,\"y\":%f,\"faction\":%d}"), X, Y, Faction));
        return;
    }
    SendRpc(
        TEXT("sim.spawn_entity"),
        FString::Printf(
            TEXT("{\"kind\":\"%s\",\"x\":%f,\"y\":%f,\"faction\":%d}"),
            *Kind,
            X,
            Y,
            Faction));
}

void UCivWsClient::PlaceVoxel(int64 X, int64 Y, int64 Z, int32 Material)
{
    SendRpc(
        TEXT("sim.place_voxel"),
        FString::Printf(TEXT("{\"x\":%lld,\"y\":%lld,\"z\":%lld,\"material\":%d}"), X, Y, Z, Material));
}

void UCivWsClient::ApplyDamage(int64 X, int64 Y, int64 Z, int32 Radius, int32 Energy)
{
    SendRpc(
        TEXT("sim.damage"),
        FString::Printf(
            TEXT("{\"x\":%lld,\"y\":%lld,\"z\":%lld,\"radius\":%d,\"energy\":%d}"),
            X,
            Y,
            Z,
            Radius,
            Energy));
}

void UCivWsClient::SendRpc(const FString& Method, const FString& ParamsJsonObject)
{
    if (!Socket.IsValid() || !bConnected)
    {
        return;
    }

    const int32 Id = RpcId++;
    PendingMethods.Add(Id, Method);

    const FString Body = FString::Printf(
        TEXT("{\"jsonrpc\":\"2.0\",\"id\":%d,\"method\":\"%s\",\"params\":%s}"),
        Id,
        *Method,
        *ParamsJsonObject);

    Socket->Send(Body);
}

void UCivWsClient::HandleMessage(const FString& Text)
{
    TSharedPtr<FJsonObject> Root;
    const TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(Text);
    if (!FJsonSerializer::Deserialize(Reader, Root) || !Root.IsValid())
    {
        return;
    }

    if (Root->HasField(TEXT("result")) || Root->HasField(TEXT("error")))
    {
        int32 Id = 0;
        Root->TryGetNumberField(TEXT("id"), Id);
        const FString* Method = PendingMethods.Find(Id);
        if (Method && *Method == TEXT("sim.snapshot"))
        {
            const TSharedPtr<FJsonObject>* ResultObj = nullptr;
            if (Root->TryGetObjectField(TEXT("result"), ResultObj) && ResultObj && ResultObj->IsValid())
            {
                OnSnapshotReceived.Broadcast(NormalizeSnapshot(*ResultObj));
            }
        }
        else if (
            Method
            && (Method->Contains(TEXT("spawn")) || *Method == TEXT("sim.place_voxel")
                || *Method == TEXT("sim.damage")))
        {
            RequestSnapshot();
        }
        PendingMethods.Remove(Id);
        return;
    }

    if (Root->HasField(TEXT("VoxelDelta")) || Root->HasField(TEXT("BuildingDiff"))
        || Root->HasField(TEXT("AgentAppearance")))
    {
        const double Now = FPlatformTime::Seconds();
        if (Now - LastSnapshotRequestSeconds >= SnapshotThrottleSec)
        {
            LastSnapshotRequestSeconds = Now;
            RequestSnapshot();
        }
    }
}

void UCivWsClient::HandleBinary(const TArray<uint8>& Data)
{
    if (Data.Num() >= 4)
    {
        const FString Magic = FString(ANSI_TO_TCHAR(reinterpret_cast<const char*>(Data.GetData())), 4);
        if (Magic == TEXT("F3D0"))
        {
            const double Now = FPlatformTime::Seconds();
            if (Now - LastSnapshotRequestSeconds >= SnapshotThrottleSec)
            {
                LastSnapshotRequestSeconds = Now;
                RequestSnapshot();
            }
        }
    }
}

void UCivWsClient::ScheduleReconnect()
{
    if (bReconnectScheduled)
    {
        return;
    }
    bReconnectScheduled = true;
    ReconnectAtSeconds = FPlatformTime::Seconds() + ReconnectDelaySec;
}

FString UCivWsClient::NormalizeSnapshot(const TSharedPtr<FJsonObject>& Result) const
{
    TSharedPtr<FJsonObject> Out = MakeShared<FJsonObject>();
    double Tick = 0.0;
    double Population = 0.0;
    Result->TryGetNumberField(TEXT("tick"), Tick);
    Result->TryGetNumberField(TEXT("population"), Population);
    Out->SetNumberField(TEXT("tick"), Tick);
    Out->SetNumberField(TEXT("population"), Population);

    const TArray<TSharedPtr<FJsonValue>>* Pins = nullptr;
    if (Result->TryGetArrayField(TEXT("civ_pins"), Pins) && Pins)
    {
        Out->SetArrayField(TEXT("civ_pins"), *Pins);
    }
    const TArray<TSharedPtr<FJsonValue>>* Buildings = nullptr;
    if (Result->TryGetArrayField(TEXT("buildings"), Buildings) && Buildings)
    {
        Out->SetArrayField(TEXT("buildings"), *Buildings);
    }
    const TArray<TSharedPtr<FJsonValue>>* Military = nullptr;
    if (Result->TryGetArrayField(TEXT("military_units"), Military) && Military)
    {
        Out->SetArrayField(TEXT("military_units"), *Military);
    }
    bool bIsDay = true;
    if (Result->TryGetBoolField(TEXT("is_day"), bIsDay))
    {
        Out->SetBoolField(TEXT("is_day"), bIsDay);
    }

    FString Serialized;
    const TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&Serialized);
    FJsonSerializer::Serialize(Out.ToSharedRef(), Writer);
    return Serialized;
}
