#nullable enable
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.Threading.Tasks;
using DINOForge.Tools.PackCompiler.Models;
using DINOForge.SDK;
using System.Diagnostics.CodeAnalysis;

namespace DINOForge.Tools.PackCompiler.Services
{
    /// <summary>
    /// Dependency resolver using Go binary for fast topological pack resolution.
    /// Falls back to C# implementation if Go binary is unavailable.
    /// </summary>
    public class GoResolverService
    {
        private readonly string _resolverPath;
        private readonly bool _useGoResolver;
        private static readonly JsonSerializerOptions JsonOptions = new()
        {
            PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower,
            WriteIndented = true
        };

        public GoResolverService(string? resolverPath = null)
        {
            _resolverPath = resolverPath ?? GetDefaultResolverPath();
            _useGoResolver = File.Exists(_resolverPath);

            if (!_useGoResolver)
            {
                Console.WriteLine($"[WARNING] Go resolver not found at {_resolverPath}. Using C# fallback.");
            }
        }

        /// <summary>
        /// Resolve pack dependencies using Go binary if available, otherwise C# fallback.
        /// </summary>
        public async Task<List<string>> ResolveDependenciesAsync(
            List<PackManifest> available,
            PackManifest target)
        {
            if (_useGoResolver)
            {
                try
                {
                    return await ResolveWithGoAsync(available, target);
                }
                catch (Exception ex)
                {
                    Console.WriteLine($"[WARNING] Go resolver failed: {ex.Message}. Falling back to C#.");
                }
            }

            return ResolveWithCSharp(available, target);
        }

        /// <summary>
        /// Invoke Go binary to resolve dependencies.
        /// Writes JSON input to temp file, invokes binary, reads JSON output.
        /// </summary>
        [RequiresUnreferencedCode("Calls System.Text.Json.JsonSerializer.Serialize<TValue>(TValue, JsonSerializerOptions)")]
        private async Task<List<string>> ResolveWithGoAsync(
            List<PackManifest> available,
            PackManifest target)
        {
            var input = new ResolverInput
            {
                Available = available,
                Target = target
            };

            // Write input to temp file
            string tempInputPath = Path.Combine(Path.GetTempPath(), $"dinoforge_resolver_{Guid.NewGuid()}.json");
            string tempOutputPath = Path.Combine(Path.GetTempPath(), $"dinoforge_resolver_out_{Guid.NewGuid()}.json");

            try
            {
                string inputJson = JsonSerializer.Serialize(input, JsonOptions);
                await File.WriteAllTextAsync(tempInputPath, inputJson);

                // Invoke Go binary
                var psi = new ProcessStartInfo
                {
                    FileName = _resolverPath,
                    Arguments = $"--input \"{tempInputPath}\" --output \"{tempOutputPath}\"",
                    RedirectStandardOutput = true,
                    RedirectStandardError = true,
                    UseShellExecute = false,
                    CreateNoWindow = true
                };

                using (var process = Process.Start(psi))
                {
                    if (process == null)
                        throw new InvalidOperationException($"Failed to start Go resolver: {_resolverPath}");

                    bool completedInTime = process.WaitForExit(30000); // 30 second timeout

                    if (!completedInTime)
                    {
                        process.Kill();
                        throw new TimeoutException("Go resolver exceeded 30 second timeout");
                    }

                    if (process.ExitCode != 0)
                    {
                        string stderr = await process.StandardError.ReadToEndAsync();
                        throw new InvalidOperationException($"Go resolver failed with exit code {process.ExitCode}: {stderr}");
                    }
                }

                // Read output
                string outputJson = await File.ReadAllTextAsync(tempOutputPath);
                var output = JsonSerializer.Deserialize<ResolverOutput>(outputJson, JsonOptions)
                    ?? throw new InvalidOperationException("Failed to deserialize Go resolver output");

                if (output.Errors?.Any() == true)
                {
                    throw new InvalidOperationException(
                        $"Go resolver reported errors:\n{string.Join("\n", output.Errors)}"
                    );
                }

                return output.Resolved ?? new List<string>();
            }
            finally
            {
                // Clean up temp files
                try { if (File.Exists(tempInputPath)) File.Delete(tempInputPath); } catch { }
                try { if (File.Exists(tempOutputPath)) File.Delete(tempOutputPath); } catch { }
            }
        }

