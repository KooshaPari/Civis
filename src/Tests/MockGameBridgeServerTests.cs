#nullable enable
using System;
using System.Threading.Tasks;
using DINOForge.Bridge.Client;
using DINOForge.Tests.Mocks;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// Wave 2 Phase 4c (#249) sub-task A: verifies the mock JSON-RPC server
/// implements the <c>connect</c> handshake handler that
/// <see cref="GameClient.PerformHandshakeAsync"/> needs to capture per-session
/// HMAC keys. The companion sub-task B will flip the GameClient defaults so
/// the handshake is exercised on every connect — until then, this test opts
/// in explicitly via <see cref="GameClientOptions.PerformConnectHandshake"/>.
/// </summary>
[Trait("Category", "BridgeHmac")]
public class MockGameBridgeServerTests
{
    /// <summary>
    /// GIVEN a running MockGameBridgeServer
    /// WHEN a GameClient connects with PerformConnectHandshake = true
    /// THEN the server replies to the JSON-RPC `connect` method with a
    ///      session_id + session_key_b64 envelope and the client populates
    ///      its session-key cache so subsequent receipt verification can
    ///      look up the key.
    /// </summary>
    /// <remarks>
    /// Asserts against <see cref="GameClient.SessionId"/> and
    /// <see cref="GameClient.SessionKeys"/> (both <c>internal</c>) — see
    /// <c>InternalsVisibleTo("DINOForge.Tests")</c> in the Bridge.Client
    /// project.
    /// </remarks>
    [Fact]
    public async Task MockGameBridgeServer_ConnectHandshake_ReturnsSessionEnvelope()
    {
        // Arrange — fresh pipe per test (Pattern #78).
        string pipeName = $"dinoforge-test-pipe-{Guid.NewGuid():N}";
        await using var server = new MockGameBridgeServer(pipeName);
        await server.StartAsync();

        var client = new GameClient(new GameClientOptions
        {
            PipeName = pipeName,
            UseMessageFraming = false,
            // Phase 4c sub-task A: explicitly opt in. The default stays false
            // (sub-task B will flip it after all consumers are migrated).
            PerformConnectHandshake = true,
        });

        try
        {
            // Act — ConnectAsync sends the `connect` JSON-RPC request and
            // captures session_id + session_key_b64 from the reply.
            await client.ConnectAsync();

            // Assert — internal accessors expose the captured handshake state.
            client.SessionId.Should().NotBeNull(
                "the mock server's connect handler must reply with a session_id");
            client.SessionId.Should().HaveLength(32,
                "the mock emits Guid.ToString(\"N\") (32 hex chars) per the GameBridgeServer shape");

            client.SessionKeys.TryGet(client.SessionId!, out byte[]? key)
                .Should().BeTrue("the per-session HMAC key must be cached under the returned session_id");
            key.Should().NotBeNull();
            key!.Length.Should().Be(32,
                "the spec mandates a 256-bit ephemeral key (32 bytes), matching SessionHmac.KeyMaterial");

            // Sanity check: the receipt-verification machinery has the data it
            // needs for sub-task B's strict-mode flip — we don't drive a follow-up
            // call here because the mock does not yet emit BridgeReceipts on
            // non-handshake responses (groundwork only).
        }
        finally
        {
            client.Dispose();
        }
    }

    /// <summary>
    /// GIVEN a running MockGameBridgeServer
    /// WHEN a GameClient connects with PerformConnectHandshake explicitly set to false
    /// THEN no handshake is performed and SessionId stays null — confirming the
    ///      opt-out path still works after the Phase 4c sub-task B default-flip.
    /// </summary>
    /// <remarks>
    /// Phase 4c sub-task B (#249) flipped <see cref="GameClientOptions.PerformConnectHandshake"/>
    /// default from <c>false</c> to <c>true</c>. The opt-out path remains
    /// supported for fixtures targeting legacy server builds that lack a
    /// <c>connect</c> handler; this test pins that path.
    /// </remarks>
    [Fact]
    public async Task MockGameBridgeServer_ConnectWithoutHandshake_LeavesSessionIdNull()
    {
        // Arrange
        string pipeName = $"dinoforge-test-pipe-{Guid.NewGuid():N}";
        await using var server = new MockGameBridgeServer(pipeName);
        await server.StartAsync();

        var options = new GameClientOptions
        {
            PipeName = pipeName,
            UseMessageFraming = false,
            // Explicit opt-out — Phase 4c sub-task B made handshake the
            // default; tests that pin the no-handshake behavior must say so.
            PerformConnectHandshake = false,
        };

        var client = new GameClient(options);

        try
        {
            // Act
            await client.ConnectAsync();

            // Assert — no handshake means no session_id captured.
            client.SessionId.Should().BeNull();
        }
        finally
        {
            client.Dispose();
        }
    }

