using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Diagnostics.CodeAnalysis;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json;
using System.Threading.Tasks;
using DINOForge.SDK.Dependencies;
using DINOForge.SDK.Json;
using DINOForge.SDK.Validation;

namespace DINOForge.SDK.NativeInterop
{
    /// <summary>
    /// Interop wrapper for Go dependency resolver binary (dinoforge-resolver).
    /// Provides high-performance topological sort and cycle detection for pack dependency graphs.
    ///
    /// **Architecture**:
    /// - Go binary runs as isolated subprocess
    /// - Communication via JSON stdin/stdout
    /// - No tight coupling (easily replaceable)
    /// - Zero cross-language exceptions
    ///
    /// **Fallback**: If Go binary unavailable, delegates to C# PackDependencyResolver.
    ///
    /// **Performance**:
    /// - Go Kahn's algorithm: 5-10x faster than C# recursive version
    /// - Process overhead: ~5-10ms (acceptable for load-time operation)
    /// - Suitable for 50+ pack graphs with deep dependencies
    /// </summary>
    [ExcludeFromCodeCoverage] // Requires Go binary in PATH — integration tests only
    public class GoDependencyResolver : IPackDependencyResolver
    {
        /// <summary>
        /// Path to Go binary (dinoforge-resolver.exe on Windows, dinoforge-resolver on Unix).
        /// Can be configured via environment variable DINOFORGE_RESOLVER_PATH.
        /// </summary>
        private static readonly string ResolverBinaryPath = FindResolverBinary();

        /// <summary>
        /// Check if Go resolver binary is available in PATH or configured location.
        /// </summary>
        public static bool IsAvailable => !string.IsNullOrEmpty(ResolverBinaryPath);

        /// <summary>
        /// Resolve pack dependencies using Go binary (fast) or fallback to C# (safe).
        /// </summary>
        public DependencyResult ResolveDependencies(
            IEnumerable<PackManifest> available,
            PackManifest target)
        {
            if (IsAvailable)
            {
                try
                {
                    return ResolveDependenciesViaGo(available, target);
                }
                catch (Exception ex)
                {
                    System.Diagnostics.Debug.WriteLine($"Go resolver failed, falling back to C#: {ex.Message}");
                    // Fall through to C# path
                }
            }

            // Fallback to C# implementation
            return ResolveDependenciesViaCSharp(available, target);
        }

        /// <summary>
        /// Resolve via Go binary (subprocess model).
        /// </summary>
        private DependencyResult ResolveDependenciesViaGo(
            IEnumerable<PackManifest> available,
            PackManifest target)
        {
            var tempInput = Path.Combine(Path.GetTempPath(), $"dinoforge_resolver_{Guid.NewGuid()}.json");
            var tempOutput = Path.Combine(Path.GetTempPath(), $"dinoforge_resolver_out_{Guid.NewGuid()}.json");

            try
            {
                // Serialize input
                var input = new ResolverInput
                {
                    Available = available.ToList(),
                    Target = target
                };

                var json = JsonSerializer.Serialize(input, JsonOptions.Compact);
                File.WriteAllText(tempInput, json, System.Text.Encoding.UTF8);

                // Invoke Go binary
                var process = new Process
                {
                    StartInfo = new ProcessStartInfo
                    {
                        FileName = ResolverBinaryPath,
                        Arguments = $"--input \"{tempInput}\" --output \"{tempOutput}\"",
                        UseShellExecute = false,
                        RedirectStandardOutput = true,
                        RedirectStandardError = true,
                        CreateNoWindow = true
                    }
                };

                process.Start();
                bool finished = process.WaitForExit(5000); // 5-second timeout

                if (!finished)
                {
                    process.Kill();
                    throw new TimeoutException("Go resolver timed out after 5 seconds");
                }

                if (process.ExitCode != 0)
                {
                    string stderr = process.StandardError.ReadToEnd();
                    throw new InvalidOperationException(
                        $"Go resolver failed with exit code {process.ExitCode}: {stderr}");
                }

                // Deserialize output
                if (!File.Exists(tempOutput))
                    throw new InvalidOperationException("Go resolver produced no output file");

                var outputJson = File.ReadAllText(tempOutput, Encoding.UTF8);
                var output = JsonSerializer.Deserialize<ResolverOutput>(outputJson, JsonOptions.Default)
                    ?? throw new InvalidOperationException("Failed to parse resolver output");

                // Convert to DependencyResult
                if (output.Errors != null && output.Errors.Count > 0)
                    return DependencyResult.Failure(output.Errors);

                if (output.Resolved == null || output.Resolved.Count == 0)
                    return DependencyResult.Failure(new List<string> { "No packs resolved" });

                // Load packs in resolved order
                var packById = available.ToDictionary(p => p.Id, StringComparer.OrdinalIgnoreCase);
                var sorted = new List<PackManifest>();

                foreach (var packId in output.Resolved)
                {
                    if (packById.TryGetValue(packId, out var pack))
                    {
                        sorted.Add(pack);
                    }
                    else
                    {
                        return DependencyResult.Failure(
                            new List<string> { $"Resolver returned unknown pack ID: {packId}" });
                    }
                }

                return DependencyResult.Success(sorted);
            }
            finally
            {
                // Cleanup temp files
                try { if (File.Exists(tempInput)) File.Delete(tempInput); } catch { } // safe-swallow: temp file cleanup
                try { if (File.Exists(tempOutput)) File.Delete(tempOutput); } catch { } // safe-swallow: temp file cleanup
            }
        }

