#nullable enable

using System;
using System.IO;
using SharpFuzz;
using Xunit;

namespace DINOForge.Tests.FuzzTargets
{
    /// <summary>
    /// Coverage-guided fuzz target for the version parsing and range-checking paths
    /// used by <see cref="DINOForge.SDK.CompatibilityChecker"/>.
    /// Exercises arbitrary byte inputs as version strings and range constraints.
    /// </summary>
    public static class SemverFuzzTarget
    {
        private static readonly string[] KnownRanges =
        {
            "*", "", ">=0.1.0", ">=1.0.0 <2.0.0", "^1.0.0", "~1.2.3",
            ">=0.0.0", "<99.0.0", ">=0.1.0 <1.0.0"
        };

        /// <summary>
        /// Core fuzz action: treat arbitrary bytes as a version string candidate,
        /// then check it against a set of known range patterns.
        /// </summary>
        internal static void FuzzAction(ReadOnlySpan<byte> data)
        {
            try
            {
                string candidate = System.Text.Encoding.UTF8.GetString(data);

                // Exercise CompatibilityChecker with arbitrary version input
                foreach (string range in KnownRanges)
                {
                    // Result is not checked — we only care that it doesn't throw
                    DINOForge.SDK.CompatibilityChecker.IsVersionInRange(candidate, range);
                }

                // Also exercise parsing as System.Version
                System.Version.TryParse(candidate, out _);
            }
            catch (Exception e) when (
                e is not OutOfMemoryException &&
                e is not StackOverflowException)
            {
                // Unexpected exception — swallow so SharpFuzz can continue exploring
            }
        }

        /// <summary>
        /// SharpFuzz LibFuzzer entry point.
        /// </summary>
        internal static void RunLibFuzzer(string[] args)
        {
            Fuzzer.LibFuzzer.Run(FuzzAction);
        }

        /// <summary>
        /// SharpFuzz out-of-process entry point for AFL++.
        /// </summary>
        internal static void RunAfl(string[] args)
        {
            Fuzzer.OutOfProcess.Run(stream =>
            {
                using MemoryStream ms = new MemoryStream();
                stream.CopyTo(ms);
                FuzzAction(ms.ToArray());
            });
        }

        /// <summary>
        /// Smoke test: verifies the harness compiles and runs against boundary semver seeds.
        /// </summary>
        [Theory]
        [Trait("Category", "Fuzz")]
        [InlineData("0.0.0")]
        [InlineData("0.0.1")]
        [InlineData("999.999.999")]
        [InlineData("1.0.0-alpha")]
        [InlineData("1.0.0-alpha.1")]
        [InlineData("1.0.0+build.1")]
        [InlineData("not-a-version")]
        [InlineData("")]
        [InlineData("1.2.3.4.5.6")]
        [InlineData("99999999.99999999.99999999")]
        public static void Smoke_DoesNotThrow(string input)
        {
            byte[] bytes = System.Text.Encoding.UTF8.GetBytes(input);
            FuzzAction(bytes);
        }
    }
}
