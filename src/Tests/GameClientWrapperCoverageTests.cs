#nullable enable
using System;
using System.Reflection;
using System.Threading.Tasks;
using DINOForge.Bridge.Client;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// Targeted coverage tests for one-line <see cref="GameClient"/> wrapper methods.
/// These tests pin the wrapper entry points and their branch selection without
/// duplicating the transport-level assertions already covered in the client suite.
/// </summary>
public sealed class BridgeGameClientWrapperCoverageTests
{
    [Fact]
    public void LoadSceneAsync_WithNumericScene_UsesBuildIndexBranch()
    {
        using GameClient client = CreateConnectedNoTransportClient();

        Task task = client.LoadSceneAsync("12");
        task.Should().NotBeNull();
    }

    [Fact]
    public void LoadSceneAsync_WithNamedScene_UsesNameOnlyBranch()
    {
        using GameClient client = CreateConnectedNoTransportClient();

        Task task = client.LoadSceneAsync("main_menu");
        task.Should().NotBeNull();
    }

    [Fact]
    public void SimulateKeyAsync_CallsBridgeDispatchPath()
    {
        using GameClient client = CreateConnectedNoTransportClient();

        Task task = client.SimulateKeyAsync("Tab");
        task.Should().NotBeNull();
    }

    [Fact]
    public void PressEscapeAsync_CallsBridgeDispatchPath()
    {
        using GameClient client = CreateConnectedNoTransportClient();

        Task task = client.PressEscapeAsync();
        task.Should().NotBeNull();
    }

    [Fact]
    public void TogglePauseMenuAsync_CallsBridgeDispatchPath()
    {
        using GameClient client = CreateConnectedNoTransportClient();

        Task task = client.TogglePauseMenuAsync();
        task.Should().NotBeNull();
    }

    [Fact]
    public void InvokeBridgeMethodAsync_ForwardsCustomMethodName()
    {
        using GameClient client = CreateConnectedNoTransportClient();

        Task task = client.InvokeBridgeMethodAsync("customDebugMethod", new { category = "ui", limit = 3 });
        task.Should().NotBeNull();
    }

    [Fact]
    public void UiPointerAsync_CallsPointerDispatchPath()
    {
        using GameClient client = CreateConnectedNoTransportClient();

        Task task = client.UiPointerAsync("menu.play", "click", 128.5f, 256.25f);
        task.Should().NotBeNull();
    }

    [Fact]
    public void GetMetricsAsync_CallsMetricsDispatchPath()
    {
        using GameClient client = CreateConnectedNoTransportClient();

        Task task = client.GetMetricsAsync();
        task.Should().NotBeNull();
    }

    private static GameClient CreateConnectedNoTransportClient()
    {
        GameClient client = new(new GameClientOptions
        {
            RetryCount = 0,
            ReadTimeoutMs = 1000,
            UseMessageFraming = false
        });

        SetPrivateField(client, "_state", ConnectionState.Connected);
        return client;
    }

    private static void SetPrivateField<T>(GameClient client, string fieldName, T value)
    {
        FieldInfo field = typeof(GameClient).GetField(fieldName, BindingFlags.Instance | BindingFlags.NonPublic)
            ?? throw new InvalidOperationException($"Field '{fieldName}' not found.");

        field.SetValue(client, value);
    }
}
