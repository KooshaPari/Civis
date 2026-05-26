#nullable enable

namespace DINOForge.Bridge.Client;

/// <summary>
/// Represents the connection state of a <see cref="GameClient"/>.
/// </summary>
public enum ConnectionState
{
    /// <summary>Not connected to the game bridge.</summary>
    Disconnected,

    /// <summary>Currently attempting to connect.</summary>
    Connecting,

    /// <summary>Successfully connected and ready for requests.</summary>
    Connected,

    /// <summary>Connection is in an error state.</summary>
    Error
}
