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

/// <summary>
/// Bridge-filtered coverage for <see cref="GameClient"/> wrapper methods that
/// deserialize previously unhit protocol result models.
/// </summary>
public sealed class BridgeGameClientResultCoverageTests
{
    private static readonly UTF8Encoding Utf8NoBom = new(encoderShouldEmitUTF8Identifier: false);

    [Fact]
    public async Task StartGameAsync_ReturnsDeserializedStartGameResult()
    {
        GameClient client = CreateClientWithResponse(
            new
            {
                success = true,
                message = "started"
            },
            out MemoryStream requestStream);

        StartGameResult result = await client.StartGameAsync().ConfigureAwait(true);

        result.Success.Should().BeTrue();
        result.Message.Should().Be("started");
        AssertRequest(requestStream, "startGame", expectedParams: null);

        client.Dispose();
    }

    [Fact]
    public async Task NavigateToGameplayAsync_ReturnsNavigationTrace()
    {
        GameClient client = CreateClientWithResponse(
            new
            {
                success = true,
                message = "ok",
                plan = "skirmish",
                finalState = "gameplay",
                entityCount = 123,
                worldName = "TestWorld",
                blockedAtStep = -1,
                steps = new[]
                {
                    new
                    {
                        name = "click PLAY",
                        success = true,
                        resolvedSelector = "button.play",
                        waitSatisfied = true,
                        waitCondition = "visible",
                        screenshot = "step1.png",
                        detail = "ok"
                    }
                }
            },
            out MemoryStream requestStream);

        NavigationResult result = await client.NavigateToGameplayAsync(
            plan: "skirmish",
            screenshotDir: @"C:\shots",
            finalShot: @"C:\final.png").ConfigureAwait(true);

        result.Success.Should().BeTrue();
        result.Plan.Should().Be("skirmish");
        result.FinalState.Should().Be("gameplay");
        result.EntityCount.Should().Be(123);
        result.WorldName.Should().Be("TestWorld");
        result.BlockedAtStep.Should().Be(-1);
        result.Steps.Should().ContainSingle();
        result.Steps[0].Name.Should().Be("click PLAY");
        result.Steps[0].ResolvedSelector.Should().Be("button.play");
        AssertRequest(requestStream, "navigateToGameplay", new JObject
        {
            ["plan"] = "skirmish",
            ["screenshotDir"] = @"C:\shots",
            ["finalShot"] = @"C:\final.png"
        });

        client.Dispose();
    }

    [Fact]
    public async Task GetUiTreeAsync_ReturnsNestedUiSnapshot()
    {
        GameClient client = CreateClientWithResponse(
            new
            {
                success = true,
                message = "snapshot",
                selector = "menu",
                generatedAtUtc = "2026-06-05T22:00:00Z",
                nodeCount = 1,
                root = new
                {
                    id = "root",
                    path = "root",
                    name = "Root",
                    label = "Root",
                    role = "panel",
                    componentType = "Canvas",
                    active = true,
                    visible = true,
                    interactable = false,
                    raycastTarget = false,
                    bounds = new
                    {
                        x = 1f,
                        y = 2f,
                        width = 3f,
                        height = 4f
                    },
                    style = new
                    {
                        transition = "ColorTint",
                        normalColor = "#ffffffff",
                        highlightedColor = "#eeeeeeff",
                        pressedColor = "#ddddddff",
                        disabledColor = "#ccccccff",
                        fontSize = 14,
                        textColor = "#ffffff",
                        imageColor = "#000000"
                    },
                    children = new[]
                    {
                        new
                        {
                            id = "child",
                            path = "root/child",
                            name = "Child",
                            label = "Child",
                            role = "button",
                            componentType = "Button",
                            active = true,
                            visible = true,
                            interactable = true,
                            raycastTarget = true,
                            bounds = new
                            {
                                x = 5f,
                                y = 6f,
                                width = 7f,
                                height = 8f
                            },
                            children = Array.Empty<object>()
                        }
                    }
                }
            },
            out MemoryStream requestStream);

        UiTreeResult result = await client.GetUiTreeAsync("menu").ConfigureAwait(true);

        result.Success.Should().BeTrue();
        result.NodeCount.Should().Be(1);
        result.Root.Name.Should().Be("Root");
        result.Root.Bounds.Should().NotBeNull();
        result.Root.Bounds!.Width.Should().Be(3f);
        result.Root.Style.Should().NotBeNull();
        result.Root.Style!.Transition.Should().Be("ColorTint");
        result.Root.Children.Should().ContainSingle();
        result.Root.Children[0].Name.Should().Be("Child");
        AssertRequest(requestStream, "getUiTree", new JObject
        {
            ["selector"] = "menu"
        });

        client.Dispose();
    }

