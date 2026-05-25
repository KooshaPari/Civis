namespace DINOForge.Tools.McpServer.Cua;

/// <summary>Per-binary defaults for <see cref="CuaNativeClient"/> JSON-RPC stdio clients.</summary>
public sealed record CuaNativeClientOptions(
    string DefaultExecutableName,
    string LogEnvironmentVariable,
    string ProcessDisplayName)
{
    public static CuaNativeClientOptions Bare { get; } = new("bare-cua-native", "BARE_CUA_LOG", "bare-cua-native");

    public static CuaNativeClientOptions Play { get; } = new("playcua-native", "PLAYCUA_LOG", "playcua-native");
}
