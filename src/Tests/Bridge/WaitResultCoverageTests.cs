#nullable enable
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Newtonsoft.Json;
using Xunit;

namespace DINOForge.Tests.Bridge;

public class WaitResultCoverageTests
{
    [Fact]
    public void CtorDefaults_InitializesExpectedValues()
    {
        // Arrange & Act
        WaitResult result = new WaitResult();

        // Assert
        result.Ready.Should().BeFalse();
        result.WorldName.Should().BeEmpty();
    }

    [Fact]
    public void Properties_CanBeSetAndRoundTripThroughJson()
    {
        // Arrange
        WaitResult original = new WaitResult
        {
            Ready = true,
            WorldName = "Overworld"
        };

        // Act
        string json = JsonConvert.SerializeObject(original);
        WaitResult? deserialized = JsonConvert.DeserializeObject<WaitResult>(json);

        // Assert
        deserialized.Should().NotBeNull();
        deserialized!.Ready.Should().BeTrue();
        deserialized.WorldName.Should().Be("Overworld");
    }
}