        /// <summary>
        /// Pure C# topological sort (Kahn's algorithm) for pack dependencies.
        /// Used as fallback when Go binary is unavailable.
        /// </summary>
        private static List<string> ResolveWithCSharp(
            List<PackManifest> available,
            PackManifest target)
        {
            // Check target exists in available
            var byId = available.ToDictionary(p => p.Id);
            if (!byId.ContainsKey(target.Id))
                throw new ArgumentException($"Target pack not found in available packs: {target.Id}");

            // Check all dependencies exist
            foreach (var dep in target.DependsOn)
            {
                if (!byId.ContainsKey(dep))
                    throw new ArgumentException($"Pack '{target.Id}' requires missing dependency: '{dep}'");
            }

            // Build in-degree map (number of dependencies for each pack)
            var inDegree = new Dictionary<string, int>();
            var dependents = new Dictionary<string, List<string>>();

            foreach (var pack in available)
            {
                inDegree[pack.Id] = 0;
                dependents[pack.Id] = new List<string>();
            }

            // Build dependency graph
            foreach (var pack in available)
            {
                foreach (var dep in pack.DependsOn)
                {
                    if (byId.ContainsKey(dep))
                    {
                        // dep must come before pack
                        dependents[dep].Add(pack.Id);
                        inDegree[pack.Id]++;
                    }
                }
            }

            // Kahn's algorithm
            var ready = available
                .Where(p => inDegree[p.Id] == 0)
                .OrderBy(p => p.LoadOrder)
                .ToList();

            var resolved = new List<string>();
            while (ready.Count > 0)
            {
                // Pop first (lowest LoadOrder)
                var current = ready[0];
                ready.RemoveAt(0);
                resolved.Add(current.Id);

                // Reduce in-degree for dependents
                foreach (var depId in dependents[current.Id])
                {
                    inDegree[depId]--;
                    if (inDegree[depId] == 0)
                    {
                        var depPack = byId[depId];
                        // Insert in sorted order
                        int insertIdx = ready.FindIndex(p => depPack.LoadOrder < p.LoadOrder);
                        if (insertIdx < 0)
                            ready.Add(depPack);
                        else
                            ready.Insert(insertIdx, depPack);
                    }
                }
            }

            // Check for cycles
            if (resolved.Count != available.Count)
                throw new InvalidOperationException("Circular dependency detected among packs");

            return resolved;
        }

        private static string GetDefaultResolverPath()
        {
            // Try to find Go resolver in common locations
            var candidates = new[]
            {
                Path.Combine(AppContext.BaseDirectory, "dinoforge-resolver.exe"),
                Path.Combine(AppContext.BaseDirectory, "dinoforge-resolver"),
                Path.Combine(Path.GetTempPath(), "dinoforge-resolver.exe"),
                Path.Combine(Path.GetTempPath(), "dinoforge-resolver"),
                Environment.GetEnvironmentVariable("DINOFORGE_GO_RESOLVER") ?? ""
            };

            return candidates.FirstOrDefault(p => !string.IsNullOrEmpty(p) && File.Exists(p))
                ?? Path.Combine(AppContext.BaseDirectory, "dinoforge-resolver");
        }

        #region JSON Serialization Models

        private class ResolverInput
        {
            [JsonPropertyName("available")]
            public List<PackManifest> Available { get; set; } = new();

            [JsonPropertyName("target")]
            public PackManifest Target { get; set; } = new();
        }

        private class ResolverOutput
        {
            [JsonPropertyName("resolved")]
            public List<string> Resolved { get; set; } = new();

            [JsonPropertyName("errors")]
            public List<string> Errors { get; set; } = new();
        }

        #endregion
    }
}
