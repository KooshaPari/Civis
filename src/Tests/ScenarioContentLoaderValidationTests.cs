// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Task #210 Phase 4 — ScenarioContentLoader JsonGuard wiring negative tests.
// Mirrors PackLoaderTests.cs / UIContentLoaderValidationTests.cs Pattern #75 / Pattern #86 negative-test pattern.

using System;
using System.IO;
using DINOForge.Domains.Scenario;
using DINOForge.Domains.Scenario.Registries;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Pins the JsonGuard.ValidateOrThrow wiring at the ScenarioContentLoader
    /// deserialize site (scenarios). ScenarioContentLoader wraps every load
    /// failure in <see cref="InvalidOperationException"/>; the underlying
    /// validation surface is the <see cref="System.IO.InvalidDataException"/>
    /// carried as InnerException.
    ///
    /// These negative tests enforce that:
    ///   - ScenarioDefinition.Validate() rejects blank id
    ///   - ScenarioDefinition.Validate() rejects blank display_name
    ///   - ScenarioDefinition.Validate() rejects non-positive wave_count
    ///   - ScenarioDefinition.Validate() rejects negative max_duration
    /// at the deserialize site, not later when Register() runs.
    /// </summary>
    public class ScenarioContentLoaderValidationTests : IDisposable
    {
        private readonly string _packDir;
        private readonly ScenarioContentLoader _loader;
        private readonly ScenarioRegistry _scenarioRegistry;

        public ScenarioContentLoaderValidationTests()
        {
            _packDir = Path.Combine(
                Path.GetTempPath(),
                "dinoforge-scenariocontentloader-tests-" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(_packDir);

            _scenarioRegistry = new ScenarioRegistry();
            _loader = new ScenarioContentLoader(_scenarioRegistry);
        }

        public void Dispose()
        {
            try
            {
                if (Directory.Exists(_packDir))
                {
                    Directory.Delete(_packDir, recursive: true);
                }
            }
            catch (IOException)
            {
                // Best-effort cleanup; leave the temp dir if locked by an antivirus etc.
            }
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void ScenarioContentLoader_RejectsScenarioWithBlankId()
        {
            // Arrange — author a scenarios/*.yaml with a blank-id scenario.
            string scenariosDir = Path.Combine(_packDir, "scenarios");
            Directory.CreateDirectory(scenariosDir);
            string yaml = @"
id: ''
display_name: Blank Id Scenario
description: Has empty id
difficulty: Normal
objective_type: Survive
wave_count: 5
max_duration: 600
allowed_factions:
  - faction1
victory_conditions: []
defeat_conditions: []
scripted_events: []
";
            File.WriteAllText(Path.Combine(scenariosDir, "bad-scenario.yaml"), yaml);

            // Act
            Action act = () => _loader.LoadPack(_packDir, "bad-scenario-pack");

            // Assert — ScenarioContentLoader wraps in InvalidOperationException; the
            // semantic violation surfaces as the InnerException with a path-prefixed
            // message that names the offending field.
            act.Should()
                .Throw<InvalidOperationException>()
                .WithInnerException<InvalidDataException>()
                .WithMessage("*id*");

            // Side-effect: nothing should have been registered.
            _scenarioRegistry.Count.Should().Be(0);
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void ScenarioContentLoader_RejectsScenarioWithBlankDisplayName()
        {
            string scenariosDir = Path.Combine(_packDir, "scenarios");
            Directory.CreateDirectory(scenariosDir);
            string yaml = @"
id: blank-display
display_name: ''
description: Has empty display_name
difficulty: Normal
objective_type: Survive
wave_count: 5
max_duration: 600
allowed_factions: []
victory_conditions: []
defeat_conditions: []
scripted_events: []
";
            File.WriteAllText(Path.Combine(scenariosDir, "blank-display.yaml"), yaml);

            Action act = () => _loader.LoadPack(_packDir, "blank-display-pack");

            act.Should()
                .Throw<InvalidOperationException>()
                .WithInnerException<InvalidDataException>()
                .WithMessage("*display_name*");

            _scenarioRegistry.Count.Should().Be(0);
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void ScenarioContentLoader_RejectsScenarioWithNonPositiveWaveCount()
        {
            string scenariosDir = Path.Combine(_packDir, "scenarios");
            Directory.CreateDirectory(scenariosDir);
            string yaml = @"
id: bad-waves
display_name: Bad Waves
description: Has zero wave_count
difficulty: Normal
objective_type: Survive
wave_count: 0
max_duration: 600
allowed_factions: []
victory_conditions: []
defeat_conditions: []
scripted_events: []
";
            File.WriteAllText(Path.Combine(scenariosDir, "bad-waves.yaml"), yaml);

            Action act = () => _loader.LoadPack(_packDir, "bad-waves-pack");

            act.Should()
                .Throw<InvalidOperationException>()
                .WithInnerException<InvalidDataException>()
                .WithMessage("*wave_count*");

            _scenarioRegistry.Count.Should().Be(0);
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void ScenarioContentLoader_RejectsScenarioWithNegativeMaxDuration()
        {
            string scenariosDir = Path.Combine(_packDir, "scenarios");
            Directory.CreateDirectory(scenariosDir);
            string yaml = @"
id: bad-duration
display_name: Bad Duration
description: Has negative max_duration
difficulty: Normal
objective_type: Survive
wave_count: 5
max_duration: -1
allowed_factions: []
victory_conditions: []
defeat_conditions: []
scripted_events: []
";
            File.WriteAllText(Path.Combine(scenariosDir, "bad-duration.yaml"), yaml);

            Action act = () => _loader.LoadPack(_packDir, "bad-duration-pack");

            act.Should()
                .Throw<InvalidOperationException>()
                .WithInnerException<InvalidDataException>()
                .WithMessage("*max_duration*");

            _scenarioRegistry.Count.Should().Be(0);
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void ScenarioContentLoader_RejectsScenarioWithBlankAllowedFaction()
        {
            string scenariosDir = Path.Combine(_packDir, "scenarios");
            Directory.CreateDirectory(scenariosDir);
            string yaml = @"
id: blank-faction
display_name: Blank Faction Entry
description: One blank faction id
difficulty: Normal
objective_type: Survive
wave_count: 5
max_duration: 600
allowed_factions:
  - faction1
  - ''
victory_conditions: []
defeat_conditions: []
scripted_events: []
";
            File.WriteAllText(Path.Combine(scenariosDir, "blank-faction.yaml"), yaml);

            Action act = () => _loader.LoadPack(_packDir, "blank-faction-pack");

            act.Should()
                .Throw<InvalidOperationException>()
                .WithInnerException<InvalidDataException>()
                .WithMessage("*allowed_factions*");

            _scenarioRegistry.Count.Should().Be(0);
        }
    }
}