        /// <summary>
        /// Fallback: Resolve via C# PackDependencyResolver.
        /// </summary>
        private DependencyResult ResolveDependenciesViaCSharp(
            IEnumerable<PackManifest> available,
            PackManifest target)
        {
            var csharpResolver = new PackDependencyResolver();
            return csharpResolver.ResolveDependencies(available, target);
        }

        // ===== Helper: Find resolver binary =====

        private static string? FindResolverBinary()
        {
            // 1. Check environment variable
            var envPath = Environment.GetEnvironmentVariable("DINOFORGE_RESOLVER_PATH");
            if (!string.IsNullOrEmpty(envPath) && File.Exists(envPath))
                return envPath;

            // 2. Check relative to runtime (bin/ directory)
            var binDir = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "bin");
            var candidates = new[]
            {
                Path.Combine(binDir, "dinoforge-resolver.exe"),
                Path.Combine(binDir, "dinoforge-resolver"),
                Path.Combine(binDir, "dinoforge_resolver.exe"),
                Path.Combine(binDir, "dinoforge_resolver")
            };

            foreach (var candidate in candidates)
            {
                if (File.Exists(candidate))
                    return candidate;
            }

            // 3. Check PATH
            var pathEnv = Environment.GetEnvironmentVariable("PATH") ?? "";
            foreach (var dir in pathEnv.Split(Path.PathSeparator))
            {
                var candidates2 = new[]
                {
                    Path.Combine(dir, "dinoforge-resolver.exe"),
                    Path.Combine(dir, "dinoforge-resolver"),
                    Path.Combine(dir, "dinoforge_resolver.exe"),
                    Path.Combine(dir, "dinoforge_resolver")
                };

                foreach (var candidate in candidates2)
                {
                    if (File.Exists(candidate))
                        return candidate;
                }
            }

            return null;
        }

        // ===== Input/Output Models =====

        /// <summary>JSON input for Go resolver.</summary>
        private class ResolverInput
        {
            public List<PackManifest> Available { get; set; } // public-mutable-ok: JSON deserializer requires mutable List
            public PackManifest Target { get; set; }
        }

        /// <summary>JSON output from Go resolver.</summary>
        /// <summary>
        /// Single-document JSON output emitted by the <c>dinoforge-resolver</c> Go subprocess.
        /// Internal so tests under <c>InternalsVisibleTo("DINOForge.Tests")</c> can pin the
        /// IValidatable + JsonGuard contract at the FFI boundary (Task #294 / Pattern #95).
        /// </summary>
        internal class ResolverOutput : IValidatable
        {
            /// <summary>Resolved load order (pack IDs).</summary>
            public List<string> Resolved { get; set; } = new List<string>(); // public-mutable-ok: JSON deserializer requires mutable List

            /// <summary>Errors encountered during resolution (e.g. missing deps, cycles).</summary>
            public List<string> Errors { get; set; } = new List<string>(); // public-mutable-ok: JSON deserializer requires mutable List

            /// <inheritdoc />
            public ValidationResult Validate()
            {
                List<ValidationError> errors = new List<ValidationError>();

                bool hasResolved = Resolved != null && Resolved.Count > 0;
                bool hasErrors = Errors != null && Errors.Count > 0;
                if (!hasResolved && !hasErrors)
                {
                    errors.Add(new ValidationError(
                        "resolved|errors",
                        "ResolverOutput must contain at least one resolved entry or one error.",
                        "non_empty"));
                }

                if (Resolved != null)
                {
                    for (int i = 0; i < Resolved.Count; i++)
                    {
                        if (string.IsNullOrWhiteSpace(Resolved[i]))
                            errors.Add(new ValidationError(
                                $"resolved[{i}]",
                                "ResolverOutput 'resolved' entries must be non-blank pack IDs.",
                                "non_empty"));
                    }
                }

                return errors.Count == 0
                    ? ValidationResult.Success()
                    : ValidationResult.Failure(errors.AsReadOnly());
            }
        }
    }

    /// <summary>
    /// <summary>
    /// Interface for dependency resolvers (implemented by both C# and Go interop).
    /// Allows swapping between C# fallback and high-performance Go implementation.
    /// </summary>
    public interface IPackDependencyResolver
    {
        /// <summary>
        /// Resolves dependencies for a target pack against a set of available packs.
        /// Returns a load order that satisfies all transitive dependencies.
        /// </summary>
        /// <param name="available">All packs available for resolution.</param>
        /// <param name="target">The target pack to resolve dependencies for.</param>
        /// <returns>Result containing load order on success, or errors on failure.</returns>
        DependencyResult ResolveDependencies(IEnumerable<PackManifest> available, PackManifest target);
    }
}
