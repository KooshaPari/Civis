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
    /// Tests for <see cref="OrphanProcessHandleAnalyzer"/> (DF0102).
    /// Analyzer does NOT support a suppression marker; the negative-case
    /// surface is "wrapped in using" or "assigned to a variable".
    /// </summary>
    public class OrphanProcessHandleAnalyzerTests
    {
        private static DiagnosticAnalyzer GetAnalyzer() => new OrphanProcessHandleAnalyzer();

        [Fact]
        public void DF0102_HasCorrectId()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics.Should().HaveCount(1);
            diagnostics[0].Id.Should().Be("DF0102");
            OrphanProcessHandleAnalyzer.DiagnosticId.Should().Be("DF0102");
        }

        [Fact]
        public async Task Reports_OnOrphanProcessStart()
        {
            const string source = @"
using System.Diagnostics;

public class C
{
    public void M()
    {
        Process.Start(""notepad.exe"");
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle()
                .Which.Id.Should().Be("DF0102");
        }

        [Fact]
        public async Task DoesNotReport_WhenWrappedInUsing()
        {
            const string source = @"
using System.Diagnostics;

public class C
{
    public void M()
    {
        using var p = Process.Start(""notepad.exe"");
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task DoesNotReport_WhenAssignedToVariable()
        {
            const string source = @"
using System.Diagnostics;

public class C
{
    public void M()
    {
        var p = Process.Start(""notepad.exe"");
        p?.Dispose();
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        private static async Task<ImmutableArray<Diagnostic>> RunAnalyzerAsync(string source)
        {
            var tree = CSharpSyntaxTree.ParseText(source);
            var references = new[]
            {
                MetadataReference.CreateFromFile(typeof(object).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Linq.Enumerable).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Runtime.CompilerServices.RuntimeHelpers).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Diagnostics.Process).Assembly.Location),
            };

            var compilation = CSharpCompilation.Create(
                "DF0102Test",
                new[] { tree },
                references,
                new CSharpCompilationOptions(OutputKind.DynamicallyLinkedLibrary));

            var compilationWithAnalyzers = compilation.WithAnalyzers(
                ImmutableArray.Create<DiagnosticAnalyzer>(new OrphanProcessHandleAnalyzer()));
            var diagnostics = await compilationWithAnalyzers
                .GetAnalyzerDiagnosticsAsync()
                .ConfigureAwait(false);
            return diagnostics.Where(d => d.Id == "DF0102").ToImmutableArray();
        }
    }
}
