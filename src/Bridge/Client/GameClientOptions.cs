#nullable enable

namespace DINOForge.Bridge.Client;

/// <summary>
/// Configuration options for <see cref="GameClient"/>.
/// </summary>
public class GameClientOptions
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
}
