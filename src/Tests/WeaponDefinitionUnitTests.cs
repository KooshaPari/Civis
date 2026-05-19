using DINOForge.SDK.Models;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Unit tests for <see cref="WeaponDefinition"/> Validate() and construction contracts.
    /// </summary>
    public class WeaponDefinitionUnitTests
    {
        [Fact]
        public void Constructor_DefaultValues_InitializesCorrectly()
        {
            // Arrange & Act
            var weapon = new WeaponDefinition();

            // Assert
            weapon.Id.Should().Be("");
            weapon.DisplayName.Should().Be("");
            weapon.WeaponClass.Should().BeNull();
            weapon.DamageType.Should().BeNull();
            weapon.BaseDamage.Should().Be(0f);
            weapon.Range.Should().Be(0f);
            weapon.RateOfFire.Should().Be(1.0f);
        }

        [Fact]
        public void Validate_WithValidData_ReturnsSuccess()
        {
            // Arrange
            var weapon = new WeaponDefinition
            {
                Id = "rifle-a",
                DisplayName = "Standard Rifle",
                BaseDamage = 25f
            };

            // Act
            var result = weapon.Validate();

            // Assert
            result.IsValid.Should().BeTrue();
            result.Errors.Should().BeEmpty();
        }

        [Fact]
        public void Validate_MissingId_ReturnsFailure()
        {
            // Arrange
            var weapon = new WeaponDefinition
            {
                Id = "",
                DisplayName = "Standard Rifle",
                BaseDamage = 25f
            };

            // Act
            var result = weapon.Validate();

            // Assert
            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
            result.Errors[0].Path.Should().Be("id");
            result.Errors[0].Message.Should().Contain("required");
        }

        [Fact]
        public void Validate_MissingDisplayName_ReturnsFailure()
        {
            // Arrange
            var weapon = new WeaponDefinition
            {
                Id = "rifle-a",
                DisplayName = null!,
                BaseDamage = 25f
            };

            // Act
            var result = weapon.Validate();

            // Assert
            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
            result.Errors[0].Path.Should().Be("display_name");
        }

        [Fact]
        public void Validate_WhitespaceId_ReturnsFailure()
        {
            // Arrange
            var weapon = new WeaponDefinition
            {
                Id = "  ",
                DisplayName = "Standard Rifle",
                BaseDamage = 25f
            };

            // Act
            var result = weapon.Validate();

            // Assert
            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "id");
        }

        [Fact]
        public void Validate_NegativeBaseDamage_ReturnsFailure()
        {
            // Arrange
            var weapon = new WeaponDefinition
            {
                Id = "rifle-a",
                DisplayName = "Standard Rifle",
                BaseDamage = -10f
            };

            // Act
            var result = weapon.Validate();

            // Assert
            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
            result.Errors[0].Path.Should().Be("base_damage");
            result.Errors[0].Message.Should().Contain("non-negative");
        }

        [Fact]
        public void Validate_ZeroBaseDamage_ReturnsSuccess()
        {
            // Arrange
            var weapon = new WeaponDefinition
            {
                Id = "placeholder",
                DisplayName = "Placeholder",
                BaseDamage = 0f
            };

            // Act
            var result = weapon.Validate();

            // Assert
            result.IsValid.Should().BeTrue();
        }

        [Fact]
        public void Validate_MultipleErrors_ReturnsAllErrors()
        {
            // Arrange
            var weapon = new WeaponDefinition
            {
                Id = "",
                DisplayName = "",
                BaseDamage = -5f
            };

            // Act
            var result = weapon.Validate();

            // Assert
            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(3); // id, display_name, base_damage
        }

        [Fact]
        public void Validate_WithOptionalFields_ReturnsSuccess()
        {
            // Arrange
            var weapon = new WeaponDefinition
            {
                Id = "cannon-heavy",
                DisplayName = "Heavy Cannon",
                WeaponClass = "cannon",
                DamageType = "explosive",
                BaseDamage = 75f,
                Range = 40f,
                RateOfFire = 0.5f,
                ProjectileId = "projectile-shell",
                AoeRadius = 5f
            };

            // Act
            var result = weapon.Validate();

            // Assert
            result.IsValid.Should().BeTrue();
        }

        [Fact]
        public void Validate_WithHighBaseDamage_ReturnsSuccess()
        {
            // Arrange
            var weapon = new WeaponDefinition
            {
                Id = "nuke",
                DisplayName = "Nuclear Missile",
                BaseDamage = 9999f
            };

            // Act
            var result = weapon.Validate();

            // Assert
            result.IsValid.Should().BeTrue();
        }

        [Fact]
        public void Validate_WithExtendedProperties_ReturnsSuccess()
        {
            // Arrange
            var weapon = new WeaponDefinition
            {
                Id = "missile-aa",
                DisplayName = "Air-to-Air Missile",
                WeaponClass = "missile",
                DamageType = "explosive",
                BaseDamage = 50f,
                Range = 60f,
                RateOfFire = 2.0f,
                AoeRadius = 8f
            };

            // Act
            var result = weapon.Validate();

            // Assert
            result.IsValid.Should().BeTrue();
            result.Errors.Should().BeEmpty();
        }

        [Fact]
        public void Validate_ZeroRateOfFire_ReturnsSuccess()
        {
            // Arrange
            var weapon = new WeaponDefinition
            {
                Id = "slow-cannon",
                DisplayName = "Slow Cannon",
                BaseDamage = 100f,
                RateOfFire = 0f
            };

            // Act
            var result = weapon.Validate();

            // Assert
            result.IsValid.Should().BeTrue();
        }

        [Fact]
        public void Validate_ZeroRange_ReturnsSuccess()
        {
            // Arrange
            var weapon = new WeaponDefinition
            {
                Id = "melee-sword",
                DisplayName = "Sword",
                BaseDamage = 15f,
                Range = 0f
            };

            // Act
            var result = weapon.Validate();

            // Assert
            result.IsValid.Should().BeTrue();
        }
    }
}
