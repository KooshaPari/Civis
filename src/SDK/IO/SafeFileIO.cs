using System;
using System.IO;
using System.Text;

namespace DINOForge.SDK.IO;

/// <summary>
/// Encoding-safe text file IO. UTF-8 with throwOnInvalidBytes — silent mojibake on
/// non-UTF-8 input is a Pattern #106 hazard for pack YAML loaders. Use these helpers
/// instead of raw File.ReadAllText/WriteAllText.
/// </summary>
public static class SafeFileIO
{
    /// <summary>Read text file as UTF-8; throws DecoderFallbackException on invalid bytes.</summary>
    public static string ReadText(string path) => Encoding.UTF8.GetString(File.ReadAllBytes(path));

    /// <summary>Read text file lines as UTF-8; throws on invalid bytes.</summary>
    public static string[] ReadAllLines(string path) => Encoding.UTF8.GetString(File.ReadAllBytes(path)).Split(new[] { "\r\n", "\r", "\n" }, System.StringSplitOptions.None);

    /// <summary>Write text file as UTF-8 (no BOM).</summary>
    public static void WriteText(string path, string content) => File.WriteAllBytes(path, Encoding.UTF8.GetBytes(content));

    /// <summary>Append text file as UTF-8 (no BOM).</summary>
    public static void AppendText(string path, string content) => File.AppendAllText(path, content, new UTF8Encoding(encoderShouldEmitUTF8Identifier: false, throwOnInvalidBytes: true));
}
