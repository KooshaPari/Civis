#nullable enable
using System;

namespace DINOForge.Bridge.Protocol;

/// <summary>
/// Thrown when a protocol violation is detected in the JSON-RPC communication.
/// Examples: malformed frames, incomplete messages, size violations.
/// </summary>
public class ProtocolException : InvalidOperationException
{
    /// <summary>
    /// Initializes a new instance of <see cref="ProtocolException"/> with the specified message.
    /// </summary>
    /// <param name="message">Description of the protocol violation.</param>
    public ProtocolException(string message) : base(message) { }

    /// <summary>
    /// Initializes a new instance of <see cref="ProtocolException"/> with the specified message and inner exception.
    /// </summary>
    /// <param name="message">Description of the protocol violation.</param>
    /// <param name="innerException">The exception that caused this error.</param>
    public ProtocolException(string message, Exception innerException) : base(message, innerException) { }
}
