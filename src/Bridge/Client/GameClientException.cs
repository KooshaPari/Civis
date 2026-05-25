#nullable enable

namespace DINOForge.Bridge.Client;

/// <summary>
/// Exception thrown when a game bridge operation fails.
/// </summary>
public class GameClientException : Exception
{
    /// <summary>
    /// Initializes a new instance of <see cref="GameClientException"/> with a message.
    /// </summary>
    /// <param name="message">The error message.</param>
    public GameClientException(string message) : base(message) { }

    /// <summary>
    /// Initializes a new instance of <see cref="GameClientException"/> with a message and inner exception.
    /// </summary>
    /// <param name="message">The error message.</param>
    /// <param name="innerException">The inner exception that caused this error.</param>
    public GameClientException(string message, Exception innerException)
        : base(message, innerException) { }
}
