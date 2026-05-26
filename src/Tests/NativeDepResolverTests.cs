using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using DINOForge.Tools.McpServer;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Tests for the unified <see cref="NativeDepResolver"/> precedence chain:
    /// env var -> installer paths.json -> hardcoded fallback -> loud error.
    /// Maps to: Task #197 (P3 INFRA — native-dep + game-path resolver).
    /// </summary>
    [Trait("Category", "Tool")]
    [Collection(EnvVarMutationCollection.Name)]
    public class NativeDepResolverTests : IDisposable
    {
        private const string TestEnvVar = "DINOFORGE_TEST_NATIVE_DEP_VAR";
        private readonly List<string> _tempFiles = new();
        private readonly string? _originalEnvVar;

        public NativeDepResolverTests()
        {
            // Pattern #93: snapshot the original env var BEFORE any test mutates it
            // so Dispose restores the pre-test process-global state (could be set
            // outside this class, e.g. by parent shell or other test runs).
            _originalEnvVar = Environment.GetEnvironmentVariable(TestEnvVar);
        }

        public void Dispose()
        {
            // Restore the snapshot rather than blindly clearing — if the env var
            // was set externally pre-test, we must preserve that.
            Environment.SetEnvironmentVariable(TestEnvVar, _originalEnvVar);
            foreach (var f in _tempFiles)
            {
                try { if (File.Exists(f)) File.Delete(f); } catch { /* best-effort */ }
            }
        }

        private string CreateTempFile(string suffix = ".bin")
        {
            var path = Path.Combine(Path.GetTempPath(), $"dinoforge-resolver-test-{Guid.NewGuid():N}{suffix}");
            File.WriteAllText(path, "test");
            _tempFiles.Add(path);
            return path;
        }

        [Fact]
        public void Resolve_EnvVarPointingToExistingFile_TakesPrecedence()
        {
            var envFile = CreateTempFile();
            var fallbackFile = CreateTempFile();
            Environment.SetEnvironmentVariable(TestEnvVar, envFile);

            var result = NativeDepResolver.Resolve(
                key: "test_key_env_precedence",
                envVar: TestEnvVar,
                hardcodedFallbacks: new[] { fallbackFile },
                description: "test dep");

            result.Should().Be(envFile, "env var should win over hardcoded fallbacks when file exists");
        }

        [Fact]
        public void Resolve_EnvVarPointingToMissingFile_FallsThroughToHardcoded()
        {
            var fallbackFile = CreateTempFile();
            Environment.SetEnvironmentVariable(TestEnvVar, @"C:\definitely\nonexistent\path\to\nowhere.exe");

            var result = NativeDepResolver.Resolve(
                key: "test_key_env_missing",
                envVar: TestEnvVar,
                hardcodedFallbacks: new[] { fallbackFile },
                description: "test dep");

            result.Should().Be(fallbackFile, "missing env-var file should fall through to next tier");
        }

        [Fact]
        public void Resolve_AllProbesMiss_ThrowsLoudFileNotFoundException()
        {
            Environment.SetEnvironmentVariable(TestEnvVar, @"C:\nonexistent\env.exe");
            var bogusFallback = @"C:\nonexistent\fallback.exe";

            Action act = () => NativeDepResolver.Resolve(
                key: "test_key_all_miss",
                envVar: TestEnvVar,
                hardcodedFallbacks: new[] { bogusFallback },
                description: "imaginary binary");

            act.Should()
                .Throw<FileNotFoundException>()
                .Where(ex => ex.Message.Contains("imaginary binary")
                          && ex.Message.Contains(TestEnvVar)
                          && ex.Message.Contains(bogusFallback)
                          && ex.Message.Contains("test_key_all_miss"),
                    "loud error should list every probe attempt and the logical key");
        }

        [Fact]
        public void Resolve_HardcodedFallbackHits_ReturnsThatPath()
        {
            // Env-var unset, paths.json absent (system state) — so we must hit fallback
            Environment.SetEnvironmentVariable(TestEnvVar, null);
            var bogusFallback = @"C:\nonexistent\fallback1.exe";
            var realFallback = CreateTempFile();

            var result = NativeDepResolver.Resolve(
                key: "test_key_fallback_hit_" + Guid.NewGuid().ToString("N"), // unique to dodge any real paths.json
                envVar: TestEnvVar,
                hardcodedFallbacks: new[] { bogusFallback, realFallback },
                description: "test dep");

            result.Should().Be(realFallback, "first existing hardcoded fallback should win when env+installer miss");
        }

        [Fact]
        public void TryResolve_AllMiss_ReturnsNullInsteadOfThrowing()
        {
            Environment.SetEnvironmentVariable(TestEnvVar, null);

            var result = NativeDepResolver.TryResolve(
                key: "test_key_try_miss_" + Guid.NewGuid().ToString("N"),
                envVar: TestEnvVar,
                hardcodedFallbacks: new[] { @"C:\nonexistent\fallback.exe" });

            result.Should().BeNull("TryResolve should swallow the FileNotFoundException");
        }

        [Fact]
        public void Resolve_DirectoryMode_AcceptsDirectoryPath()
        {
            var tempDir = Path.Combine(Path.GetTempPath(), $"dinoforge-resolver-dir-{Guid.NewGuid():N}");
            Directory.CreateDirectory(tempDir);
            try
            {
                Environment.SetEnvironmentVariable(TestEnvVar, tempDir);

                var result = NativeDepResolver.Resolve(
                    key: "test_key_dir_mode",
                    envVar: TestEnvVar,
                    hardcodedFallbacks: Array.Empty<string>(),
                    description: "game install path",
                    requireFile: false);

                result.Should().Be(tempDir);
            }
            finally
            {
                Directory.Delete(tempDir);
            }
        }
    }
}
