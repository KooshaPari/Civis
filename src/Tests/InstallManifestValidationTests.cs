// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Task #264 / Pattern #95 — IValidatable + JsonGuard at HIGH cross-FFI/on-disk DTOs.
// Negative tests for InstallManifest at the InstallLifecycle.TryReadManifest deserialize site.

using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using DINOForge.SDK.Validation;
using DINOForge.Tools.Installer;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Pins the <see cref="JsonGuard.ValidateOrThrow{T}"/> wiring at the
    /// <see cref="InstallLifecycle.TryReadManifest(string)"/> deserialize site.
    /// </summary>
    public class InstallManifestValidationTests : IDisposable
    {
        private readonly string _tempGameDir;

        public InstallManifestValidationTests()
        {
            _tempGameDir = Path.Combine(
                Path.GetTempPath(),
                "dinoforge-installmanifest-tests-" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(Path.Combine(_tempGameDir, "BepInEx", "plugins"));
        }

        public void Dispose()
        {
            if (Directory.Exists(_tempGameDir))
            {
                Directory.Delete(_tempGameDir, recursive: true);
            }
        }

        // ── Direct Validate() invariants ────────────────────────────────

        [Fact]
        [Trait("Category", "Validation")]
        public void InstallManifest_EmptyFiles_FailsValidation()
        {
            var manifest = new InstallManifest
            {
                SchemaVersion = "1",
                InstallerVersion = "0.24.0",
                InstalledAtUtc = DateTime.UtcNow.ToString("O"),
                Files = new List<InstalledFileRecord>()
            };

            ValidationResult result = manifest.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "files");
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void InstallManifest_MalformedSha256_FailsValidation()
        {
            var manifest = new InstallManifest
            {
                SchemaVersion = "1",
                Files = new List<InstalledFileRecord>
                {
                    new InstalledFileRecord
                    {
                        RelativePath = "BepInEx/plugins/DINOForge.Runtime.dll",
                        Size = 1234,
                        // Not 64 hex chars — pattern violation.
                        Sha256 = "not-a-hex-digest"
                    }
                }
            };

            ValidationResult result = manifest.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "files[0].sha256");
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void InstallManifest_BlankRelativePath_FailsValidation()
        {
            var manifest = new InstallManifest
            {
                SchemaVersion = "1",
                Files = new List<InstalledFileRecord>
                {
                    new InstalledFileRecord
                    {
                        RelativePath = "",
                        Size = 0,
                        Sha256 = new string('a', 64)
                    }
                }
            };

            ValidationResult result = manifest.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "files[0].relative_path");
        }

        // ── Integration: TryReadManifest swallows InvalidDataException ──

        [Fact]
        [Trait("Category", "Validation")]
        public void TryReadManifest_MalformedManifest_ReturnsNull()
        {
            // Malformed manifest with empty files → InstallManifest.Validate fails →
            // JsonGuard throws → TryReadManifest swallows and returns null (current
            // contract is best-effort, see #147).
            string manifestPath = InstallLifecycle.GetManifestPath(_tempGameDir);
            var malformed = new
            {
                SchemaVersion = "1",
                InstallerVersion = "0.24.0",
                InstalledAtUtc = DateTime.UtcNow.ToString("O"),
                Files = System.Array.Empty<object>()
            };
            File.WriteAllText(manifestPath, JsonSerializer.Serialize(malformed));

            InstallManifest? result = InstallLifecycle.TryReadManifest(_tempGameDir);

            result.Should().BeNull();
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void TryReadManifest_WellFormedManifest_ReturnsParsed()
        {
            string manifestPath = InstallLifecycle.GetManifestPath(_tempGameDir);
            var wellFormed = new InstallManifest
            {
                SchemaVersion = "1",
                InstallerVersion = "0.24.0",
                InstalledAtUtc = DateTime.UtcNow.ToString("O"),
                Files = new List<InstalledFileRecord>
                {
                    new InstalledFileRecord
                    {
                        RelativePath = "BepInEx/plugins/DINOForge.Runtime.dll",
                        Size = 4096,
                        Sha256 = new string('a', 64)
                    }
                }
            };
            File.WriteAllText(manifestPath, JsonSerializer.Serialize(wellFormed));

            InstallManifest? result = InstallLifecycle.TryReadManifest(_tempGameDir);

            result.Should().NotBeNull();
            result!.Files.Should().HaveCount(1);
        }
    }
}
