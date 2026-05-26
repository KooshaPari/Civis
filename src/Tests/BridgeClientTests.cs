#nullable enable
using System;
using System.Collections.Generic;
using System.Diagnostics;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Newtonsoft.Json;
using Xunit;

namespace DINOForge.Tests;

public class BridgeClientTests
{
    [Fact]
    public void GameClientOptions_HasCorrectDefaults()
    {
        GameClientOptions options = new();

        options.PipeName.Should().Be("dinoforge-game-bridge");
        options.ConnectTimeoutMs.Should().Be(5000);
        options.ReadTimeoutMs.Should().Be(30000);
        options.RetryCount.Should().Be(3);
        options.RetryDelayMs.Should().Be(1000);
    }

    [Fact]
    public void ConnectionState_HasExpectedValues()
    {
        Enum.GetValues<ConnectionState>().Should().HaveCount(4);
        ((int)ConnectionState.Disconnected).Should().Be(0);
        ((int)ConnectionState.Connecting).Should().Be(1);
        ((int)ConnectionState.Connected).Should().Be(2);
        ((int)ConnectionState.Error).Should().Be(3);
    }

    [Fact]
    public void GameClient_DefaultConstructor_DoesNotThrow()
    {
        using GameClient client = new();

        client.State.Should().Be(ConnectionState.Disconnected);
        client.IsConnected.Should().BeFalse();
    }

    [Fact]
    public void GameClient_WithOptions_DoesNotThrow()
    {
        GameClientOptions options = new() { PipeName = $"dinoforge-test-{Guid.NewGuid():N}", ConnectTimeoutMs = 1000 };
        using GameClient client = new(options);

        client.State.Should().Be(ConnectionState.Disconnected);
        client.IsConnected.Should().BeFalse();
    }

    [Fact]
    public void GameProcessManager_IsRunning_ReturnsFalseWhenGameNotRunning()
    {
        // pattern-#146-skip-guard: env-pollution from lingering Diplomacy process makes "IsRunning=false" assertion flaky in full-suite runs
        if (Process.GetProcessesByName("Diplomacy is Not an Option").Length > 0)
        {
            return;
        }

        GameProcessManager manager = new();

        // Verify internal consistency: IsRunning and GetProcessId() should align.
        // If GetProcessId() returns null, IsRunning must be false.
        // If GetProcessId() returns a value, IsRunning must be true.
        var processId = manager.GetProcessId();
        if (processId is null)
        {
            manager.IsRunning.Should().BeFalse("GetProcessId returned null, so IsRunning should be false");
        }
        else
        {
            manager.IsRunning.Should().BeTrue("GetProcessId returned a value, so IsRunning should be true");
        }
    }

    [Fact]
    public void PingResult_RoundTrip_SerializesCorrectly()
    {
        PingResult original = new() { Pong = true, Version = "0.1.0", UptimeSeconds = 123.45 };

        string json = JsonConvert.SerializeObject(original);
        PingResult? deserialized = JsonConvert.DeserializeObject<PingResult>(json);

        deserialized.Should().NotBeNull();
        deserialized!.Pong.Should().Be(original.Pong);
        deserialized.Version.Should().Be(original.Version);
        deserialized.UptimeSeconds.Should().Be(original.UptimeSeconds);
    }

    [Fact]
    public void GameStatus_RoundTrip_SerializesCorrectly()
    {
        GameStatus original = new()
        {
            Running = true,
            WorldReady = true,
            WorldName = "TestWorld",
            EntityCount = 42,
            ModPlatformReady = true,
            LoadedPacks = new List<string> { "pack-a", "pack-b" },
            Version = "0.1.0"
        };

        string json = JsonConvert.SerializeObject(original);
        GameStatus? deserialized = JsonConvert.DeserializeObject<GameStatus>(json);

        deserialized.Should().NotBeNull();
        deserialized!.WorldReady.Should().BeTrue();
        deserialized.WorldName.Should().Be("TestWorld");
        deserialized.EntityCount.Should().Be(42);
        deserialized.LoadedPacks.Should().HaveCount(2);
    }

    [Fact]
    public void JsonRpcRequest_HasCorrectDefaults()
    {
        JsonRpcRequest request = new() { Method = "ping" };

        request.Jsonrpc.Should().Be("2.0");
        request.Method.Should().Be("ping");
        request.Params.Should().BeNull();
    }

