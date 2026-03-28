#nullable enable
using System;
using System.IO;
using System.Reflection;
using System.Text;
using System.Threading.Tasks;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using Xunit;

namespace DINOForge.Tests;

public class GameClientPipelineTests
{
    private static readonly UTF8Encoding Utf8NoBom = new(encoderShouldEmitUTF8Identifier: false);

    [Fact]
    public async Task CoreRequestWrappers_WriteExpectedJsonRpcRequests()
    {
        (PingResult ping, JObject pingRequest) = await InvokeAsync(
            client => client.PingAsync(),
            new PingResult { Pong = true, Version = "1.0.0", UptimeSeconds = 12.5 });
        ping.Pong.Should().BeTrue();
        AssertMethod(pingRequest, "ping");
        pingRequest["params"].Should().BeNull();

        (GameStatus status, JObject statusRequest) = await InvokeAsync(
            client => client.StatusAsync(),
            new GameStatus
            {
                Running = true,
                WorldReady = true,
                WorldName = "TestWorld",
                EntityCount = 7,
                ModPlatformReady = true,
                LoadedPacks = ["pack-a", "pack-b"],
                Version = "1.0.0"
            });
        status.WorldName.Should().Be("TestWorld");
        AssertMethod(statusRequest, "status");

        (WaitResult wait, JObject waitRequest) = await InvokeAsync(
            client => client.WaitForWorldAsync(2500),
            new WaitResult { Ready = true, WorldName = "TestWorld" });
        wait.Ready.Should().BeTrue();
        AssertMethod(waitRequest, "waitForWorld");
        waitRequest["params"]!["timeoutMs"]!.Value<int>().Should().Be(2500);

        (QueryResult query, JObject queryRequest) = await InvokeAsync(
            client => client.QueryEntitiesAsync("Health", "unit"),
            new QueryResult
            {
                Count = 1,
                Entities =
                [
                    new EntityInfo { Index = 3, Components = ["Health", "Armor"] }
                ]
            });
        query.Count.Should().Be(1);
        AssertMethod(queryRequest, "queryEntities");
        queryRequest["params"]!["componentType"]!.Value<string>().Should().Be("Health");
        queryRequest["params"]!["category"]!.Value<string>().Should().Be("unit");

        (StatResult stat, JObject statRequest) = await InvokeAsync(
            client => client.GetStatAsync("unit.stats.hp", 7),
            new StatResult
            {
                SdkPath = "unit.stats.hp",
                Value = 42.5f,
                EntityCount = 2,
                Values = [41f, 42.5f],
                ComponentType = "Health",
                FieldName = "hp"
            });
        stat.Value.Should().Be(42.5f);
        AssertMethod(statRequest, "getStat");
        statRequest["params"]!["entityIndex"]!.Value<int>().Should().Be(7);

        (OverrideResult overrideResult, JObject overrideRequest) = await InvokeAsync(
            client => client.ApplyOverrideAsync("unit.stats.hp", 12.5f, "set", "elite"),
            new OverrideResult
            {
                Success = true,
                ModifiedCount = 4,
                SdkPath = "unit.stats.hp",
                Message = "ok"
            });
        overrideResult.ModifiedCount.Should().Be(4);
        AssertMethod(overrideRequest, "applyOverride");
        overrideRequest["params"]!["mode"]!.Value<string>().Should().Be("set");
        overrideRequest["params"]!["filter"]!.Value<string>().Should().Be("elite");

        (ReloadResult reload, JObject reloadRequest) = await InvokeAsync(
            client => client.ReloadPacksAsync("packs/warfare"),
            new ReloadResult
            {
                Success = true,
                LoadedPacks = ["pack-a"],
                Errors = []
            });
        reload.Success.Should().BeTrue();
        AssertMethod(reloadRequest, "reloadPacks");
        reloadRequest["params"]!["path"]!.Value<string>().Should().Be("packs/warfare");

        (CatalogSnapshot catalog, JObject catalogRequest) = await InvokeAsync(
            client => client.GetCatalogAsync(),
            new CatalogSnapshot
            {
                Units = [new CatalogEntry { InferredId = "vanilla:unit", ComponentCount = 3, EntityCount = 2, Category = "unit" }]
            });
        catalog.Units.Should().ContainSingle();
        AssertMethod(catalogRequest, "getCatalog");

        (CatalogSnapshot dump, JObject dumpRequest) = await InvokeAsync(
            client => client.DumpStateAsync("ui"),
            new CatalogSnapshot
            {
                Buildings = [new CatalogEntry { InferredId = "vanilla:building", ComponentCount = 5, EntityCount = 1, Category = "building" }]
            });
        dump.Buildings.Should().ContainSingle();
        AssertMethod(dumpRequest, "dumpState");
        dumpRequest["params"]!["category"]!.Value<string>().Should().Be("ui");

        (ResourceSnapshot resources, JObject resourcesRequest) = await InvokeAsync(
            client => client.GetResourcesAsync(),
            new ResourceSnapshot
            {
                Food = 100,
                Wood = 200,
                Stone = 50,
                Iron = 25,
                Money = 300,
                Souls = 5,
                Bones = 2,
                Spirit = 1
            });
        resources.Wood.Should().Be(200);
        AssertMethod(resourcesRequest, "getResources");

        (ScreenshotResult screenshot, JObject screenshotRequest) = await InvokeAsync(
            client => client.ScreenshotAsync("C:/captures/shot.png"),
            new ScreenshotResult { Path = "C:/captures/shot.png", Width = 1920, Height = 1080, Success = true });
        screenshot.Success.Should().BeTrue();
        AssertMethod(screenshotRequest, "screenshot");
        screenshotRequest["params"]!["path"]!.Value<string>().Should().Be("C:/captures/shot.png");

        (LoadSceneResult loadScene, JObject loadSceneRequest) = await InvokeAsync(
            client => client.LoadSceneAsync("MainMenu"),
            new LoadSceneResult { Success = true, Scene = "MainMenu", SceneCount = 5, BuildIndex = 0 });
        loadScene.Scene.Should().Be("MainMenu");
        AssertMethod(loadSceneRequest, "loadScene");

        (StartGameResult startGame, JObject startGameRequest) = await InvokeAsync(
            client => client.StartGameAsync(),
            new StartGameResult { Success = true, Message = "started" });
        startGame.Success.Should().BeTrue();
        AssertMethod(startGameRequest, "startGame");

        (JObject saves, JObject savesRequest) = await InvokeAsync(
            client => client.ListSavesAsync(),
            new JObject { ["saves"] = new JArray("AUTOSAVE_1") });
        saves["saves"]!.Should().NotBeNull();
        AssertMethod(savesRequest, "listSaves");

        (StartGameResult dismiss, JObject dismissRequest) = await InvokeAsync(
            client => client.DismissLoadScreenAsync(),
            new StartGameResult { Success = true, Message = "dismissed" });
        dismiss.Message.Should().Be("dismissed");
        AssertMethod(dismissRequest, "dismissLoadScreen");

        (StartGameResult loadSave, JObject loadSaveRequest) = await InvokeAsync(
            client => client.LoadSaveAsync("AUTOSAVE_2"),
            new StartGameResult { Success = true, Message = "loaded" });
        loadSave.Success.Should().BeTrue();
        AssertMethod(loadSaveRequest, "loadSave");
        loadSaveRequest["params"]!["saveName"]!.Value<string>().Should().Be("AUTOSAVE_2");

        (StartGameResult clickButton, JObject clickButtonRequest) = await InvokeAsync(
            client => client.ClickButtonAsync("DINOForge_ModsButton"),
            new StartGameResult { Success = true, Message = "clicked" });
        clickButton.Message.Should().Be("clicked");
        AssertMethod(clickButtonRequest, "clickButton");
        clickButtonRequest["params"]!["buttonName"]!.Value<string>().Should().Be("DINOForge_ModsButton");

        (StartGameResult toggleUi, JObject toggleUiRequest) = await InvokeAsync(
            client => client.ToggleUiAsync("debug"),
            new StartGameResult { Success = true, Message = "toggled" });
        toggleUi.Success.Should().BeTrue();
        AssertMethod(toggleUiRequest, "toggleUi");
        toggleUiRequest["params"]!["target"]!.Value<string>().Should().Be("debug");

        (StartGameResult scanScene, JObject scanSceneRequest) = await InvokeAsync(
            client => client.ScanSceneAsync("GameManager"),
            new StartGameResult { Success = true, Message = "scanned" });
        scanScene.Message.Should().Be("scanned");
        AssertMethod(scanSceneRequest, "pressKey");
        scanSceneRequest["params"]!["filter"]!.Value<string>().Should().Be("GameManager");

        (StartGameResult invokeMethod, JObject invokeMethodRequest) = await InvokeAsync(
            client => client.InvokeMethodAsync("GameManager", "Reset"),
            new StartGameResult { Success = true, Message = "invoked" });
        invokeMethod.Message.Should().Be("invoked");
        AssertMethod(invokeMethodRequest, "invokeMethod");
        invokeMethodRequest["params"]!["target"]!.Value<string>().Should().Be("GameManager");
        invokeMethodRequest["params"]!["method"]!.Value<string>().Should().Be("Reset");

        (VerifyResult verify, JObject verifyRequest) = await InvokeAsync(
            client => client.VerifyModAsync("packs/test"),
            new VerifyResult
            {
                PackId = "packs/test",
                Loaded = true,
                StatChanges = ["hp:+1"],
                Errors = [],
                EntityCount = 3
            });
        verify.Loaded.Should().BeTrue();
        AssertMethod(verifyRequest, "verifyMod");
        verifyRequest["params"]!["packPath"]!.Value<string>().Should().Be("packs/test");

        (ComponentMapResult componentMap, JObject componentMapRequest) = await InvokeAsync(
            client => client.GetComponentMapAsync("unit.stats.hp"),
            new ComponentMapResult
            {
                Mappings =
                [
                    new ComponentMapEntry
                    {
                        SdkPath = "unit.stats.hp",
                        EcsType = "Components.Health",
                        FieldName = "currentHealth",
                        Resolved = true,
                        Description = "HP mapping"
                    }
                ]
            });
        componentMap.Mappings.Should().ContainSingle();
        AssertMethod(componentMapRequest, "getComponentMap");
        componentMapRequest["params"]!["sdkPath"]!.Value<string>().Should().Be("unit.stats.hp");
    }

