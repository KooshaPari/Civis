#nullable enable

namespace DINOForge.Bridge.Client;

/// <summary>
/// Configuration options for <see cref="GameClient"/>.
/// </summary>
public sealed class GameClientOptions
{
    /// <summary>Name of the named pipe to connect to.</summary>
    public string PipeName { get; set; } = "dinoforge-game-bridge";

    /// <summary>Timeout in milliseconds when connecting to the pipe.</summary>
    public int ConnectTimeoutMs { get; set; } = 5000;

    /// <summary>Timeout in milliseconds when sending a request.</summary>
    public int SendTimeoutMs { get; set; } = 5000;

    /// <summary>Timeout in milliseconds when reading a response.</summary>
    public int ReadTimeoutMs { get; set; } = 30000;

    /// <summary>Maximum message size in bytes (1MB default).</summary>
    public uint MaxMessageSizeBytes { get; set; } = 1_000_000;

    /// <summary>Number of retry attempts for failed operations.</summary>
    public int RetryCount { get; set; } = 3;

    /// <summary>Delay in milliseconds between retry attempts.</summary>
    public int RetryDelayMs { get; set; } = 1000;

    /// <summary>
    /// Enable message framing with length prefixes for better reliability.
    /// Default is true (recommended for production).
    /// </summary>
    public bool UseMessageFraming { get; set; } = true;

    /// <summary>
    /// When <c>true</c>, <see cref="GameClient.ConnectAsync(System.Threading.CancellationToken)"/>
    /// performs a JSON-RPC <c>connect</c> handshake against the bridge server to obtain a
    /// <c>session_id</c> + <c>session_key_b64</c> pair, populating
    /// <see cref="GameClient.SessionKeys"/> for subsequent receipt verification.
    /// </summary>
    /// <remarks>
    /// <para>
    /// Wave 2 Phase 4c sub-task A wires this up; sub-task B (#249) will flip the default
    /// from <c>false</c> to <c>true</c> once all consumer fixtures are migrated to the
    /// receipt-emitting mock. Until then, opt in explicitly per call site.
    /// </para>
    /// <para>
    /// Setting this to <c>false</c> preserves the legacy "skip handshake" behavior — useful
    /// for fixtures that target older server builds without a <c>connect</c> handler.
    /// </para>
    /// </remarks>
    public bool PerformConnectHandshake { get; set; } = true;
}
