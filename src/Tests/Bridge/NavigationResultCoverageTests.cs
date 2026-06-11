#nullable enable
using System.Collections.Generic;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Newtonsoft.Json;
using Xunit;

namespace DINOForge.Tests.Bridge;

/// <summary>
/// Coverage for <see cref="NavigationResult"/> (+ nested <see cref="NavigationStepResult"/>) —
/// constructor defaults (incl. the -1 BlockedAtStep sentinel and empty Steps) and JSON round-trip.
/// </summary>
public class NavigationResultCoverageTests
{
    [Fact]
    public void NavigationResult_Defaults_AreEmptyWithSentinelBlockedStep()
    {
        NavigationResult r = new();

        r.Success.Should().BeFalse();
        r.Message.Should().BeEmpty();
        r.Plan.Should().BeEmpty();
        r.FinalState.Should().BeEmpty();
        r.EntityCount.Should().Be(0);
        r.WorldName.Should().BeEmpty();
        r.BlockedAtStep.Should().Be(-1); // sentinel: "not blocked"
        r.Steps.Should().NotBeNull().And.BeEmpty();
    }

    [Fact]
    public void NavigationStepResult_Defaults_AreEmptyAndFalse()
    {
        NavigationStepResult s = new();

        s.Name.Should().BeEmpty();
        s.Success.Should().BeFalse();
        s.ResolvedSelector.Should().BeEmpty();
        s.WaitSatisfied.Should().BeFalse();
        s.WaitCondition.Should().BeEmpty();
        s.Screenshot.Should().BeEmpty();
        s.Detail.Should().BeEmpty();
    }

    [Fact]
    public void NavigationResult_WithSteps_RoundTripsThroughJson()
    {
        NavigationResult original = new()
        {
            Success = true,
            Message = "navigated",
            Plan = "click MODS",
            FinalState = "mods-panel",
            EntityCount = 12,
            WorldName = "Default",
            BlockedAtStep = 2,
            Steps = new List<NavigationStepResult>
            {
                new()
                {
                    Name = "open-menu",
                    Success = true,
                    ResolvedSelector = "#mods",
                    WaitSatisfied = true,
                    WaitCondition = "visible",
                    Screenshot = "shot1.png",
                    Detail = "ok"
                }
            }
        };

        NavigationResult? back = JsonConvert.DeserializeObject<NavigationResult>(JsonConvert.SerializeObject(original));

        back.Should().NotBeNull();
        back!.Success.Should().BeTrue();
        back.Message.Should().Be("navigated");
        back.Plan.Should().Be("click MODS");
        back.FinalState.Should().Be("mods-panel");
        back.EntityCount.Should().Be(12);
        back.WorldName.Should().Be("Default");
        back.BlockedAtStep.Should().Be(2);
        back.Steps.Should().ContainSingle();
        back.Steps[0].Name.Should().Be("open-menu");
        back.Steps[0].Success.Should().BeTrue();
        back.Steps[0].ResolvedSelector.Should().Be("#mods");
        back.Steps[0].WaitSatisfied.Should().BeTrue();
        back.Steps[0].WaitCondition.Should().Be("visible");
        back.Steps[0].Screenshot.Should().Be("shot1.png");
        back.Steps[0].Detail.Should().Be("ok");
    }
}