    [Fact]
    public async Task UiRequestWrappers_WriteExpectedJsonRpcRequests()
    {
        (UiTreeResult tree, JObject treeRequest) = await InvokeAsync(
            client => client.GetUiTreeAsync("main"),
            new UiTreeResult
            {
                Success = true,
                Message = "ok",
                Selector = "main",
                GeneratedAtUtc = "2026-03-27T00:00:00Z",
                NodeCount = 1,
                Root = new UiNode
                {
                    Id = "root",
                    Path = "/root",
                    Name = "Root",
                    Label = "Root",
                    Role = "panel",
                    ComponentType = "UnityEngine.UI.Image",
                    Active = true,
                    Visible = true,
                    Interactable = true,
                    RaycastTarget = false,
                    Bounds = new UiBounds { X = 1, Y = 2, Width = 3, Height = 4 }
                }
            });
        tree.Root.Should().NotBeNull();
        AssertMethod(treeRequest, "getUiTree");
        treeRequest["params"]!["selector"]!.Value<string>().Should().Be("main");

        UiNode matched = new()
        {
            Id = "btn",
            Path = "/root/button",
            Name = "Button",
            Label = "Button",
            Role = "button",
            ComponentType = "UnityEngine.UI.Button",
            Active = true,
            Visible = true,
            Interactable = true,
            RaycastTarget = true
        };

        (UiActionResult query, JObject queryRequest) = await InvokeAsync(
            client => client.QueryUiAsync("button.mods"),
            new UiActionResult
            {
                Success = true,
                Message = "found",
                Selector = "button.mods",
                MatchedNode = matched,
                MatchCount = 1,
                Actionable = true,
                ActionabilityReason = ""
            });
        query.MatchCount.Should().Be(1);
        AssertMethod(queryRequest, "queryUi");
        queryRequest["params"]!["selector"]!.Value<string>().Should().Be("button.mods");

        (UiActionResult click, JObject clickRequest) = await InvokeAsync(
            client => client.ClickUiAsync("button.mods"),
            new UiActionResult
            {
                Success = true,
                Message = "clicked",
                Selector = "button.mods",
                MatchCount = 1,
                Actionable = true,
                ActionabilityReason = ""
            });
        click.Success.Should().BeTrue();
        AssertMethod(clickRequest, "clickUi");

        (UiWaitResult wait, JObject waitRequest) = await InvokeAsync(
            client => client.WaitForUiAsync("button.mods", "visible", 100),
            new UiWaitResult
            {
                Ready = true,
                Selector = "button.mods",
                State = "visible",
                Message = "ready",
                MatchedNode = matched,
                MatchCount = 1
            });
        wait.Ready.Should().BeTrue();
        AssertMethod(waitRequest, "waitForUi");
        waitRequest["params"]!["timeoutMs"]!.Value<int>().Should().Be(100);

        (UiExpectationResult expect, JObject expectRequest) = await InvokeAsync(
            client => client.ExpectUiAsync("button.mods", "visible"),
            new UiExpectationResult
            {
                Success = true,
                Selector = "button.mods",
                Condition = "visible",
                Message = "ok",
                MatchedNode = matched,
                MatchCount = 1
            });
        expect.Success.Should().BeTrue();
        AssertMethod(expectRequest, "expectUi");
        expectRequest["params"]!["condition"]!.Value<string>().Should().Be("visible");
    }

