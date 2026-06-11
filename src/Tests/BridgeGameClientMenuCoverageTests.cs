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
/// Bridge-filtered coverage for menu/save <see cref="GameClient"/> wrappers
/// that were not covered by the previous Bridge wrapper pushes.
/// </summary>
public sealed class BridgeGameClientMenuCoverageTests
{
    private static readonly UTF8Encoding Utf8NoBom = new(encoderShouldEmitUTF8Identifier: false);

    [Fact]
    public async Task ListSavesAsync_ReturnsRawSavePayload()
    {
        GameClient client = CreateClientWithResponse(
            new
            {
                saves = new[] { "AUTOSAVE_1", "manual-slot" },
                count = 2
            },
            out MemoryStream requestStream);

        JObject result = await client.ListSavesAsync().ConfigureAwait(true);

        result.Value<int>("count").Should().Be(2);
        result["saves"]!.Values<string>().Should().ContainInOrder("AUTOSAVE_1", "manual-slot");
        AssertRequest(requestStream, "listSaves", expectedParams: null);

        client.Dispose();
    }

    [Fact]
    public async Task DismissLoadScreenAsync_ReturnsStartGameResult()
    {
        GameClient client = CreateClientWithResponse(
            new
            {
                success = true,
                message = "dismissed"
            },
            out MemoryStream requestStream);

        StartGameResult result = await client.DismissLoadScreenAsync().ConfigureAwait(true);

        result.Success.Should().BeTrue();
        result.Message.Should().Be("dismissed");
        AssertRequest(requestStream, "dismissLoadScreen", expectedParams: null);

        client.Dispose();
    }

    [Fact]
    public async Task LoadSaveAsync_ForwardsSaveName()
    {
        GameClient client = CreateClientWithResponse(
            new
            {
                success = true,
                message = "loaded"
            },
            out MemoryStream requestStream);

        StartGameResult result = await client.LoadSaveAsync("manual-slot").ConfigureAwait(true);

        result.Success.Should().BeTrue();
        result.Message.Should().Be("loaded");
        AssertRequest(requestStream, "loadSave", new JObject
        {
            ["saveName"] = "manual-slot"
        });

        client.Dispose();
    }

    [Fact]
    public async Task ClickButtonAsync_ForwardsButtonName()
    {
        GameClient client = CreateClientWithResponse(
            new
            {
                success = true,
                message = "clicked"
            },
            out MemoryStream requestStream);

        StartGameResult result = await client.ClickButtonAsync("DINOForge_ModsButton").ConfigureAwait(true);

        result.Success.Should().BeTrue();
        result.Message.Should().Be("clicked");
        AssertRequest(requestStream, "clickButton", new JObject
        {
            ["buttonName"] = "DINOForge_ModsButton"
        });

        client.Dispose();
    }

    [Fact]
    public async Task ToggleUiAsync_ForwardsTarget()
    {
        GameClient client = CreateClientWithResponse(
            new
            {
                success = true,
                message = "debug toggled"
            },
            out MemoryStream requestStream);

        StartGameResult result = await client.ToggleUiAsync("debug").ConfigureAwait(true);

        result.Success.Should().BeTrue();
        result.Message.Should().Be("debug toggled");
        AssertRequest(requestStream, "toggleUi", new JObject
        {
            ["target"] = "debug"
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
            ?? throw new InvalidOperationException($"Field '{fieldName}' not found.");

        field.SetValue(client, value);
    }
}
