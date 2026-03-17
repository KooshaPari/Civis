#nullable enable
using System;
using System.Collections.Generic;
using System.Text.Json;
using DINOForge.Bridge.Protocol;
using DINOForge.Runtime.UI;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// Unit tests for <see cref="UiActionTrace"/> — the pure-.NET trace recorder
/// used by UI automation and the UiSelectorEngine.
/// Requires InternalsVisibleTo("DINOForge.Tests") on the Runtime project.
/// </summary>
public class UiActionTraceTests : IDisposable
{
    public UiActionTraceTests()
    {
        UiActionTrace.Clear();
        UiActionTrace.Enabled = true;
    }

    public void Dispose() => UiActionTrace.Clear();

    // ─── Enabled / disabled ───────────────────────────────────────────────────

    [Fact]
    public void Record_WhenDisabled_DoesNotAddToHistory()
    {
        UiActionTrace.Enabled = false;
        UiActionTrace.Record("click", "#btn", null);
        UiActionTrace.GetHistory().Should().BeEmpty();
    }

    [Fact]
    public void Record_WhenEnabled_AddsEntry()
    {
        UiActionTrace.Enabled = true;
        UiActionTrace.Record("click", "#btn", new UiActionResult { Success = true });
        UiActionTrace.GetHistory().Should().HaveCount(1);
    }

    // ─── Record overloads ─────────────────────────────────────────────────────

    [Fact]
    public void Record_SetsActionAndSelector()
    {
        UiActionTrace.Record("query", "#pack-list", null);
        List<UiActionEntry> history = UiActionTrace.GetHistory();
        history.Should().ContainSingle();
        history[0].Action.Should().Be("query");
        history[0].Selector.Should().Be("#pack-list");
    }

    [Fact]
    public void Record_WithMatchedNode_SetsNodeIdAndPath()
    {
        UiNode node = new UiNode { Id = "n42", Path = "Canvas/Panel/Button" };
        UiActionTrace.Record("click", "#btn", new UiActionResult { Success = true }, node);

        UiActionEntry entry = UiActionTrace.GetHistory()[0];
        entry.MatchedNodeId.Should().Be("n42");
        entry.MatchedNodePath.Should().Be("Canvas/Panel/Button");
    }

    [Fact]
    public void Record_NullResult_SetsResultTypeToNull()
    {
        UiActionTrace.Record("wait", "#overlay", null);
        // Implementation stores "null" literal when result is null
        UiActionTrace.GetHistory()[0].ResultType.Should().Be("null");
    }

    [Fact]
    public void Record_MultipleEntries_AllAppended()
    {
        UiActionTrace.Record("query", "#a", null);
        UiActionTrace.Record("click", "#b", null);
        UiActionTrace.Record("expect", "#c", null);
        UiActionTrace.GetHistory().Should().HaveCount(3);
    }

    // ─── GetHistory ───────────────────────────────────────────────────────────

    [Fact]
    public void GetHistory_NoArgs_ReturnsAll()
    {
        UiActionTrace.Record("a", "#x", null);
        UiActionTrace.Record("b", "#y", null);
        UiActionTrace.GetHistory().Should().HaveCount(2);
    }

    [Fact]
    public void GetHistory_WithMaxEntries_LimitsToMostRecent()
    {
        for (int i = 0; i < 5; i++)
            UiActionTrace.Record($"act{i}", $"#sel{i}", null);

        List<UiActionEntry> last2 = UiActionTrace.GetHistory(maxEntries: 2);
        last2.Should().HaveCount(2);
        last2[^1].Action.Should().Be("act4", "last entry should be the most recent");
    }

    [Fact]
    public void GetHistory_MaxEntriesLargerThanCount_ReturnsAll()
    {
        UiActionTrace.Record("a", "#x", null);
        UiActionTrace.GetHistory(maxEntries: 100).Should().HaveCount(1);
    }

    // ─── Clear ────────────────────────────────────────────────────────────────

    [Fact]
    public void Clear_RemovesAllEntries()
    {
        UiActionTrace.Record("a", "#x", null);
        UiActionTrace.Clear();
        UiActionTrace.GetHistory().Should().BeEmpty();
    }

    // ─── ExportToJson ─────────────────────────────────────────────────────────

    [Fact]
    public void ExportToJson_EmptyHistory_ReturnsEmptyArray()
    {
        string json = UiActionTrace.ExportToJson();
        // Implementation wraps in {"traceEntries": [...]} envelope
        using JsonDocument doc = JsonDocument.Parse(json);
        doc.RootElement.GetProperty("traceEntries").GetArrayLength().Should().Be(0);
    }

    [Fact]
    public void ExportToJson_PopulatedHistory_ProducesValidJson()
    {
        UiActionTrace.Record("click", "#pack-list", new UiActionResult { Success = true, MatchCount = 1 });
        UiActionTrace.Record("query", "#status", null);

        string json = UiActionTrace.ExportToJson();

        // Must be valid JSON
        Action parse = () => JsonDocument.Parse(json);
        parse.Should().NotThrow("exported JSON must be parseable");

        using JsonDocument doc = JsonDocument.Parse(json);
        doc.RootElement.GetProperty("traceEntries").GetArrayLength().Should().Be(2);
    }

    [Fact]
    public void ExportToJson_EntryContainsActionField()
    {
        UiActionTrace.Record("expect", "#overlay", new UiExpectationResult { Success = true });
        string json = UiActionTrace.ExportToJson();
        json.Should().Contain("expect");
    }
}
