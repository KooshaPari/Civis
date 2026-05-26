#nullable enable

namespace DINOForge.Tools.McpServer.Cua;

/// <summary>
/// Resolves bare-cua / playcua native binary paths via <see cref="NativeDepResolver"/>.
/// </summary>
internal static class CuaNativePaths
{
    internal static string? TryFindBare() =>
        NativeDepResolver.TryResolve(
            "bare_cua_native",
            "BARE_CUA_NATIVE",
            BuildFallbacks("bare-cua-native.exe"));

    internal static string? TryFindPlay() =>
        NativeDepResolver.TryResolve(
            "playcua_native",
            "PLAYCUA_NATIVE_EXE",
            BuildFallbacks("playcua-native.exe", "bare-cua-native.exe"));

    private static string[] BuildFallbacks(params string[] names)
    {
        var paths = new List<string>
        {
            Environment.GetEnvironmentVariable("BARE_CUA_DEV_PATH") ?? string.Empty,
            Environment.GetEnvironmentVariable("PLAYCUA_DEV_PATH") ?? string.Empty,
        };
        foreach (string name in names)
            paths.Add(Path.Combine(AppContext.BaseDirectory, name));
        return paths.ToArray();
    }
}
