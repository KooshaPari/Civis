using System.Collections.Immutable;
using System.Linq;
using System.Threading.Tasks;
using DINOForge.Analyzers;
using FluentAssertions;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.Diagnostics;
using Xunit;

namespace DINOForge.Tests.Analyzers
{
    /// <summary>
    /// Tests for <see cref="SleepBasedTestSyncAnalyzer"/> (DF0108).
    /// Path-scoped: only fires in files whose path contains `/Tests/` or `\\Tests\\`.
    /// Suppression marker: `// test-sleep-ok: &lt;reason&gt;`.
    /// </summary>
    public class SleepBasedTestSyncAnalyzerTests
    {
        private static DiagnosticAnalyzer GetAnalyzer() => new SleepBasedTestSyncAnalyzer();

        [Fact]
        public void DF0108_HasCorrectId()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics.Should().HaveCount(1);
            diagnostics[0].Id.Should().Be("DF0108");
            SleepBasedTestSyncAnalyzer.DiagnosticId.Should().Be("DF0108");
        }

        [Fact]
        public async Task Reports_OnTaskDelayInTestFile()
        {
            const string source = @"
using System.Threading.Tasks;

public class C
{
    public async Task M()
    {
        await Task.Delay(100);
    }
}";
            var diagnostics = await RunAnalyzerAsync(source, @"C:\repo\src\Tests\FooTests.cs");
            diagnostics.Should().ContainSingle()
                .Which.Id.Should().Be("DF0108");
        }

        [Fact]
        public async Task DoesNotReport_WhenSuppressionMarkerPresent()
        {
            // Marker is checked on the invocation's leading trivia. Thread.Sleep
            // is a plain invocation (no await wrapper), so the comment binds to it.
            const string source = @"
using System.Threading;

public class C
{
    public void M()
    {
        // test-sleep-ok: required by external timing constraint
        Thread.Sleep(100);
    }
}";
            var diagnostics = await RunAnalyzerAsync(source, @"C:\repo\src\Tests\FooTests.cs");
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task DoesNotReport_WhenNotInTestFile()
        {
            const string source = @"
using System.Threading.Tasks;

public class C
{
    public async Task M()
    {
        await Task.Delay(100);
    }
}";
            var diagnostics = await RunAnalyzerAsync(source, @"C:\repo\src\Runtime\Foo.cs");
            diagnostics.Should().BeEmpty();
        }

        private static async Task<ImmutableArray<Diagnostic>> RunAnalyzerAsync(string source, string filePath)
        {
            var tree = CSharpSyntaxTree.ParseText(source, path: filePath);
            var references = new[]
            {
                MetadataReference.CreateFromFile(typeof(object).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Linq.Enumerable).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Runtime.CompilerServices.RuntimeHelpers).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Threading.Tasks.Task).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Threading.Thread).Assembly.Location),
            };

            var compilation = CSharpCompilation.Create(
                "DF0108Test",
                new[] { tree },
                references,
                new CSharpCompilationOptions(OutputKind.DynamicallyLinkedLibrary));

            var compilationWithAnalyzers = compilation.WithAnalyzers(
                ImmutableArray.Create<DiagnosticAnalyzer>(new SleepBasedTestSyncAnalyzer()));
            var diagnostics = await compilationWithAnalyzers
                .GetAnalyzerDiagnosticsAsync()
                .ConfigureAwait(false);
            return diagnostics.Where(d => d.Id == "DF0108").ToImmutableArray();
        }
    }
}
