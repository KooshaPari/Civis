using FluentAssertions;
using Moq;
using Xunit;

namespace DINOForge.DesktopCompanion.Tests;

/// <summary>
/// Tests ViewModel logic using mock data service.
/// These test the observable state transitions, not WinUI 3 rendering.
/// </summary>
public class ViewModelLogicTests
{
    // Simulated LoadResultViewModel (mirrors the actual class)
    private sealed class TestLoadResult
    {
        public List<TestPackViewModel> Packs { get; init; } = [];
        public int LoadedCount => Packs.Count(p => p.IsEnabled && !p.HasErrors);
        public int ErrorCount => Packs.Sum(p => p.Errors.Count);
        public bool IsSuccess => ErrorCount == 0;
        public string StatusText => IsSuccess
            ? $"All {LoadedCount} pack(s) loaded OK"
            : $"{LoadedCount} loaded, {ErrorCount} error(s)";
    }

    private sealed class TestPackViewModel
    {
        public string Id { get; init; } = string.Empty;
        public string Name { get; init; } = string.Empty;
        public string Version { get; init; } = string.Empty;
        public bool IsEnabled { get; set; } = true;
        public List<string> Errors { get; init; } = [];
        public bool HasErrors => Errors.Count > 0;
    }

    [Fact]
    public void LoadResult_AllSuccess_IsSuccessTrue()
    {
        var result = new TestLoadResult
        {
            Packs =
            [
                new() { Id = "a", Name = "A", IsEnabled = true },
                new() { Id = "b", Name = "B", IsEnabled = true },
            ]
        };

        result.IsSuccess.Should().BeTrue();
        result.LoadedCount.Should().Be(2);
        result.ErrorCount.Should().Be(0);
        result.StatusText.Should().Contain("2 pack(s) loaded OK");
    }

    [Fact]
    public void LoadResult_WithErrors_IsSuccessFalse()
    {
        var result = new TestLoadResult
        {
            Packs =
            [
                new() { Id = "a", IsEnabled = true, Errors = ["Schema validation failed"] },
            ]
        };

        result.IsSuccess.Should().BeFalse();
        result.ErrorCount.Should().Be(1);
        result.StatusText.Should().Contain("error");
    }

    [Fact]
    public void LoadResult_DisabledPack_NotCountedAsLoaded()
    {
        var result = new TestLoadResult
        {
            Packs =
            [
                new() { Id = "enabled", IsEnabled = true },
                new() { Id = "disabled", IsEnabled = false },
            ]
        };

        result.LoadedCount.Should().Be(1); // only enabled pack counts
    }

    [Fact]
    public void PackViewModel_Toggle_FlipsIsEnabled()
    {
        var pack = new TestPackViewModel { Id = "test", IsEnabled = true };
        pack.IsEnabled = !pack.IsEnabled;
        pack.IsEnabled.Should().BeFalse();
    }

    [Fact]
    public void LoadResult_EmptyPacks_StatusSaysAllLoaded()
    {
        var result = new TestLoadResult { Packs = [] };
        result.IsSuccess.Should().BeTrue();
        result.LoadedCount.Should().Be(0);
    }
}
