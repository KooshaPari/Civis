#include "CivProtocolClient.h"

#include "Dom/JsonObject.h"
#include "Serialization/JsonReader.h"
#include "Serialization/JsonSerializer.h"

void UCivProtocolClient::Connect(const FString& InBaseUrl)
{
    BaseUrl = InBaseUrl;
}

void UCivProtocolClient::FetchTerrain()
{
    RequestJson(TEXT("GET"), TEXT("/terrain"), TEXT(""), [this](const FString& Json)
    {
        TSharedPtr<FJsonObject> Root;
        const TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(Json);
        if (!FJsonSerializer::Deserialize(Reader, Root) || !Root.IsValid())
        {
            return;
        }

        Heights.Reset();
        Biomes.Reset();

        const TArray<TSharedPtr<FJsonValue>>* HeightsJson = nullptr;
        const TArray<TSharedPtr<FJsonValue>>* BiomesJson = nullptr;
        if (Root->TryGetArrayField(TEXT("heights"), HeightsJson))
        {
            for (const TSharedPtr<FJsonValue>& Value : *HeightsJson)
            {
                Heights.Add(static_cast<float>(Value->AsNumber()));
            }
        }
        if (Root->TryGetArrayField(TEXT("biomes"), BiomesJson))
        {
            for (const TSharedPtr<FJsonValue>& Value : *BiomesJson)
            {
                const FString Biome = Value->AsString();
                if (Biome == TEXT("deepwater")) Biomes.Add(0);
                else if (Biome == TEXT("water")) Biomes.Add(1);
                else if (Biome == TEXT("sand")) Biomes.Add(2);
                else if (Biome == TEXT("grass")) Biomes.Add(3);
                else if (Biome == TEXT("forest")) Biomes.Add(4);
                else if (Biome == TEXT("stone")) Biomes.Add(5);
                else Biomes.Add(6);
            }
        }
    });
}

void UCivProtocolClient::PollSnapshot()
{
    if (bSnapshotTickerActive)
    {
        return;
    }

    bSnapshotTickerActive = true;
    LastSnapshotPollSeconds = 0.0;
    SnapshotTickerHandle = FTicker::GetCoreTicker().AddTicker(
        FTickerDelegate::CreateUObject(this, &UCivProtocolClient::TickSnapshotPoll), 0.1f);
}

bool UCivProtocolClient::TickSnapshotPoll(float DeltaTime)
{
    LastSnapshotPollSeconds += DeltaTime;
    RequestJson(TEXT("GET"), TEXT("/snapshot"), TEXT(""), [this](const FString& Json)
    {
        OnSnapshot.Broadcast(Json);
    });
    return true;
}

void UCivProtocolClient::PlaceVoxel(int64 X, int64 Y, int64 Z, int32 Material)
{
    const FString Body = FString::Printf(
        TEXT("{\"x\":%lld,\"y\":%lld,\"z\":%lld,\"material\":%d}"),
        X, Y, Z, Material);
    RequestJson(TEXT("POST"), TEXT("/control/place_voxel"), Body, [](const FString&) {});
}

void UCivProtocolClient::SpawnCivilian(float X, float Y, int32 Faction)
{
    const FString Body = FString::Printf(
        TEXT("{\"x\":%f,\"y\":%f,\"faction\":%d}"),
        X, Y, Faction);
    RequestJson(TEXT("POST"), TEXT("/control/spawn_civilian"), Body, [](const FString&) {});
}

bool UCivProtocolClient::RequestJson(const FString& Verb, const FString& Path, const FString& Body, TFunction<void(const FString&)> OnOk)
{
    const FString Url = BaseUrl + Path;
    TSharedRef<IHttpRequest, ESPMode::ThreadSafe> Request = FHttpModule::Get().CreateRequest();
    Request->SetURL(Url);
    Request->SetVerb(Verb);
    Request->SetHeader(TEXT("Content-Type"), TEXT("application/json"));
    if (!Body.IsEmpty())
    {
        Request->SetContentAsString(Body);
    }

    Request->OnProcessRequestComplete().BindLambda(
        [OnOk](FHttpRequestPtr, FHttpResponsePtr Response, bool bSucceeded)
        {
            if (!bSucceeded || !Response.IsValid())
            {
                return;
            }
            OnOk(Response->GetContentAsString());
        });

    return Request->ProcessRequest();
}