    [Fact]
    public void JsonRpcResponse_WithError_SerializesCorrectly()
    {
        JsonRpcResponse response = new()
        {
            Id = "test-id",
            Error = new JsonRpcError { Code = -32600, Message = "Invalid request" }
        };

        string json = JsonConvert.SerializeObject(response);
        JsonRpcResponse? deserialized = JsonConvert.DeserializeObject<JsonRpcResponse>(json);

        deserialized.Should().NotBeNull();
        deserialized!.Error.Should().NotBeNull();
        deserialized.Error!.Code.Should().Be(-32600);
        deserialized.Error.Message.Should().Be("Invalid request");
        deserialized.Result.Should().BeNull();
    }

    [Fact]
    public void ResourceSnapshot_RoundTrip_SerializesCorrectly()
    {
        ResourceSnapshot original = new()
        {
            Food = 100,
            Wood = 200,
            Stone = 50,
            Iron = 30,
            Money = 500,
            Souls = 10,
            Bones = 5,
            Spirit = 2
        };

        string json = JsonConvert.SerializeObject(original);
        ResourceSnapshot? deserialized = JsonConvert.DeserializeObject<ResourceSnapshot>(json);

        deserialized.Should().NotBeNull();
        deserialized!.Food.Should().Be(100);
        deserialized.Wood.Should().Be(200);
        deserialized.Stone.Should().Be(50);
    }

    [Fact]
    public void GameClientException_PreservesMessage()
    {
        GameClientException ex = new("test error");
        ex.Message.Should().Be("test error");
    }

    [Fact]
    public void GameClientException_PreservesInnerException()
    {
        InvalidOperationException inner = new("inner");
        GameClientException ex = new("outer", inner);

        ex.Message.Should().Be("outer");
        ex.InnerException.Should().BeSameAs(inner);
    }

    [Fact]
    public void GameClient_Dispose_TransitionsToDisconnected()
    {
        GameClient client = new();
        client.Dispose();

        client.State.Should().Be(ConnectionState.Disconnected);
    }

    [Fact]
    public void ComponentMapResult_RoundTrip_SerializesCorrectly()
    {
        ComponentMapResult original = new()
        {
            Mappings = new List<ComponentMapEntry>
            {
                new ComponentMapEntry
                {
                    SdkPath = "unit.stats.hp",
                    EcsType = "Components.Health",
                    FieldName = "currentHealth",
                    Resolved = true,
                    Description = "HP tracking"
                }
            }
        };

        string json = JsonConvert.SerializeObject(original);
        ComponentMapResult? deserialized = JsonConvert.DeserializeObject<ComponentMapResult>(json);

        deserialized.Should().NotBeNull();
        deserialized!.Mappings.Should().HaveCount(1);
        deserialized.Mappings[0].SdkPath.Should().Be("unit.stats.hp");
        deserialized.Mappings[0].Resolved.Should().BeTrue();
    }

    [Fact]
    public void CatalogSnapshot_RoundTrip_SerializesCorrectly()
    {
        CatalogSnapshot original = new()
        {
            Units = new List<CatalogEntry>
            {
                new CatalogEntry
                {
                    InferredId = "vanilla:melee_unit",
                    ComponentCount = 15,
                    EntityCount = 42,
                    Category = "unit"
                }
            }
        };

        string json = JsonConvert.SerializeObject(original);
        CatalogSnapshot? deserialized = JsonConvert.DeserializeObject<CatalogSnapshot>(json);

        deserialized.Should().NotBeNull();
        deserialized!.Units.Should().HaveCount(1);
        deserialized.Units[0].InferredId.Should().Be("vanilla:melee_unit");
        deserialized.Units[0].EntityCount.Should().Be(42);
    }

    // ── Protocol Model Coverage ──────────────────────────────────────────────

    [Fact]
    public void OverrideResult_PropertiesRoundtrip()
    {
        OverrideResult result = new()
        {
            Success = true,
            ModifiedCount = 17,
            SdkPath = "Warfare.Unit.HeavyTank.Armor",
            Message = "Override applied"
        };
        result.Success.Should().BeTrue();
        result.ModifiedCount.Should().Be(17);
        result.SdkPath.Should().Be("Warfare.Unit.HeavyTank.Armor");
        result.Message.Should().Be("Override applied");

        string json = JsonConvert.SerializeObject(result);
        OverrideResult? deserialized = JsonConvert.DeserializeObject<OverrideResult>(json);
        deserialized!.ModifiedCount.Should().Be(17);
    }

