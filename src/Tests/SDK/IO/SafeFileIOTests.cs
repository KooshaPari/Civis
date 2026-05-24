// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Pattern #106 — SafeFileIO encoding-safe text IO.

using System;
using System.IO;
using System.Text;
using DINOForge.SDK.IO;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.SDK.IO
{
    [Trait("Category", "IO")]
    public class SafeFileIOTests
    {
        [Fact]
        public void ReadText_ValidUtf8_ReadsContent()
        {
            string path = Path.Combine(Path.GetTempPath(), $"safefileio_valid_{Guid.NewGuid():N}.txt");
            try
            {
                // Write valid UTF-8 (multi-byte chars: é U+00E9, 中 U+4E2D)
                File.WriteAllBytes(path, new UTF8Encoding(false).GetBytes("hello é 中"));

                string content = SafeFileIO.ReadText(path);

                content.Should().Be("hello é 中");
            }
            finally
            {
                if (File.Exists(path)) File.Delete(path);
            }
        }

        [Fact]
        public void ReadText_Utf8Bom_StripsBom()
        {
            string path = Path.Combine(Path.GetTempPath(), $"safefileio_bom_{Guid.NewGuid():N}.txt");
            try
            {
                File.WriteAllText(path, "id: test", Encoding.UTF8);

                string content = SafeFileIO.ReadText(path);

                content.Should().Be("id: test");
                content.Should().NotStartWith("\uFEFF");
            }
            finally
            {
                if (File.Exists(path)) File.Delete(path);
            }
        }

        [Fact]
        public void ReadText_InvalidUtf8_Throws()
        {
            string path = Path.Combine(Path.GetTempPath(), $"safefileio_invalid_{Guid.NewGuid():N}.txt");
            try
            {
                // 0xC3 0x28 is an invalid UTF-8 sequence (lone continuation byte after lead byte).
                // 0xFF is never valid as a UTF-8 start byte.
                File.WriteAllBytes(path, new byte[] { 0xC3, 0x28, 0xFF, 0xFE, 0xFD });

                Action act = () => SafeFileIO.ReadText(path);

                // StrictUtf8 (throwOnInvalidBytes: true) raises DecoderFallbackException via
                // System.Text.DecoderExceptionFallback. The IO layer surfaces this directly.
                act.Should().Throw<System.Text.DecoderFallbackException>();
            }
            finally
            {
                if (File.Exists(path)) File.Delete(path);
            }
        }
    }
}