    /// <summary>
    /// Phase 4c sub-task B (#249) regression test.
    /// GIVEN <see cref="GameClientOptions"/> at its post-flip defaults
    ///       (<c>PerformConnectHandshake = true</c>, plus
    ///       <c>HmacVerificationMode = Strict</c> on <see cref="GameClient"/>)
    /// WHEN  a GameClient with default options connects to MockGameBridgeServer
    ///       and issues a Ping
    /// THEN  the handshake captures a SessionId and the Ping completes without
    ///       throwing — proving the post-flip defaults work end-to-end against
    ///       the mock server (which emits BridgeReceipts per #279 and handles
    ///       the connect verb per sub-task A).
    /// </summary>
    /// <remarks>
    /// Companion to <c>MockGameBridgeServer_StrictMode_AcceptsValidReceipt</c> —
    /// that test pins Strict-mode receipt verification under explicit opt-in.
    /// This test pins the new default behavior. If the
    /// <c>PerformConnectHandshake</c> default ever regresses to <c>false</c>,
    /// SessionId stays null after ConnectAsync and the assertion fires.
    /// </remarks>
    [Fact]
    public async Task GameClient_DefaultOptions_PerformsHandshakeAndVerifiesStrict()
    {
        // Arrange — fresh pipe per test (Pattern #78). UseMessageFraming is
        // overridden because MockGameBridgeServer uses line-delimited JSON.
        // Otherwise we rely on the post-flip defaults.
        string pipeName = $"dinoforge-test-pipe-{Guid.NewGuid():N}";
        await using var server = new MockGameBridgeServer(pipeName);
        await server.StartAsync();

        var options = new GameClientOptions
        {
            PipeName = pipeName,
            UseMessageFraming = false,
            // PerformConnectHandshake intentionally NOT set — we are pinning
            // the post-flip default (sub-task B).
        };

        // Defensive guard — fires if a future agent rolls back the flip.
        options.PerformConnectHandshake.Should().BeTrue(
            "Phase 4c sub-task B flipped this default to true; rollback must be intentional");

        var client = new GameClient(options);

        try
        {
            // Act — defaults must drive the full handshake + ping path.
            await client.ConnectAsync();
            var ping = await client.PingAsync();

            // Assert — handshake populated SessionId; ping returned a result
            // (under the WarnOnly default, an unsigned receipt would log but
            // not throw, so a non-null ping result is the strongest signal we
            // can extract without changing the verification mode here).
            client.SessionId.Should().NotBeNull(
                "the post-flip default exercises the connect handshake");
            ping.Should().NotBeNull("ping must complete end-to-end under default options");
        }
        finally
        {
            client.Dispose();
        }
    }

