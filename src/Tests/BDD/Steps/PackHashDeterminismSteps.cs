#nullable enable
using System;
using System.IO;
using DINOForge.SDK.Signing;
using FluentAssertions;
using Reqnroll;

namespace DINOForge.Tests.BDD.Steps;

[Binding]
public sealed class PackHashDeterminismSteps
{
    private string? _packDirectory;
    private string? _baselineHash;
    private string? _recomputedHash;

    [Given(@"a pack directory with content files")]
    public void GivenAPackDirectoryWithContentFiles()
    {
        _packDirectory = Path.Combine(Path.GetTempPath(), "dinoforge-pack-hash-" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(Path.Combine(_packDirectory, "content"));

        File.WriteAllText(Path.Combine(_packDirectory, "content", "alpha.txt"), "alpha");
        File.WriteAllText(Path.Combine(_packDirectory, "content", "beta.txt"), "beta");
    }

    [When(@"I compute the baseline pack hash")]
    public void WhenIComputeTheBaselinePackHash()
    {
        _baselineHash = PackSigner.ComputePackHash(_packDirectory!);
    }

    [When(@"I add signature artifacts to the pack directory")]
    public void WhenIAddSignatureArtifactsToThePackDirectory()
    {
        File.WriteAllText(Path.Combine(_packDirectory!, "pack.signature"), "ignored-signature");
        File.WriteAllText(Path.Combine(_packDirectory!, "pack.publickey"), "ignored-public-key");
    }

    [When(@"I recompute the pack hash")]
    public void WhenIRecomputeThePackHash()
    {
        _recomputedHash = PackSigner.ComputePackHash(_packDirectory!);
    }

    [Then(@"the pack hash should stay the same")]
    public void ThenThePackHashShouldStayTheSame()
    {
        _baselineHash.Should().NotBeNull();
        _recomputedHash.Should().NotBeNull();
        _recomputedHash.Should().Be(_baselineHash);
    }

    [AfterScenario]
    public void AfterScenario()
    {
        if (_packDirectory != null && Directory.Exists(_packDirectory))
        {
            Directory.Delete(_packDirectory, recursive: true);
        }
    }
}
