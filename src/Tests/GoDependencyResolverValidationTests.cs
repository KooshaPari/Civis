// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Task #264 / Pattern #95 — IValidatable + JsonGuard at HIGH cross-FFI DTOs.
// Negative tests for SDK Go dependency resolver DTO (ResolverOutput).
// Internal type accessed via SDK's InternalsVisibleTo("DINOForge.Tests").

using System.Collections.Generic;
using System.IO;
using DINOForge.SDK.NativeInterop;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Pins the <see cref="JsonGuard.ValidateOrThrow{T}"/> wiring at the
    /// <see cref="GoDependencyResolver"/> deserialize site (single output JSON
    /// emitted by the <c>dinoforge-resolver</c> Go subprocess).
    /// </summary>
    public class GoDependencyResolverValidationTests
    {
        [Fact]
        [Trait("Category", "Validation")]
        public void ResolverOutput_EmptyResolvedAndErrors_FailsValidation()
        {
            // Both Resolved + Errors empty — Go subprocess returned a meaningless payload.
            var output = new GoDependencyResolver.ResolverOutput
            {
                Resolved = new List<string>(),
                Errors = new List<string>()
            };

            ValidationResult result = output.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "resolved|errors");
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void ResolverOutput_BlankResolvedEntry_FailsValidation()
        {
            var output = new GoDependencyResolver.ResolverOutput
            {
                Resolved = new List<string> { "pack-a", "", "pack-c" },
                Errors = new List<string>()
            };

            ValidationResult result = output.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "resolved[1]");
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void ResolverOutput_JsonGuard_EmptyPayload_ThrowsInvalidDataException()
        {
            var output = new GoDependencyResolver.ResolverOutput
            {
                Resolved = new List<string>(),
                Errors = new List<string>()
            };

            System.Action act = () => JsonGuard.ValidateOrThrow(output, "GoDependencyResolverValidationTests");
            act.Should().Throw<InvalidDataException>().WithMessage("*resolved*errors*");
        }
    }
}
