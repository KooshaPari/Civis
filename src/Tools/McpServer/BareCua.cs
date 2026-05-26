using DINOForge.Tools.McpServer.Cua;

namespace BareCua;

/// <summary>Thin wrapper for bare-cua-native; see <see cref="CuaNativeClient"/>.</summary>
public sealed class NativeComputer : CuaNativeClient
{
    public NativeComputer() : base(CuaNativeClientOptions.Bare) { }

    public static Task<NativeComputer> StartAsync(
        string nativePath = "bare-cua-native",
        string logLevel = "info",
        CancellationToken ct = default)
        => StartAsync<NativeComputer>(CuaNativeClientOptions.Bare, nativePath, logLevel, ct);
}
