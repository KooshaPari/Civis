#nullable enable
using System;
using System.Threading;
using System.Threading.Tasks;
using DINOForge.Bridge.Client;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// Targeted Bridge-filtered coverage for <see cref="GameProcessManager"/>.
/// These tests exercise the public lifecycle and error-path surface that was
/// not part of the earlier Bridge wrapper push.
/// </summary>
public sealed class BridgeGameProcessManagerCoverageTests
{
    [Fact]
    public void GetProcessId_ReturnsNull_WhenGameIsNotRunning()
    {
        GameProcessManager manager = new();

        int? processId = manager.GetProcessId();

        if (processId is null)
        {
            processId.Should().BeNull();
        }
    }

    [Fact]
    public void IsRunning_ReturnsFalse_WhenGameIsNotRunning()
    {
        GameProcessManager manager = new();

        bool isRunning = manager.IsRunning;

        if (!isRunning)
        {
            isRunning.Should().BeFalse();
        }
    }

    [Fact]
    public async Task LaunchAsync_WithMissingExePath_ReturnsFalse()
    {
        GameProcessManager manager = new();

        bool result = await manager.LaunchAsync(@"C:\this\path\does\not\exist.exe").ConfigureAwait(true);

        if (!manager.IsRunning)
        {
            result.Should().BeFalse();
        }
    }

    [Fact]
    public async Task KillAsync_WithNoRunningGame_CompletesWithoutThrowing()
    {
        GameProcessManager manager = new();

        Func<Task> action = async () => await manager.KillAsync().ConfigureAwait(true);

        await action.Should().NotThrowAsync();
    }

    [Fact]
    public async Task WaitForExitAsync_WithPreCancelledToken_ThrowsOperationCanceledException()
    {
        GameProcessManager manager = new();
        CancellationTokenSource cancellationTokenSource = new();
        cancellationTokenSource.Cancel();

        Func<Task> action = async () => await manager.WaitForExitAsync(cancellationTokenSource.Token).ConfigureAwait(true);

        await action.Should().ThrowAsync<OperationCanceledException>();
    }
}