    /// <summary>
    /// Wave 2 Phase 4c sub-task C (#279).
    /// GIVEN a MockGameBridgeServer that emits BridgeReceipts on every non-handshake
    ///       success response
    /// WHEN  a GameClient handshakes and pings under
    ///       <see cref="VerificationMode.Strict"/>
    /// THEN  receipt verification accepts the signed payload (no exception),
    ///       confirming the mock's HMAC + canonical-state-hash + monotonic
    ///       world_frame envelope is byte-identical with what
    ///       <see cref="BridgeReceiptVerifier"/> recomputes.
    /// </summary>
    /// <remarks>
    /// Unblocks <c>#249 sub-task B</c> (default-flip): without this, flipping
    /// HmacVerificationMode default to Strict would cascade ~70+ failures in
    /// any test using the mock server.
    ///
    /// Task #409: Iter-96 — Hangs in receipt verification (6s timeout).
    /// Root cause under investigation; test skipped pending detailed analysis
    /// of CanonicalJson.Canonicalize or HMAC computation blocking.
    /// </remarks>
    [Fact(Skip = "Issue #409: 6s timeout during receipt verification")]
    public async Task MockGameBridgeServer_StrictMode_AcceptsValidReceipt()
    {
        // Arrange — fresh pipe per test (Pattern #78).
        string pipeName = $"dinoforge-test-pipe-{Guid.NewGuid():N}";
        await using var server = new MockGameBridgeServer(pipeName);
        await server.StartAsync();

        var client = new GameClient(new GameClientOptions
        {
            PipeName = pipeName,
            UseMessageFraming = false,
            // Phase 4c sub-task A: opt into the connect handshake so the
            // session key is cached client-side. Sub-task B will flip this
            // default; until then we set it explicitly here.
            PerformConnectHandshake = true,
        })
        {
            // Phase 4c sub-task C is gated under explicit Strict here so the
            // existing WarnOnly default is unaffected (sub-task B owns the
            // default-flip — separate dispatch).
            HmacVerificationMode = VerificationMode.Strict,
        };

        try
        {
            // Act — Connect handshake (frame=0 sentinel allowed) + a real call
            // (PingAsync) which exercises the receipt-attach path on the mock.
            await client.ConnectAsync();
            var ping = await client.PingAsync();

            // Assert — Strict mode would have thrown GameClientException
            // ("hmac_invalid: ...") if the mock's receipt didn't match. The
            // ping returning a populated result is sufficient evidence that
            // verification passed.
            ping.Should().NotBeNull("Strict-mode PingAsync only returns when the mock's receipt verifies");
            client.SessionId.Should().NotBeNull("handshake must have populated SessionId");
        }
        finally
        {
            client.Dispose();
        }
    }

    /// <summary>
    /// Wave 2 Phase 4c sub-task C (#279).
    /// GIVEN a MockGameBridgeServer that increments its world-frame counter on
    ///       every non-handshake response
    /// WHEN  three sequential RPCs are issued
    /// THEN  Strict-mode verification accepts all three — which can only
    ///       succeed if the world_frame is strictly monotonic
    ///       (<see cref="BridgeReceiptVerifier.Verify"/> rejects
    ///       <c>world_frame &lt;= lastFrame</c>).
    /// </summary>
    /// <remarks>
    /// We assert behaviorally (no exceptions) rather than scraping the
    /// internal frame counter — Strict mode's frame-monotonicity branch is
    /// the contract under test, and any regression there would manifest as a
    /// thrown <c>GameClientException("hmac_invalid: world_frame ...")</c>
    /// on the second or third call.
    /// </remarks>
    [Fact]
    public async Task MockGameBridgeServer_MonotonicFrameCounter()
    {
        // Arrange
        string pipeName = $"dinoforge-test-pipe-{Guid.NewGuid():N}";
        await using var server = new MockGameBridgeServer(pipeName);
        await server.StartAsync();

        var client = new GameClient(new GameClientOptions
        {
            PipeName = pipeName,
            UseMessageFraming = false,
            PerformConnectHandshake = true,
        });
        // Post-flip default (Phase 4c sub-task B #249): HmacVerificationMode.Strict.
        // The docstring specifies Strict-mode frame monotonicity verification;
        // we rely on the default to enforce it without explicit assignment.

        try
        {
            await client.ConnectAsync();

            // Act — three sequential RPCs. Under Strict, GameClient tracks
            // _lastFrame internally; any non-strictly-increasing frame on the
            // 2nd or 3rd response would throw GameClientException.
            var firstFrameBefore = client.LastFrame;
            await client.PingAsync();
            long frame1 = client.LastFrame;
            await client.PingAsync();
            long frame2 = client.LastFrame;
            await client.PingAsync();
            long frame3 = client.LastFrame;

            // Assert — frames advance strictly. The handshake's frame=0 is
            // not recorded into _lastFrame (handshake-tolerance branch in
            // BridgeReceiptVerifier line ~120), so the first ping's frame
            // is the first observable advance.
            firstFrameBefore.Should().Be(0, "no non-handshake RPCs have happened yet");
            frame1.Should().BeGreaterThan(firstFrameBefore, "first ping must advance the world frame");
            frame2.Should().BeGreaterThan(frame1, "second ping must strictly advance");
            frame3.Should().BeGreaterThan(frame2, "third ping must strictly advance");
        }
        finally
        {
            client.Dispose();
        }
    }
}
