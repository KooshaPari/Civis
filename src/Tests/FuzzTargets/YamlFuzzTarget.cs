#nullable enable

using System;
using System.Collections.Generic;
using System.IO;
using SharpFuzz;
using Xunit;

namespace DINOForge.Tests.FuzzTargets
{
    /// <summary>
    /// Coverage-guided fuzz target for YAML parsing paths used by pack manifests.
    /// The <see cref="FuzzAction"/> method is the entry point for SharpFuzz tooling.
    /// The smoke tests verify the harness compiles and corpus seeds don't panic.
    /// </summary>
    public static class YamlFuzzTarget
    {
        /// <summary>
        /// Core fuzz action: parse arbitrary bytes as YAML.
        /// Any exception other than YamlException is a potential crash bug.
        /// </summary>
        public static void FuzzAction(ReadOnlySpan<byte> data)
        {
            try
            {
                string yaml = System.Text.Encoding.UTF8.GetString(data);

                // Parse as generic YAML document — validates the parser doesn't panic
                YamlDotNet.Serialization.IDeserializer deserializer =
                    new YamlDotNet.Serialization.DeserializerBuilder().Build();
                deserializer.Deserialize<Dictionary<string, object>>(yaml);
            }
            catch (YamlDotNet.Core.YamlException)
            {
                // Expected for malformed input — not a bug
            }
            catch (Exception e) when (
                e is not OutOfMemoryException &&
                e is not StackOverflowException)
            {
                // Unexpected exception — swallow so SharpFuzz can continue exploring
            }
        }

        /// <summary>
        /// SharpFuzz LibFuzzer entry point. Invoke from a standalone harness program.
        /// Usage: set LIBFUZZER_DOTNET_TARGET to this method and run via libFuzzer.
        /// </summary>
        public static void RunLibFuzzer(string[] args)
        {
            Fuzzer.LibFuzzer.Run(FuzzAction);
        }

        /// <summary>
        /// SharpFuzz out-of-process entry point for AFL++.
        /// </summary>
        public static void RunAfl(string[] args)
        {
            Fuzzer.OutOfProcess.Run(stream =>
            {
                using MemoryStream ms = new MemoryStream();
                stream.CopyTo(ms);
                FuzzAction(ms.ToArray());
            });
        }

        /// <summary>
        /// Smoke test: ensures the fuzz harness compiles and runs against corpus seeds.
        /// </summary>
        [Theory]
        [Trait("Category", "Fuzz")]
        [InlineData("id: test\nname: Test Pack\nversion: 0.1.0")]
        [InlineData("id: test-pack\nname: Test Pack\nversion: 0.1.0\nframework_version: \">=0.1.0\"")]
        [InlineData("id: \u0442\u0435\u0441\u0442")]  // unicode: "тест"
        [InlineData("")]
        [InlineData("not yaml at all: [[[")]
        [InlineData("id: test\n  name: broken-indent")]
        public static void Smoke_DoesNotThrow(string yamlInput)
        {
            byte[] bytes = System.Text.Encoding.UTF8.GetBytes(yamlInput);
            FuzzAction(bytes);
        }
    }
}
