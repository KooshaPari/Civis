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
    private static readonly UTF8Encoding StrictUtf8 = new UTF8Encoding(encoderShouldEmitUTF8Identifier: false, throwOnInvalidBytes: true);
    private const char Utf8Bom = '\uFEFF';

    /// <summary>Read text file as UTF-8; throws DecoderFallbackException on invalid bytes.</summary>
    public static string ReadText(string path) => StripBom(StrictUtf8.GetString(File.ReadAllBytes(path)));

    /// <summary>Read text file lines as UTF-8; throws on invalid bytes.</summary>
    public static string[] ReadAllLines(string path) => StripBom(StrictUtf8.GetString(File.ReadAllBytes(path))).Split(new[] { "\r\n", "\r", "\n" }, System.StringSplitOptions.None);

    /// <summary>Write text file as UTF-8 (no BOM).</summary>
    public static void WriteText(string path, string content) => File.WriteAllBytes(path, StrictUtf8.GetBytes(content));

    /// <summary>Append text file as UTF-8 (no BOM).</summary>
    public static void AppendText(string path, string content) => File.AppendAllText(path, content, StrictUtf8);

    private static string StripBom(string text) => text.Length > 0 && text[0] == Utf8Bom ? text.Substring(1) : text;
}
