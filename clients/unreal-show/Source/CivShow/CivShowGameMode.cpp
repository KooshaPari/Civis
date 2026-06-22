#include "CivShowGameMode.h"

#include "CivMinimapCapture.h"
#include "CivMinimapWidget.h"
#include "CivProtocolClient.h"
#include "CivWsClient.h"
#include "CivilianActor.h"
#include "Blueprint/UserWidget.h"
#include "Dom/JsonObject.h"
#include "Components/DirectionalLightComponent.h"
#include "Engine/DirectionalLight.h"
#include "Engine/World.h"
#include "EngineUtils.h"
#include "Serialization/JsonReader.h"
#include "Serialization/JsonSerializer.h"
#include "CivisJobColors.h"
#include "VoxelTerrain.h"
#include "Components/StaticMeshComponent.h"
#include "Engine/StaticMeshActor.h"

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
    WsClient->OnF3d0FrameReceived.AddDynamic(this, &ACivShowGameMode::OnF3d0Frame);
    WsClient->ConnectServer(ServerWsUrl);

    SpawnMinimapHud();
}

void ACivShowGameMode::SpawnMinimapHud()
{
    if (!GetWorld())
    {
        return;
    }

    MinimapCapture = GetWorld()->SpawnActor<ACivMinimapCapture>(
        ACivMinimapCapture::StaticClass(),
        FVector::ZeroVector,
        FRotator::ZeroRotator);
    if (!MinimapCapture)
    {
        return;
    }

    if (APlayerController* Pc = GetWorld()->GetFirstPlayerController())
    {
        MinimapWidget = CreateWidget<UCivMinimapWidget>(Pc, UCivMinimapWidget::StaticClass());
        if (MinimapWidget)
        {
            MinimapWidget->AddToViewport(1);
            if (MinimapCapture->MinimapTexture)
            {
                MinimapWidget->SetMinimapTexture(MinimapCapture->MinimapTexture);
            }
        }
    }
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

void ACivShowGameMode::OnF3d0Frame(const FString& Kind, const FString& FrameJson)
{
    if (Kind == TEXT("VoxelDelta"))
    {
        ApplyVoxelDeltaOverlay(FrameJson);
    }
}

static FVector ChunkWorldCentreFromId(const uint64 ChunkRaw)
{
    static constexpr float ChunkEdge = 16.0f;
    int64 Cx = static_cast<int64>((ChunkRaw >> 40) & 0xFFFFFF);
    int64 Cy = static_cast<int64>((ChunkRaw >> 16) & 0xFFFFFF);
    int64 Cz = static_cast<int64>(ChunkRaw & 0xFFFF);
    if (Cx & 0x800000)
    {
        Cx |= ~0xFFFFFFLL;
    }
    if (Cy & 0x800000)
    {
        Cy |= ~0xFFFFFFLL;
    }
    if (Cz & 0x8000)
    {
        Cz |= ~0xFFFFLL;
    }
    return FVector(
        (static_cast<float>(Cx) + 0.5f) * ChunkEdge,
        (static_cast<float>(Cy) + 0.5f) * ChunkEdge * 0.2f,
        (static_cast<float>(Cz) + 0.5f) * ChunkEdge);
}

void ACivShowGameMode::ApplyVoxelDeltaOverlay(const FString& FrameJson)
{
    TSharedPtr<FJsonObject> Root;
    const TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(FrameJson);
    if (!FJsonSerializer::Deserialize(Reader, Root) || !Root.IsValid())
    {
        return;
    }

    const TSharedPtr<FJsonObject>* VoxelObj = nullptr;
    if (!Root->TryGetObjectField(TEXT("VoxelDelta"), VoxelObj) || !VoxelObj || !VoxelObj->IsValid())
    {
        return;
    }

    const TArray<TSharedPtr<FJsonValue>>* Deltas = nullptr;
    if (!(*VoxelObj)->TryGetArrayField(TEXT("deltas"), Deltas) || !Deltas)
    {
        return;
    }

    int32 Shown = 0;
    for (const TSharedPtr<FJsonValue>& DeltaVal : *Deltas)
    {
        if (Shown >= MaxChunkOverlays)
        {
            break;
        }
        const TSharedPtr<FJsonObject> Delta = DeltaVal->AsObject();
        if (!Delta.IsValid())
        {
            continue;
        }
        const TSharedPtr<FJsonObject>* EventObj = nullptr;
        if (!Delta->TryGetObjectField(TEXT("event"), EventObj) || !EventObj || !EventObj->IsValid())
        {
            continue;
        }
        double ChunkIdRaw = 0.0;
        if (!(*EventObj)->TryGetNumberField(TEXT("chunk_id"), ChunkIdRaw) || ChunkIdRaw == 0.0)
        {
            continue;
        }
        const uint64 ChunkKey = static_cast<uint64>(ChunkIdRaw);

        AActor* Marker = ChunkOverlayActors.FindRef(ChunkKey);
        if (!Marker)
        {
            Marker = GetWorld()->SpawnActor<AStaticMeshActor>(
                AStaticMeshActor::StaticClass(),
                ChunkWorldCentreFromId(ChunkKey),
                FRotator::ZeroRotator);
            if (!Marker)
            {
                continue;
            }
            if (AStaticMeshActor* MeshActor = Cast<AStaticMeshActor>(Marker))
            {
                if (UStaticMesh* Cube = LoadObject<UStaticMesh>(
                        nullptr,
                        TEXT("/Engine/BasicShapes/Cube.Cube")))
                {
                    MeshActor->GetStaticMeshComponent()->SetStaticMesh(Cube);
                    MeshActor->GetStaticMeshComponent()->SetWorldScale3D(
                        FVector(ChunkEdge * 0.9f, ChunkEdge * 0.35f, ChunkEdge * 0.9f));
                }
            }
            ChunkOverlayActors.Add(ChunkKey, Marker);
        }
        else
        {
            Marker->SetActorLocation(ChunkWorldCentreFromId(ChunkKey));
        }
        ++Shown;
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
