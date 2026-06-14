#nullable enable
using System;
using DINOForge.SDK.Models;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Reqnroll;

namespace DINOForge.Tests.BDD.Steps;

[Binding]
public sealed class TotalConversionManifestValidationSteps
{
    private TotalConversionManifest? _manifest;
    private ValidationResult? _validationResult;

    [Given(@"a total conversion manifest with type ""(.*)""")]
    public void GivenATotalConversionManifestWithType(string type)
    {
        _manifest = new TotalConversionManifest
        {
            Id = "sample-total-conversion",
            Name = "Sample Total Conversion",
            Version = "1.0.0",
            Type = type,
            FrameworkVersion = ">=0.1.0 <1.0.0"
        };
    }

    [When(@"I validate the manifest")]
    public void WhenIValidateTheManifest()
    {
        _validationResult = _manifest?.Validate();
    }

    [Then(@"validation should fail with a type error")]
    public void ThenValidationShouldFailWithATypeError()
    {
        _validationResult.Should().NotBeNull();
        _validationResult!.IsValid.Should().BeFalse();
        _validationResult.Errors.Should().ContainSingle(error =>
            error.Path == "type" &&
            error.Message.Contains("total_conversion", StringComparison.Ordinal));
    }
}
