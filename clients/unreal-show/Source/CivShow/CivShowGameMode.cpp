#include "CivShowGameMode.h"

#include "CivProtocolClient.h"
#include "CivWsClient.h"
#include "CivilianActor.h"
#include "Dom/JsonObject.h"
#include "Components/DirectionalLightComponent.h"
#include "Engine/DirectionalLight.h"
#include "Engine/World.h"
#include "EngineUtils.h"
#include "Serialization/JsonReader.h"
#include "Serialization/JsonSerializer.h"
#include "CivisJobColors.h"
#include "VoxelTerrain.h"

ACivShowGameMode::ACivShowGameMode()
{
    PrimaryActorTick.bCanEverTick = true;
    TerrainClass = AVoxelTerrain::StaticClass();
    CivilianClass = ACivilianActor::StaticClass();
}

void ACivShowGameMode::BeginPlay()
{
    Super::BeginPlay();

    HttpClient = NewObject<UCivProtocolClient>(this);
    WsClient = NewObject<UCivWsClient>(this);

    HttpClient->Connect(WatchHttpUrl);
    HttpClient->FetchTerrain();

    FTimerHandle TerrainTimer;
    GetWorld()->GetTimerManager().SetTimer(
        TerrainTimer,
        this,
        &ACivShowGameMode::OnTerrainFetched,
        0.5f,
        false);

    WsClient->OnSnapshotReceived.AddDynamic(this, &ACivShowGameMode::OnWsSnapshot);
    WsClient->ConnectServer(ServerWsUrl);
}

void ACivShowGameMode::Tick(float DeltaSeconds)
{
    Super::Tick(DeltaSeconds);
    if (WsClient)
    {
        WsClient->Poll();
    }
}

void ACivShowGameMode::OnTerrainFetched()
{
    if (!HttpClient || HttpClient->Heights.Num() == 0)
    {
        return;
    }

    const int32 Size = FMath::RoundToInt(FMath::Sqrt(static_cast<float>(HttpClient->Heights.Num())));
    if (Size <= 0)
    {
        return;
    }

    if (!TerrainActor)
    {
        TerrainActor = GetWorld()->SpawnActor<AVoxelTerrain>(
            TerrainClass,
            FVector::ZeroVector,
            FRotator::ZeroRotator);
    }
    if (TerrainActor)
    {
        TerrainActor->BuildFromHeightmap(HttpClient->Heights, HttpClient->Biomes, Size);
    }
}

void ACivShowGameMode::OnWsSnapshot(const FString& SnapshotJson)
{
    TSharedPtr<FJsonObject> Root;
    const TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(SnapshotJson);
    if (FJsonSerializer::Deserialize(Reader, Root) && Root.IsValid())
    {
        bool bIsDay = true;
        if (Root->TryGetBoolField(TEXT("is_day"), bIsDay))
        {
            ApplyDayNight(bIsDay);
        }
    }
    SyncCiviliansFromSnapshot(SnapshotJson);
}

void ACivShowGameMode::ApplyDayNight(bool bIsDay)
{
    const float Intensity = bIsDay ? 3.0f : 0.85f;
    for (TActorIterator<ADirectionalLight> It(GetWorld()); It; ++It)
    {
        if (UDirectionalLightComponent* Light = It->GetComponent())
        {
            Light->SetIntensity(Intensity);
            break;
        }
    }
}

void ACivShowGameMode::SyncCiviliansFromSnapshot(const FString& SnapshotJson)
{
    TSharedPtr<FJsonObject> Root;
    const TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(SnapshotJson);
    if (!FJsonSerializer::Deserialize(Reader, Root) || !Root.IsValid())
    {
        return;
    }

    const TArray<TSharedPtr<FJsonValue>>* Pins = nullptr;
    if (!Root->TryGetArrayField(TEXT("civ_pins"), Pins) || !Pins)
    {
        return;
    }

    TSet<int32> Seen;
    const float MapSize = 128.0f;
    static constexpr float CivilianFootOffset = 55.0f;

    for (const TSharedPtr<FJsonValue>& Value : *Pins)
    {
        const TSharedPtr<FJsonObject> Pin = Value->AsObject();
        if (!Pin.IsValid())
        {
            continue;
        }

        int32 Idx = 0;
        double X = 0.0;
        double Y = 0.0;
        FString Job = TEXT("unemployed");
        Pin->TryGetNumberField(TEXT("idx"), Idx);
        Pin->TryGetNumberField(TEXT("x"), X);
        Pin->TryGetNumberField(TEXT("y"), Y);
        Pin->TryGetStringField(TEXT("job"), Job);
        Seen.Add(Idx);

        const float NormX = static_cast<float>(X);
        const float NormY = static_cast<float>(Y);
        const float WorldY = TerrainActor
            ? TerrainActor->SampleWorldHeightAtNorm(NormX, NormY, CivilianFootOffset)
            : 12.0f;
        const FVector WorldPos(
            NormX * MapSize,
            WorldY,
            NormY * MapSize);

        const FLinearColor JobColor = FCivisJobColors::FromJobName(Job);

        ACivilianActor* Actor = CivilianActors.FindRef(Idx);
        if (!Actor)
        {
            Actor = GetWorld()->SpawnActor<ACivilianActor>(
                CivilianClass,
                WorldPos,
                FRotator::ZeroRotator);
            if (Actor)
            {
                CivilianActors.Add(Idx, Actor);
                Actor->SetJobColor(JobColor);
            }
        }
        else
        {
            Actor->SetActorLocation(WorldPos);
            Actor->SetJobColor(JobColor);
        }
    }

    TArray<int32> Stale;
    for (const TPair<int32, ACivilianActor*>& Pair : CivilianActors)
    {
        if (!Seen.Contains(Pair.Key) && Pair.Value)
        {
            Pair.Value->Destroy();
            Stale.Add(Pair.Key);
        }
    }
    for (int32 Key : Stale)
    {
        CivilianActors.Remove(Key);
    }
}