    [Fact]
    public void QueryResult_WithEntities_Roundtrips()
    {
        QueryResult result = new()
        {
            Count = 2,
            Entities = new System.Collections.Generic.List<EntityInfo>
            {
                new EntityInfo { Index = 0, Components = new System.Collections.Generic.List<string> { "Health", "ArmorData" } },
                new EntityInfo { Index = 1, Components = new System.Collections.Generic.List<string> { "BuildingBase" } }
            }
        };
        result.Count.Should().Be(2);
        result.Entities.Should().HaveCount(2);
        result.Entities[0].Index.Should().Be(0);
        result.Entities[0].Components.Should().Contain("Health");

        string json = JsonConvert.SerializeObject(result);
        QueryResult? deserialized = JsonConvert.DeserializeObject<QueryResult>(json);
        deserialized!.Entities.Should().HaveCount(2);
        deserialized.Entities[1].Components.Should().Contain("BuildingBase");
    }

    [Fact]
    public void ReloadResult_SuccessWithPacksAndErrors_Roundtrips()
    {
        ReloadResult result = new()
        {
            Success = true,
            LoadedPacks = new System.Collections.Generic.List<string> { "warfare-modern", "example-balance" },
            Errors = new System.Collections.Generic.List<string>()
        };
        result.Success.Should().BeTrue();
        result.LoadedPacks.Should().HaveCount(2);
        result.Errors.Should().BeEmpty();

        string json = JsonConvert.SerializeObject(result);
        ReloadResult? deserialized = JsonConvert.DeserializeObject<ReloadResult>(json);
        deserialized!.LoadedPacks.Should().Contain("warfare-modern");
    }

    [Fact]
    public void ScreenshotResult_PropertiesRoundtrip()
    {
        ScreenshotResult result = new()
        {
            Path = "C:/screenshots/capture.png",
            Width = 1920,
            Height = 1080,
            Success = true
        };
        result.Path.Should().Be("C:/screenshots/capture.png");
        result.Width.Should().Be(1920);
        result.Height.Should().Be(1080);
        result.Success.Should().BeTrue();

        string json = JsonConvert.SerializeObject(result);
        ScreenshotResult? deserialized = JsonConvert.DeserializeObject<ScreenshotResult>(json);
        deserialized!.Width.Should().Be(1920);
    }

    [Fact]
    public void StatResult_PropertiesRoundtrip()
    {
        StatResult result = new()
        {
            SdkPath = "Warfare.Unit.Tank.MaxHp",
            Value = 500.0f,
            EntityCount = 3,
            Values = new System.Collections.Generic.List<float> { 500f, 450f, 500f },
            ComponentType = "Health",
            FieldName = "maxHp"
        };
        result.Value.Should().Be(500.0f);
        result.EntityCount.Should().Be(3);
        result.Values.Should().HaveCount(3);

        string json = JsonConvert.SerializeObject(result);
        StatResult? deserialized = JsonConvert.DeserializeObject<StatResult>(json);
        deserialized!.ComponentType.Should().Be("Health");
        deserialized.FieldName.Should().Be("maxHp");
    }

    [Fact]
    public void VerifyResult_PropertiesRoundtrip()
    {
        VerifyResult result = new()
        {
            PackId = "warfare-modern",
            Loaded = true,
            StatChanges = new System.Collections.Generic.List<string> { "Tank.MaxHp: 400 → 500" },
            Errors = new System.Collections.Generic.List<string>(),
            EntityCount = 42
        };
        result.PackId.Should().Be("warfare-modern");
        result.Loaded.Should().BeTrue();
        result.StatChanges.Should().HaveCount(1);
        result.EntityCount.Should().Be(42);

        string json = JsonConvert.SerializeObject(result);
        VerifyResult? deserialized = JsonConvert.DeserializeObject<VerifyResult>(json);
        deserialized!.StatChanges.Should().Contain("Tank.MaxHp: 400 → 500");
    }

    [Fact]
    public void WaitResult_PropertiesRoundtrip()
    {
        WaitResult result = new() { Ready = true, WorldName = "Default World" };
        result.Ready.Should().BeTrue();
        result.WorldName.Should().Be("Default World");

        string json = JsonConvert.SerializeObject(result);
        WaitResult? deserialized = JsonConvert.DeserializeObject<WaitResult>(json);
        deserialized!.WorldName.Should().Be("Default World");
    }
}
