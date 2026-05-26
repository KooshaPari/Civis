#nullable enable
using BareCua;
using DINOForge.Tools.McpServer.PlayCua;

namespace DINOForge.Tools.McpServer.Cua;

/// <summary>
/// Shared start/dispose lifecycle for bare-cua and playcua native JSON-RPC clients.
/// </summary>
internal static class CuaNativeSession
{
    internal static async Task<bool> TryScreenshotAsync(
        CuaNativeClientOptions options,
        string? nativePath,
        string windowTitle,
        string outputPath,
        CancellationToken ct = default)
    {
        if (string.IsNullOrEmpty(nativePath) || !File.Exists(nativePath))
            return false;

        try
        {
            CuaNativeClient computer = await StartClientAsync(options, nativePath, "warn", ct).ConfigureAwait(false);
            try
            {
                byte[] pngBytes = await computer.ScreenshotAsync(windowTitle: windowTitle, ct: ct).ConfigureAwait(false);
                if (pngBytes.Length == 0)
                    return false;

                await File.WriteAllBytesAsync(outputPath, pngBytes, ct).ConfigureAwait(false);
                return File.Exists(outputPath) && new FileInfo(outputPath).Length > 1000;
            }
            finally
            {
                await computer.DisposeAsync().ConfigureAwait(false);
            }
        }
        catch
        {
            return false;
        }
    }

    internal static async Task<bool> TryPressKeyAsync(
        CuaNativeClientOptions options,
        string? nativePath,
        string keyName,
        CancellationToken ct = default)
    {
        if (string.IsNullOrEmpty(nativePath) || !File.Exists(nativePath))
            return false;

        try
        {
            CuaNativeClient computer = await StartClientAsync(options, nativePath, "warn", ct).ConfigureAwait(false);
            try
            {
                await computer.PressKeyAsync(keyName, ct).ConfigureAwait(false);
                return true;
            }
            finally
            {
                await computer.DisposeAsync().ConfigureAwait(false);
            }
        }
        catch
        {
            return false;
        }
    }

    private static async Task<CuaNativeClient> StartClientAsync(
        CuaNativeClientOptions options,
        string nativePath,
        string logLevel,
        CancellationToken ct)
    {
        if (options == CuaNativeClientOptions.Play)
            return await PlayCua.NativeComputer.StartAsync(nativePath, logLevel, ct).ConfigureAwait(false);

        return await BareCua.NativeComputer.StartAsync(nativePath, logLevel, ct).ConfigureAwait(false);
    }
}
