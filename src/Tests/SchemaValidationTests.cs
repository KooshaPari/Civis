using System.Collections.Generic;
using System.IO;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    public class SchemaValidationTests
    {
        private readonly NJsonSchemaValidator _validator;

        public SchemaValidationTests()
        {
            // Load JSON schemas from the schemas/ directory.
            // NJsonSchemaValidator accepts schema content keyed by name.
            // It converts YAML->JSON internally, but our schemas are already JSON,
            // so we pass them as-is (JSON is valid YAML).
            string schemasDir = FindSchemasDirectory();
            var schemas = new Dictionary<string, string>();

            foreach (string file in Directory.GetFiles(schemasDir, "*.schema.json"))
            {
                string name = Path.GetFileNameWithoutExtension(file)
                    .Replace(".schema", "");
                schemas[name] = File.ReadAllText(file);
            }

            _validator = new NJsonSchemaValidator(schemas);
        }

        [Fact]
        public void ValidManifest_PassesValidation()
        {
            string yaml = @"
id: test-pack
name: Test Pack
version: 0.1.0
author: Test Author
type: content
depends_on: []
conflicts_with: []
";

            ValidationResult result = _validator.Validate("pack-manifest", yaml);

            result.IsValid.Should().BeTrue(
                because: "a valid manifest should pass schema validation. Errors: {0}",
                string.Join("; ", result.Errors));
        }

        [Fact]
        public void InvalidManifest_MissingRequiredFields_FailsValidation()
        {
            // Missing required fields: id, name, version, author, type
            string yaml = @"
description: An incomplete manifest
";

            ValidationResult result = _validator.Validate("pack-manifest", yaml);

            result.IsValid.Should().BeFalse();
            result.Errors.Should().NotBeEmpty();
        }

        [Fact]
        public void ValidUnit_PassesValidation()
        {
            string yaml = @"
id: test-soldier
display_name: Test Soldier
unit_class: CoreLineInfantry
faction_id: test-faction
tier: 2
stats:
  hp: 100
  damage: 15
  armor: 2
  range: 0
  speed: 3.0
  accuracy: 0.7
  fire_rate: 1.0
  morale: 80
  cost:
    food: 30
    wood: 10
    iron: 5
defense_tags:
  - InfantryArmor
  - Biological
behavior_tags:
  - HoldLine
";

            ValidationResult result = _validator.Validate("unit", yaml);

            result.IsValid.Should().BeTrue(
                because: "a valid unit should pass schema validation. Errors: {0}",
                string.Join("; ", result.Errors));
        }

        [Fact]
        public void InvalidUnit_MissingId_FailsValidation()
        {
            string yaml = @"
display_name: No ID Unit
unit_class: CoreLineInfantry
faction_id: test-faction
";

            ValidationResult result = _validator.Validate("unit", yaml);

            result.IsValid.Should().BeFalse();
            result.Errors.Should().NotBeEmpty();
        }

        [Fact]
        public void ValidBuilding_PassesValidation()
        {
            string yaml = @"
id: test-barracks
display_name: Test Barracks
building_type: barracks
cost:
  wood: 50
  stone: 30
health: 500
";

            ValidationResult result = _validator.Validate("building", yaml);

            result.IsValid.Should().BeTrue(
                because: "a valid building should pass schema validation. Errors: {0}",
                string.Join("; ", result.Errors));
        }

        [Fact]
        public void ValidFaction_PassesValidation()
        {
            string yaml = @"
faction:
  id: test-faction
  display_name: Test Faction
  theme: fantasy
  archetype: order
economy:
  gather_bonus: 1.0
army:
  morale_style: disciplined
";

            ValidationResult result = _validator.Validate("faction", yaml);

            result.IsValid.Should().BeTrue(
                because: "a valid faction should pass schema validation. Errors: {0}",
                string.Join("; ", result.Errors));
        }

        // ── Weapon Schema ─────────────────────────────────────────────────

        [Fact]
        public void ValidWeapon_PassesValidation()
        {
            string yaml = @"
id: test-rifle
display_name: Test Rifle
weapon_class: rifle
damage_type: kinetic
base_damage: 20
range: 30
rate_of_fire: 1.5
projectile_id: bullet-1
aoe_radius: 0
";

            ValidationResult result = _validator.Validate("weapon", yaml);
            result.IsValid.Should().BeTrue(
                because: "a valid weapon should pass. Errors: {0}",
                string.Join("; ", result.Errors));
        }

        [Fact]
        public void InvalidWeapon_MissingId_Fails()
        {
            string yaml = @"
display_name: No ID Weapon
";

            ValidationResult result = _validator.Validate("weapon", yaml);
            result.IsValid.Should().BeFalse();
        }

        // ── Projectile Schema ────────────────────────────────────────────

        [Fact]
        public void ValidProjectile_PassesValidation()
        {
            string yaml = @"
id: test-projectile
display_name: Test Projectile
speed: 50
damage: 15
aoe_radius: 0
";

            ValidationResult result = _validator.Validate("projectile", yaml);
            result.IsValid.Should().BeTrue(
                because: "a valid projectile should pass. Errors: {0}",
                string.Join("; ", result.Errors));
        }

        [Fact]
        public void InvalidProjectile_MissingDisplayName_Fails()
        {
            string yaml = @"
id: no-name-projectile
";

            ValidationResult result = _validator.Validate("projectile", yaml);
            result.IsValid.Should().BeFalse();
        }

        // ── Doctrine Schema ──────────────────────────────────────────────

        [Fact]
        public void ValidDoctrine_PassesValidation()
        {
            string yaml = @"
id: test-doctrine
display_name: Test Doctrine
description: A test doctrine
faction_archetype: order
modifiers:
  armor: 1.1
  speed: 0.9
";

            ValidationResult result = _validator.Validate("doctrine", yaml);
            result.IsValid.Should().BeTrue(
                because: "a valid doctrine should pass. Errors: {0}",
                string.Join("; ", result.Errors));
        }

        [Fact]
        public void InvalidDoctrine_MissingId_Fails()
        {
            string yaml = @"
display_name: No ID Doctrine
";

            ValidationResult result = _validator.Validate("doctrine", yaml);
            result.IsValid.Should().BeFalse();
        }

        // ── Skill Schema ────────────────────────────────────────────────

        [Fact]
        public void ValidSkill_PassesValidation()
        {
            string yaml = @"
id: test-heal
display_name: Heal
skill_class: heal
target_type: single_ally
cooldown: 10
range: 5
effects:
  - stat: health
    modifier_type: flat
    value: 50
";

            ValidationResult result = _validator.Validate("skill", yaml);
            result.IsValid.Should().BeTrue(
                because: "a valid skill should pass. Errors: {0}",
                string.Join("; ", result.Errors));
        }

        [Fact]
        public void InvalidSkill_MissingSkillClass_Fails()
        {
            string yaml = @"
id: test-skill
display_name: Missing Class
target_type: self
";

            ValidationResult result = _validator.Validate("skill", yaml);
            result.IsValid.Should().BeFalse();
        }

        // ── Wave Schema ─────────────────────────────────────────────────

        [Fact]
        public void ValidWave_PassesValidation()
        {
            string yaml = @"
id: test-wave-1
display_name: Wave 1
wave_number: 1
delay_seconds: 30
is_final_wave: false
spawn_groups:
  - unit_id: skeleton
    count: 50
    spawn_delay: 0.1
";

            ValidationResult result = _validator.Validate("wave", yaml);
            result.IsValid.Should().BeTrue(
                because: "a valid wave should pass. Errors: {0}",
                string.Join("; ", result.Errors));
        }

        [Fact]
        public void InvalidWave_MissingDisplayName_Fails()
        {
            string yaml = @"
id: bad-wave
";

            ValidationResult result = _validator.Validate("wave", yaml);
            result.IsValid.Should().BeFalse();
        }

        // ── Squad Schema ────────────────────────────────────────────────

        [Fact]
        public void ValidSquad_PassesValidation()
        {
            string yaml = @"
id: test-squad
display_name: Test Squad
unit_id: archer
min_size: 5
max_size: 30
default_formation: line
formation_spacing: 1.5
color_primary: '#FF0000'
behavior_tags:
  - defensive
  - hold_position
";

            ValidationResult result = _validator.Validate("squad", yaml);
            result.IsValid.Should().BeTrue(
                because: "a valid squad should pass. Errors: {0}",
                string.Join("; ", result.Errors));
        }

        [Fact]
        public void InvalidSquad_MissingUnitId_Fails()
        {
            string yaml = @"
id: test-squad
display_name: Test Squad
";

            ValidationResult result = _validator.Validate("squad", yaml);
            result.IsValid.Should().BeFalse();
        }

        // ── Boundary Value Tests ─────────────────────────────────────────

        [Fact]
        public void Unit_EmptyStringId_FailsValidation()
        {
            string yaml = @"
id: ''
display_name: Empty ID
unit_class: CoreLineInfantry
faction_id: test
";

            ValidationResult result = _validator.Validate("unit", yaml);
            result.IsValid.Should().BeFalse();
        }

        [Fact]
        public void Manifest_EmptyStringId_FailsValidation()
        {
            string yaml = @"
id: ''
name: Empty ID Pack
version: 0.1.0
author: Test
type: content
";

            ValidationResult result = _validator.Validate("pack-manifest", yaml);
            result.IsValid.Should().BeFalse();
        }

        [Fact]
        public void Building_ValidMinimal_PassesValidation()
        {
            string yaml = @"
id: minimal-building
display_name: Minimal Building
";

            ValidationResult result = _validator.Validate("building", yaml);
            result.IsValid.Should().BeTrue(
                because: "minimal building should pass. Errors: {0}",
                string.Join("; ", result.Errors));
        }

        /// <summary>
        /// Walks up from the test assembly output directory to find the schemas/ directory.
        /// </summary>
        private static string FindSchemasDirectory()
        {
            // Start from the current directory and walk up to find schemas/
            string? dir = Directory.GetCurrentDirectory();
            while (dir != null)
            {
                string candidate = Path.Combine(dir, "schemas");
                if (Directory.Exists(candidate) && Directory.GetFiles(candidate, "*.schema.json").Length > 0)
                    return candidate;
                dir = Directory.GetParent(dir)?.FullName;
            }

            // Fallback: try relative to the assembly location
            string assemblyDir = Path.GetDirectoryName(typeof(SchemaValidationTests).Assembly.Location) ?? "";
            dir = assemblyDir;
            while (dir != null)
            {
                string candidate = Path.Combine(dir, "schemas");
                if (Directory.Exists(candidate) && Directory.GetFiles(candidate, "*.schema.json").Length > 0)
                    return candidate;
                dir = Directory.GetParent(dir)?.FullName;
            }

            throw new DirectoryNotFoundException(
                "Could not find schemas/ directory. Searched up from: " + Directory.GetCurrentDirectory());
        }
    }
}
