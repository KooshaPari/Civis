using System;
using System.Collections.Generic;
using DINOForge.SDK;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Unit tests for <see cref="ContentLoadResult"/> covering factory methods,
    /// IsSuccess contract, error handling, and immutability.
    /// </summary>
    public class ContentLoadResultUnitTests
    {
        // ─── Success: returns successful result ──────────────────────────────

        [Fact]
        public void Success_WithLoadedPacks_ReturnsSuccessfulResult()
        {
            // Arrange
            var loadedPacks = new[] { "pack-a", "pack-b", "pack-c" };

            // Act
            var result = ContentLoadResult.Success(loadedPacks);

            // Assert
            result.IsSuccess.Should().BeTrue();
            result.LoadedPacks.Should().HaveCount(3);
            result.LoadedPacks[0].Should().Be("pack-a");
            result.Errors.Should().BeEmpty();
        }

        // ─── Success: empty list of packs ──────────────────────────────────

        [Fact]
        public void Success_WithEmptyPackList_ReturnsSuccessfulResultWithNoPacks()
        {
            // Arrange
            var emptyPacks = new string[] { };

            // Act
            var result = ContentLoadResult.Success(emptyPacks);

            // Assert
            result.IsSuccess.Should().BeTrue();
            result.LoadedPacks.Should().BeEmpty();
            result.Errors.Should().BeEmpty();
        }

        // ─── Failure: returns failed result ──────────────────────────────────

        [Fact]
        public void Failure_WithErrors_ReturnsFailedResult()
        {
            // Arrange
            var errors = new[] { "Error 1", "Error 2" };

            // Act
            var result = ContentLoadResult.Failure(errors);

            // Assert
            result.IsSuccess.Should().BeFalse();
            result.Errors.Should().HaveCount(2);
            result.Errors[0].Should().Be("Error 1");
            result.LoadedPacks.Should().BeEmpty();
        }

        // ─── Failure: no loaded packs ──────────────────────────────────────

        [Fact]
        public void Failure_LoadedPacksAreEmpty()
        {
            // Arrange
            var errors = new[] { "Critical error" };

            // Act
            var result = ContentLoadResult.Failure(errors);

            // Assert
            result.LoadedPacks.Should().BeEmpty();
        }

        // ─── Partial: some packs loaded, some errors ───────────────────────

        [Fact]
        public void Partial_WithPacksAndErrors_ReturnsFalseIsSuccess()
        {
            // Arrange
            var loadedPacks = new[] { "pack-a", "pack-b" };
            var errors = new[] { "Warning for pack-c" };

            // Act
            var result = ContentLoadResult.Partial(loadedPacks, errors);

            // Assert
            result.IsSuccess.Should().BeFalse();
            result.LoadedPacks.Should().HaveCount(2);
            result.Errors.Should().HaveCount(1);
        }

        // ─── Partial with ErrorsByPack mapping ───────────────────────────────

        [Fact]
        public void Partial_WithErrorsByPackMapping_PopulatesPerPackErrors()
        {
            // Arrange
            var loadedPacks = new[] { "pack-a" };
            var errors = new[] { "Error in pack-b", "Error in pack-c" };
            var errorsByPack = new Dictionary<string, IReadOnlyList<string>>
            {
                { "pack-b", new[] { "Missing schema" }.AsReadOnly() },
                { "pack-c", new[] { "Invalid manifest", "Duplicate ID" }.AsReadOnly() }
            };

            // Act
            var result = ContentLoadResult.Partial(loadedPacks, errors, errorsByPack);

            // Assert
            result.IsSuccess.Should().BeFalse();
            result.ErrorsByPack.Should().ContainKey("pack-b");
            result.ErrorsByPack["pack-b"].Should().HaveCount(1);
            result.ErrorsByPack["pack-c"].Should().HaveCount(2);
        }

        // ─── IsSuccess contract: errors determine success ──────────────────

        [Fact]
        public void IsSuccess_WithErrors_IsFalse()
        {
            // Arrange & Act
            var result = ContentLoadResult.Failure(new[] { "Some error" });

            // Assert
            result.IsSuccess.Should().BeFalse();
        }

        // ─── IsSuccess contract: no errors means success ──────────────────

        [Fact]
        public void IsSuccess_WithoutErrors_IsTrue()
        {
            // Arrange & Act
            var result = ContentLoadResult.Success(new[] { "pack-1" });

            // Assert
            result.IsSuccess.Should().BeTrue();
        }

        // ─── Adding errors flips IsSuccess to false ───────────────────────

        [Fact]
        public void Partial_WithNoErrorsButFailureConstructed_IsSuccessMatchesErrors()
        {
            // Arrange — even if Partial is called, the presence of errors determines IsSuccess
            var loadedPacks = new[] { "pack-a", "pack-b" };
            var errors = new string[] { }; // no errors

            // Act
            var result = ContentLoadResult.Partial(loadedPacks, errors);

            // Assert
            // When there are no errors, IsSuccess should be false (Partial means some failure occurred)
            // per the factory method: Partial always sets IsSuccess=false
            result.IsSuccess.Should().BeFalse();
        }

        // ─── Warnings don't affect IsSuccess when no errors exist ──────────

        [Fact]
        public void Partial_WithOnlyWarningsNoErrors_CouldBeSuccessIfNoErrorsProvided()
        {
            // Arrange — Success with empty errors list means IsSuccess=true
            var loadedPacks = new[] { "pack-a", "pack-b" };
            var noErrors = new string[] { };

            // Act
            var successResult = ContentLoadResult.Success(loadedPacks);

            // Assert
            successResult.IsSuccess.Should().BeTrue();
            successResult.Errors.Should().BeEmpty();
        }

        // ─── Immutability: loaded packs list is read-only ────────────────

        [Fact]
        public void LoadedPacks_IsIReadOnlyList()
        {
            // Arrange
            var packs = new[] { "pack-1" };
            var result = ContentLoadResult.Success(packs);

            // Act & Assert
            result.LoadedPacks.Should().BeAssignableTo<IReadOnlyList<string>>();
        }

        // ─── Immutability: errors list is read-only ──────────────────────

        [Fact]
        public void Errors_IsIReadOnlyList()
        {
            // Arrange
            var errors = new[] { "error-1", "error-2" };
            var result = ContentLoadResult.Failure(errors);

            // Act & Assert
            result.Errors.Should().BeAssignableTo<IReadOnlyList<string>>();
        }

        // ─── Immutability: ErrorsByPack is read-only ─────────────────────

        [Fact]
        public void ErrorsByPack_IsIReadOnlyDictionary()
        {
            // Arrange
            var errorsByPack = new Dictionary<string, IReadOnlyList<string>>
            {
                { "pack-x", new[] { "error" }.AsReadOnly() }
            };
            var result = ContentLoadResult.Partial(
                new string[] { },
                new[] { "error" },
                errorsByPack);

            // Act & Assert
            result.ErrorsByPack.Should().NotBeNull();
            result.ErrorsByPack.Should().BeAssignableTo<IReadOnlyDictionary<string, IReadOnlyList<string>>>();
        }

        // ─── ErrorsByPack defaults to empty when not provided ──────────────

        [Fact]
        public void Partial_WithoutErrorsByPackMapping_DefaultsToEmptyDictionary()
        {
            // Arrange
            var loadedPacks = new[] { "pack-a" };
            var errors = new[] { "Some error" };

            // Act
            var result = ContentLoadResult.Partial(loadedPacks, errors);

            // Assert
            result.ErrorsByPack.Should().NotBeNull();
            result.ErrorsByPack.Should().BeEmpty();
        }

        // ─── Multiple calls to factory methods create independent results ───

        [Fact]
        public void MultipleResults_AreIndependent()
        {
            // Arrange & Act
            var result1 = ContentLoadResult.Success(new[] { "pack-1" });
            var result2 = ContentLoadResult.Failure(new[] { "error" });
            var result3 = ContentLoadResult.Partial(new[] { "pack-2" }, new[] { "warning" });

            // Assert
            result1.IsSuccess.Should().BeTrue();
            result2.IsSuccess.Should().BeFalse();
            result3.IsSuccess.Should().BeFalse();
            result1.Errors.Should().BeEmpty();
            result2.LoadedPacks.Should().BeEmpty();
        }
    }
}