    [Fact]
    public async Task QueryUiAsync_ReturnsMatchedNodeAndActionability()
    {
        GameClient client = CreateClientWithResponse(
            new
            {
                success = true,
                message = "matched",
                selector = "menu.play",
                matchedNode = new
                {
                    id = "btn",
                    path = "root/button",
                    name = "Play",
                    label = "Play",
                    role = "button",
                    componentType = "Button",
                    active = true,
                    visible = true,
                    interactable = true,
                    raycastTarget = true,
                    children = Array.Empty<object>()
                },
                matchCount = 1,
                actionable = true,
                actionabilityReason = ""
            },
            out MemoryStream requestStream);

        UiActionResult result = await client.QueryUiAsync("menu.play").ConfigureAwait(true);

        result.Success.Should().BeTrue();
        result.MatchCount.Should().Be(1);
        result.Actionable.Should().BeTrue();
        result.MatchedNode.Should().NotBeNull();
        result.MatchedNode!.Name.Should().Be("Play");
        AssertRequest(requestStream, "queryUi", new JObject
        {
            ["selector"] = "menu.play"
        });

        client.Dispose();
    }

    [Fact]
    public async Task WaitForUiAsync_ReturnsWaitState()
    {
        GameClient client = CreateClientWithResponse(
            new
            {
                ready = true,
                selector = "menu.play",
                state = "visible",
                message = "ready",
                matchedNode = new
                {
                    id = "btn",
                    path = "root/button",
                    name = "Play",
                    label = "Play",
                    role = "button",
                    componentType = "Button",
                    active = true,
                    visible = true,
                    interactable = true,
                    raycastTarget = true,
                    children = Array.Empty<object>()
                },
                matchCount = 1
            },
            out MemoryStream requestStream);

        UiWaitResult result = await client.WaitForUiAsync("menu.play", "visible", 2500).ConfigureAwait(true);

        result.Ready.Should().BeTrue();
        result.State.Should().Be("visible");
        result.MatchCount.Should().Be(1);
        result.MatchedNode.Should().NotBeNull();
        result.MatchedNode!.Label.Should().Be("Play");
        AssertRequest(requestStream, "waitForUi", new JObject
        {
            ["selector"] = "menu.play",
            ["state"] = "visible",
            ["timeoutMs"] = 2500
        });

        client.Dispose();
    }

    [Fact]
    public async Task ExpectUiAsync_ReturnsExpectationResult()
    {
        GameClient client = CreateClientWithResponse(
            new
            {
                success = true,
                selector = "menu.play",
                condition = "visible",
                message = "ok",
                matchedNode = new
                {
                    id = "btn",
                    path = "root/button",
                    name = "Play",
                    label = "Play",
                    role = "button",
                    componentType = "Button",
                    active = true,
                    visible = true,
                    interactable = true,
                    raycastTarget = true,
                    children = Array.Empty<object>()
                },
                matchCount = 1
            },
            out MemoryStream requestStream);

        UiExpectationResult result = await client.ExpectUiAsync("menu.play", "visible").ConfigureAwait(true);

        result.Success.Should().BeTrue();
        result.Condition.Should().Be("visible");
        result.MatchCount.Should().Be(1);
        result.MatchedNode.Should().NotBeNull();
        result.MatchedNode!.Role.Should().Be("button");
        AssertRequest(requestStream, "expectUi", new JObject
        {
            ["selector"] = "menu.play",
            ["condition"] = "visible"
        });

        client.Dispose();
    }

    private static GameClient CreateClientWithResponse(object result, out MemoryStream requestStream)
    {
        string responseJson = JsonConvert.SerializeObject(new
        {
            jsonrpc = "2.0",
            id = "1",
            result
        }) + "\n";

        requestStream = new MemoryStream();
        MemoryStream responseStream = new(Utf8NoBom.GetBytes(responseJson));

        GameClient client = new(new GameClientOptions
        {
            RetryCount = 0,
            ReadTimeoutMs = 1000,
            UseMessageFraming = false
        });

        SetPrivateField(client, "_state", ConnectionState.Connected);
        SetPrivateField(client, "_reader", new StreamReader(responseStream, Utf8NoBom, false, 1024, leaveOpen: true));
        SetPrivateField(client, "_writer", new StreamWriter(requestStream, Utf8NoBom, 1024, leaveOpen: true) { AutoFlush = true });
        return client;
    }

    private static void AssertRequest(MemoryStream requestStream, string expectedMethod, JObject? expectedParams)
    {
        string requestJson = Utf8NoBom.GetString(requestStream.ToArray());
        JsonRpcRequest request = JsonConvert.DeserializeObject<JsonRpcRequest>(requestJson)
            ?? throw new Xunit.Sdk.XunitException("Request did not deserialize.");

        request.Method.Should().Be(expectedMethod);
        if (expectedParams is null)
        {
            request.Params.Should().BeNull();
            return;
        }

        request.Params.Should().NotBeNull();
        request.Params!.ToString(Formatting.None).Should().Be(expectedParams.ToString(Formatting.None));
    }

    private static void SetPrivateField<T>(GameClient client, string fieldName, T value)
    {
        FieldInfo field = typeof(GameClient).GetField(fieldName, BindingFlags.Instance | BindingFlags.NonPublic)
            ?? throw new System.InvalidOperationException($"Field '{fieldName}' not found.");

        field.SetValue(client, value);
    }
}
