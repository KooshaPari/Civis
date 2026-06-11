#nullable enable
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Newtonsoft.Json;
using Xunit;

namespace DINOForge.Tests.Bridge;

public class PingResultCoverageTests
{
    [Fact]
    public void CtorDefaults_InitializesExpectedValues()
    {
        // Arrange & Act
        PingResult result = new PingResult();

        // Assert
        result.Pong.Should().BeFalse();
        result.Version.Should().BeEmpty();
        result.UptimeSeconds.Should().Be(0d);
    }

    [Fact]
    public void Properties_CanBeSetAndRoundTripThroughJson()
    {
        // Arrange
        PingResult original = new PingResult
        {
            Pong = true,
            Version = "1.2.3",
            UptimeSeconds = 123.45
        };

        // Act
        string json = JsonConvert.SerializeObject(original);
        PingResult? deserialized = JsonConvert.DeserializeObject<PingResult>(json);

        // Assert
        deserialized.Should().NotBeNull();
        deserialized!.Pong.Should().BeTrue();
        deserialized.Version.Should().Be("1.2.3");
        deserialized.UptimeSeconds.Should().Be(123.45);
    }
}
