#nullable enable
using System.Collections.Generic;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Newtonsoft.Json;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// Verifies JSON serialization round-trips for every Protocol message type.
/// Covers the full IPC contract so broken property names or serialiser defaults
/// are caught before hitting the bridge.
/// </summary>
public class ProtocolSerializationTests
{
    private static T RoundTrip<T>(T obj)
    {
        string json = JsonConvert.SerializeObject(obj);
        return JsonConvert.DeserializeObject<T>(json)!;
    }

    // ─── PingResult ───────────────────────────────────────────────────────────

    [Fact]
    public void PingResult_Defaults_RoundTrip()
    {
        PingResult r = RoundTrip(new PingResult());
        r.Pong.Should().BeFalse();
        r.Version.Should().BeEmpty();
        r.UptimeSeconds.Should().Be(0);
    }

    [Fact]
    public void PingResult_Populated_RoundTrip()
    {
        PingResult r = RoundTrip(new PingResult { Pong = true, Version = "0.9.0", UptimeSeconds = 123.4 });
        r.Pong.Should().BeTrue();
        r.Version.Should().Be("0.9.0");
        r.UptimeSeconds.Should().BeApproximately(123.4, 0.01);
    }

    // ─── GameStatus ───────────────────────────────────────────────────────────

    [Fact]
    public void GameStatus_Defaults_RoundTrip()
    {
        GameStatus s = RoundTrip(new GameStatus());
        s.Running.Should().BeFalse();
        s.WorldReady.Should().BeFalse();
        s.EntityCount.Should().Be(0);
        s.LoadedPacks.Should().BeEmpty();
    }

    [Fact]
    public void GameStatus_Populated_RoundTrip()
    {
        GameStatus s = RoundTrip(new GameStatus
        {
            Running = true,
            WorldReady = true,
            WorldName = "Default",
            EntityCount = 45776,
            ModPlatformReady = true,
            LoadedPacks = new List<string> { "warfare-starwars", "example-balance" },
            Version = "0.7.0"
        });
        s.Running.Should().BeTrue();
        s.EntityCount.Should().Be(45776);
        s.LoadedPacks.Should().HaveCount(2).And.Contain("warfare-starwars");
    }

    // ─── QueryResult / EntityInfo ─────────────────────────────────────────────

    [Fact]
    public void QueryResult_Defaults_RoundTrip()
    {
        QueryResult r = RoundTrip(new QueryResult());
        r.Count.Should().Be(0);
        r.Entities.Should().BeEmpty();
    }

    [Fact]
    public void QueryResult_WithEntities_RoundTrip()
    {
        QueryResult r = RoundTrip(new QueryResult
        {
            Count = 2,
            Entities = new List<EntityInfo>
            {
                new EntityInfo { Index = 1, Components = new List<string> { "Health", "Unit" } },
                new EntityInfo { Index = 2, Components = new List<string> { "Health" } }
            }
        });
        r.Count.Should().Be(2);
        r.Entities[0].Components.Should().Contain("Health");
    }

    // ─── StatResult ──────────────────────────────────────────────────────────

    [Fact]
    public void StatResult_Populated_RoundTrip()
    {
        StatResult r = RoundTrip(new StatResult
        {
            SdkPath = "units.rep_clone_trooper.hp",
            Value = 150f,
            EntityCount = 10,
            Values = new List<float> { 150f, 150f },
            ComponentType = "Components.Health",
            FieldName = "hp"
        });
        r.Value.Should().BeApproximately(150f, 0.001f);
        r.EntityCount.Should().Be(10);
        r.FieldName.Should().Be("hp");
    }

    // ─── OverrideResult ───────────────────────────────────────────────────────

    [Fact]
    public void OverrideResult_Success_RoundTrip()
    {
        OverrideResult r = RoundTrip(new OverrideResult
        {
            Success = true,
            ModifiedCount = 5,
            SdkPath = "units.rep_clone_trooper.hp",
            Message = "Applied"
        });
        r.Success.Should().BeTrue();
        r.ModifiedCount.Should().Be(5);
    }

    // ─── ReloadResult ─────────────────────────────────────────────────────────

    [Fact]
    public void ReloadResult_Success_RoundTrip()
    {
        ReloadResult r = RoundTrip(new ReloadResult
        {
            Success = true,
            LoadedPacks = new List<string> { "warfare-starwars" },
            Errors = new List<string>()
        });
        r.Success.Should().BeTrue();
        r.Errors.Should().BeEmpty();
        r.LoadedPacks.Should().Contain("warfare-starwars");
    }

