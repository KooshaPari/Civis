using System.Text.Json.Serialization;

namespace DINOForge.Tools.McpServer.Cua;

/// <summary>Metadata about a top-level window.</summary>
public sealed record WindowInfo(
    [property: JsonPropertyName("hwnd")] long Hwnd,
    [property: JsonPropertyName("title")] string Title,
    [property: JsonPropertyName("pid")] uint Pid,
    [property: JsonPropertyName("x")] int X,
    [property: JsonPropertyName("y")] int Y,
    [property: JsonPropertyName("width")] int Width,
    [property: JsonPropertyName("height")] int Height,
    [property: JsonPropertyName("visible")] bool Visible);

/// <summary>Process running state.</summary>
public sealed record ProcessStatus(
    [property: JsonPropertyName("running")] bool Running,
    [property: JsonPropertyName("exit_code")] int? ExitCode);

/// <summary>Thrown when the native binary returns a JSON-RPC error object.</summary>
public sealed class RpcException : Exception
{
    public int Code { get; }

    public RpcException(int code, string message) : base($"RPC error {code}: {message}")
    {
        Code = code;
    }
}
