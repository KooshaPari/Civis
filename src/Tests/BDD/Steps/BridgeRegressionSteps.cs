#nullable enable
using System;
using System.Threading.Tasks;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using DINOForge.Tests.BDD.Support;
using FluentAssertions;
using Reqnroll;

namespace DINOForge.Tests.BDD.Steps;

[Binding]
public sealed class BridgeRegressionSteps
{
    private BridgeRegressionServer? _server;
    private GameClient? _client;
    private PingResult? _pingResult;
    private GameClientException? _replayException;

    [Given(@"a bridge server that mints a fresh session and signs ping responses")]
    public async Task GivenABridgeServerThatMintsAFreshSessionAndSignsPingResponses()
    {
        await StartServerAsync(replaySamePingFrame: false).ConfigureAwait(false);
    }

    [Given(@"a bridge server that replays the same signed ping frame")]
    public async Task GivenABridgeServerThatReplaysTheSameSignedPingFrame()
    {
        await StartServerAsync(replaySamePingFrame: true).ConfigureAwait(false);
    }

    [When(@"I connect the bridge client")]
    public async Task WhenIConnectTheBridgeClient()
    {
        _client.Should().NotBeNull("the bridge server must be started before connecting");
        await _client!.ConnectAsync().ConfigureAwait(false);
    }

    [When(@"I request a ping")]
    public async Task WhenIRequestAPing()
    {
        _client.Should().NotBeNull("the bridge client must be connected before pinging");
        _pingResult = await _client!.PingAsync().ConfigureAwait(false);
    }

    [When(@"I replay the same ping")]
    public async Task WhenIReplayTheSamePing()
    {
        _client.Should().NotBeNull("the bridge client must be connected before replaying a ping");

        try
        {
            _pingResult = await _client!.PingAsync().ConfigureAwait(false);
            _replayException = null;
        }
        catch (GameClientException ex)
        {
            _replayException = ex;
        }
    }

    [Then(@"the bridge client should remain connected")]
    public void ThenTheBridgeClientShouldRemainConnected()
    {
        _client.Should().NotBeNull();
        _client!.IsConnected.Should().BeTrue();
    }

    [Then(@"the ping should succeed")]
    public void ThenThePingShouldSucceed()
    {
        _pingResult.Should().NotBeNull();
        _pingResult!.Pong.Should().BeTrue();
        _pingResult.Version.Should().Be("bdd-bridge");
        _pingResult.UptimeSeconds.Should().BeGreaterThan(0);
        _server.Should().NotBeNull();
        _server!.ReceivedMethods.Should().ContainInOrder("connect", "ping");
    }

    [Then(@"the replay should be rejected as a frame regression")]
    public void ThenTheReplayShouldBeRejectedAsAFrameRegression()
    {
        _replayException.Should().NotBeNull("the second ping should be rejected by the bridge receipt verifier");
        _replayException!.Message.Should().Contain("world_frame regressed");
    }

    [AfterScenario]
    public async Task AfterScenarioAsync()
    {
        _client?.Dispose();
        _client = null;

        if (_server != null)
        {
            await _server.DisposeAsync().ConfigureAwait(false);
            _server = null;
        }
    }

    private async Task StartServerAsync(bool replaySamePingFrame)
    {
        _server = new BridgeRegressionServer(replaySamePingFrame);
        await _server.StartAsync().ConfigureAwait(false);

        _client = new GameClient(new GameClientOptions
        {
            PipeName = _server.PipeName,
            ConnectTimeoutMs = 2000,
            SendTimeoutMs = 2000,
            ReadTimeoutMs = 2000,
            RetryCount = 0,
        })
        {
            HmacVerificationMode = VerificationMode.Strict
        };
    }
}