    [Fact]
    public async Task ServerErrorResponse_ThrowsGameClientException()
    {
        GameClient client = CreateConnectedClient(
            new JsonRpcResponse
            {
                Id = "1",
                Error = new JsonRpcError { Code = -32000, Message = "boom" }
            });

        Func<Task> action = async () => await client.PingAsync();

        await action.Should().ThrowAsync<GameClientException>()
            .WithMessage("Failed to execute*");

        client.Dispose();
    }

    [Fact]
    public void Disconnect_ClearsConnectedState()
    {
        GameClient client = CreateConnectedClient(
            new JsonRpcResponse { Id = "1", Result = JToken.FromObject(new PingResult { Pong = true }) });

        client.Disconnect();

        client.State.Should().Be(ConnectionState.Disconnected);
        client.IsConnected.Should().BeFalse();
        client.Dispose();
    }

    private static async Task<(T Result, JObject Request)> InvokeAsync<T>(
        Func<GameClient, Task<T>> call,
        object responsePayload)
    {
        GameClient client = CreateConnectedClient(
            new JsonRpcResponse
            {
                Id = "1",
                Result = JToken.FromObject(responsePayload)
            });

        MemoryStream requestStream = GetRequestStream(client);
        try
        {
            T result = await call(client).ConfigureAwait(false);
            string requestJson = Utf8NoBom.GetString(requestStream.ToArray()).Trim();
            JObject request = JObject.Parse(requestJson);
            return (result, request);
        }
        finally
        {
            client.Dispose();
        }
    }

