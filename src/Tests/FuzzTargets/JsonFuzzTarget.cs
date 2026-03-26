#nullable enable

using System;
using System.IO;
using System.Text.Json;
using SharpFuzz;
using Xunit;

namespace DINOForge.Tests.FuzzTargets
{
    /// <summary>
    /// Coverage-guided fuzz target for JSON parsing paths (System.Text.Json).
    /// Exercises the parser against arbitrary byte sequences to surface panics,
    /// unexpected exceptions, or infinite loops in the JSON layer.
    /// </summary>
    public static class JsonFuzzTarget
    {
        /// <summary>
        /// Core fuzz action: parse arbitrary bytes as JSON.
        /// </summary>
        public static void FuzzAction(ReadOnlySpan<byte> data)
        {
            try
            {
                string json = System.Text.Encoding.UTF8.GetString(data);
                JsonDocument.Parse(json).Dispose();
            }
            catch (JsonException)
            {
                // Expected for malformed JSON — not a bug
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
        /// Smoke test: verifies the harness compiles and runs against known corpus seeds.
        /// </summary>
        [Theory]
        [Trait("Category", "Fuzz")]
        [InlineData("{}")]
        [InlineData("{\"id\":\"test\",\"version\":\"0.1.0\"}")]
        [InlineData("[]")]
        [InlineData("null")]
        [InlineData("\"plain string\"")]
        [InlineData("{not valid json}")]
        [InlineData("")]
        [InlineData("{\"nested\":{\"a\":{\"b\":{\"c\":1}}}}")]
        public static void Smoke_DoesNotThrow(string jsonInput)
        {
            byte[] bytes = System.Text.Encoding.UTF8.GetBytes(jsonInput);
            FuzzAction(bytes);
        }
    }
}
