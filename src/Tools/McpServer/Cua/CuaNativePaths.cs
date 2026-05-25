#nullable enable

namespace DINOForge.Tools.McpServer.Cua;

/// <summary>
/// Resolves bare-cua / playcua native binary paths via <see cref="NativeDepResolver"/>.
/// </summary>
internal static class CuaNativePaths
{
    private const string DevBareFallback = @"C:\Users\koosh\bare-cua\target\release\bare-cua-native.exe";

    internal static string? TryFindBare() =>
        NativeDepResolver.TryResolve(
            "bare_cua_native",
            "BARE_CUA_NATIVE",
            [
                Path.Combine(AppContext.BaseDirectory, "bare-cua-native.exe"),
                DevBareFallback,
            ]);

    internal static string? TryFindPlay() =>
        NativeDepResolver.TryResolve(
            "playcua_native",
            "PLAYCUA_NATIVE_EXE",
            [
                Environment.GetEnvironmentVariable("BARE_CUA_NATIVE") ?? string.Empty,
                Path.Combine(AppContext.BaseDirectory, "playcua-native.exe"),
                Path.Combine(AppContext.BaseDirectory, "bare-cua-native.exe"),
                DevBareFallback,
            ]);
}