    private static GameClient CreateConnectedClient(JsonRpcResponse response)
    {
        GameClient client = new(new GameClientOptions
        {
            RetryCount = 0,
            ReadTimeoutMs = 1000
        });

        MemoryStream responseStream = new(Utf8NoBom.GetBytes(JsonConvert.SerializeObject(response) + Environment.NewLine));
        MemoryStream requestStream = new();

        SetPrivateField(client, "_state", ConnectionState.Connected);
        SetPrivateField(client, "_reader", new StreamReader(responseStream, Utf8NoBom, false, 1024, leaveOpen: true));
        SetPrivateField(client, "_writer", new StreamWriter(requestStream, Utf8NoBom, 1024, leaveOpen: true)
        {
            AutoFlush = true
        });
        return client;
    }

    private static MemoryStream GetRequestStream(GameClient client)
    {
        FieldInfo? field = typeof(GameClient).GetField("_writer", BindingFlags.Instance | BindingFlags.NonPublic);
        StreamWriter? writer = field?.GetValue(client) as StreamWriter;
        MemoryStream? stream = writer?.BaseStream as MemoryStream;
        return stream ?? throw new InvalidOperationException("Request stream not available.");
    }

    private static void AssertMethod(JObject request, string expectedMethod)
    {
        request["method"]!.Value<string>().Should().Be(expectedMethod);
    }

    private static void SetPrivateField<T>(GameClient client, string fieldName, T value)
    {
        FieldInfo field = typeof(GameClient).GetField(fieldName, BindingFlags.Instance | BindingFlags.NonPublic)
            ?? throw new InvalidOperationException($"Field '{fieldName}' not found.");

        field.SetValue(client, value);
    }
}