    [Fact]
    public void ReloadResult_WithErrors_RoundTrip()
    {
        ReloadResult r = RoundTrip(new ReloadResult
        {
            Success = false,
            LoadedPacks = new List<string>(),
            Errors = new List<string> { "Missing pack.yaml", "Unknown faction ref" }
        });
        r.Success.Should().BeFalse();
        r.Errors.Should().HaveCount(2);
    }

    // ─── CatalogSnapshot / CatalogEntry ───────────────────────────────────────

    [Fact]
    public void CatalogSnapshot_Populated_RoundTrip()
    {
        CatalogSnapshot snap = RoundTrip(new CatalogSnapshot
        {
            Units = new List<CatalogEntry>
            {
                new CatalogEntry { InferredId = "vanilla:melee_unit", ComponentCount = 8, EntityCount = 12, Category = "unit" }
            },
            Buildings = new List<CatalogEntry>
            {
                new CatalogEntry { InferredId = "vanilla:farm", ComponentCount = 5, EntityCount = 3, Category = "building" }
            }
        });
        snap.Units.Should().HaveCount(1);
        snap.Units[0].EntityCount.Should().Be(12);
        snap.Buildings[0].InferredId.Should().Be("vanilla:farm");
        snap.Projectiles.Should().BeEmpty();
        snap.Other.Should().BeEmpty();
    }

    // ─── ResourceSnapshot ─────────────────────────────────────────────────────

    [Fact]
    public void ResourceSnapshot_Defaults_RoundTrip()
    {
        ResourceSnapshot r = RoundTrip(new ResourceSnapshot());
        r.Food.Should().Be(0); r.Wood.Should().Be(0); r.Stone.Should().Be(0);
        r.Iron.Should().Be(0); r.Money.Should().Be(0); r.Souls.Should().Be(0);
    }

    [Fact]
    public void ResourceSnapshot_Populated_RoundTrip()
    {
        ResourceSnapshot r = RoundTrip(new ResourceSnapshot
        { Food = 100, Wood = 200, Stone = 50, Iron = 25, Money = 10, Souls = 5, Bones = 3, Spirit = 1 });
        r.Food.Should().Be(100);
        r.Money.Should().Be(10);
        r.Spirit.Should().Be(1);
    }

    // ─── ScreenshotResult ─────────────────────────────────────────────────────

    [Fact]
    public void ScreenshotResult_RoundTrip()
    {
        ScreenshotResult r = RoundTrip(new ScreenshotResult
        { Path = "C:/tmp/shot.png", Width = 1920, Height = 1080, Success = true });
        r.Path.Should().Be("C:/tmp/shot.png");
        r.Width.Should().Be(1920);
        r.Success.Should().BeTrue();
    }

    // ─── LoadSceneResult ─────────────────────────────────────────────────────

    [Fact]
    public void LoadSceneResult_RoundTrip()
    {
        LoadSceneResult r = RoundTrip(new LoadSceneResult
        { Success = true, Scene = "MainMap", SceneCount = 4, BuildIndex = 1 });
        r.Success.Should().BeTrue();
        r.Scene.Should().Be("MainMap");
        r.BuildIndex.Should().Be(1);
    }

    // ─── StartGameResult ─────────────────────────────────────────────────────

    [Fact]
    public void StartGameResult_RoundTrip()
    {
        StartGameResult r = RoundTrip(new StartGameResult { Success = true, Message = "OK" });
        r.Success.Should().BeTrue();
        r.Message.Should().Be("OK");
    }

    // ─── VerifyResult ─────────────────────────────────────────────────────────

    [Fact]
    public void VerifyResult_RoundTrip()
    {
        VerifyResult r = RoundTrip(new VerifyResult
        {
            PackId = "warfare-starwars",
            Loaded = true,
            EntityCount = 28,
            StatChanges = new List<string> { "hp: 100→150" },
            Errors = new List<string>()
        });
        r.PackId.Should().Be("warfare-starwars");
        r.Loaded.Should().BeTrue();
        r.EntityCount.Should().Be(28);
        r.StatChanges.Should().ContainSingle();
    }

    // ─── WaitResult ──────────────────────────────────────────────────────────

