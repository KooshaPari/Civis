using DINOForge.Tools.McpServer.Cua;

namespace DINOForge.Tools.McpServer.PlayCua;

/// <summary>Thin wrapper for playcua-native; see <see cref="CuaNativeClient"/>.</summary>
public sealed class NativeComputer : CuaNativeClient
{
    public NativeComputer() : base(CuaNativeClientOptions.Play) { }

    public static Task<NativeComputer> StartAsync(
        string nativePath = "playcua-native",
        string logLevel = "info",
        CancellationToken ct = default)
        => StartAsync<NativeComputer>(CuaNativeClientOptions.Play, nativePath, logLevel, ct);
}