    [Fact]
    public void WaitResult_RoundTrip()
    {
        WaitResult r = RoundTrip(new WaitResult { Ready = true, WorldName = "Default" });
        r.Ready.Should().BeTrue();
        r.WorldName.Should().Be("Default");
    }

    // ─── UiActionResult ───────────────────────────────────────────────────────

    [Fact]
    public void UiActionResult_NoMatch_RoundTrip()
    {
        UiActionResult r = RoundTrip(new UiActionResult
        { Success = false, Message = "No element", Selector = "#pack-list", MatchCount = 0 });
        r.Success.Should().BeFalse();
        r.MatchedNode.Should().BeNull();
    }

    [Fact]
    public void UiActionResult_WithNode_RoundTrip()
    {
        UiActionResult r = RoundTrip(new UiActionResult
        {
            Success = true,
            MatchCount = 1,
            Actionable = true,
            MatchedNode = new UiNode
            {
                Id = "n1",
                Name = "PackListView",
                Role = "list",
                Active = true,
                Visible = true,
                Interactable = true
            }
        });
        r.MatchedNode.Should().NotBeNull();
        r.MatchedNode!.Name.Should().Be("PackListView");
        r.Actionable.Should().BeTrue();
    }

    // ─── UiExpectationResult ─────────────────────────────────────────────────

    [Fact]
    public void UiExpectationResult_RoundTrip()
    {
        UiExpectationResult r = RoundTrip(new UiExpectationResult
        {
            Success = true,
            Selector = "#overlay",
            Condition = "visible",
            Message = "Element is visible",
            MatchCount = 1
        });
        r.Success.Should().BeTrue();
        r.Condition.Should().Be("visible");
    }

    // ─── UiWaitResult ─────────────────────────────────────────────────────────

    [Fact]
    public void UiWaitResult_TimedOut_RoundTrip()
    {
        UiWaitResult r = RoundTrip(new UiWaitResult
        { Ready = false, Selector = "#overlay", State = "visible", Message = "Timeout", MatchCount = 0 });
        r.Ready.Should().BeFalse();
        r.MatchedNode.Should().BeNull();
    }

    // ─── UiTreeResult / UiNode / UiBounds ────────────────────────────────────

    [Fact]
    public void UiTreeResult_WithHierarchy_RoundTrip()
    {
        UiTreeResult r = RoundTrip(new UiTreeResult
        {
            Success = true,
            NodeCount = 3,
            GeneratedAtUtc = "2026-03-15T00:00:00Z",
            Root = new UiNode
            {
                Id = "root",
                Name = "Canvas",
                Role = "container",
                Active = true,
                Bounds = new UiBounds { X = 0, Y = 0, Width = 1920, Height = 1080 },
                Children = new List<UiNode>
                {
                    new UiNode { Id = "c1", Name = "Panel", Role = "panel", Active = true },
                    new UiNode { Id = "c2", Name = "Button", Role = "button", Interactable = true }
                }
            }
        });
        r.Success.Should().BeTrue();
        r.NodeCount.Should().Be(3);
        r.Root.Children.Should().HaveCount(2);
        r.Root.Bounds!.Width.Should().BeApproximately(1920f, 0.01f);
        r.Root.Children[1].Interactable.Should().BeTrue();
    }

    [Fact]
    public void UiNode_DefaultsAreNonNull()
    {
        UiNode n = new UiNode();
        n.Id.Should().NotBeNull();
        n.Children.Should().NotBeNull().And.BeEmpty();
        n.Bounds.Should().BeNull();
    }

    // ─── ComponentMapResult ───────────────────────────────────────────────────

    [Fact]
    public void ComponentMapResult_RoundTrip()
    {
        ComponentMapResult r = RoundTrip(new ComponentMapResult
        {
            Mappings = new List<ComponentMapEntry>
            {
                new ComponentMapEntry { SdkPath = "units.hp", EcsType = "Components.Health", FieldName = "hp", Resolved = true },
                new ComponentMapEntry { SdkPath = "units.armor", EcsType = "Components.ArmorData", FieldName = "armor", Resolved = false }
            }
        });
        r.Mappings.Should().HaveCount(2);
        r.Mappings[0].SdkPath.Should().Be("units.hp");
        r.Mappings[0].Resolved.Should().BeTrue();
        r.Mappings[1].Resolved.Should().BeFalse();
    }
}
