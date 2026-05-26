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
    /// Tests for <see cref="LocalTimeLoggingDriftAnalyzer"/> (DF0103).
    /// Fires only when DateTime.Now appears in a "logging context" (variable
    /// named *log*/*timestamp*, method call containing Log/Write, catch
    /// clause, or AppendAllText-like context).
    /// Suppression marker: `// local-time-ok: &lt;reason&gt;`.
    /// </summary>
    public class LocalTimeLoggingDriftAnalyzerTests
    {
        private static DiagnosticAnalyzer GetAnalyzer() => new LocalTimeLoggingDriftAnalyzer();

        [Fact]
        public void DF0103_HasCorrectId()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics.Should().HaveCount(1);
            diagnostics[0].Id.Should().Be("DF0103");
            LocalTimeLoggingDriftAnalyzer.DiagnosticId.Should().Be("DF0103");
        }

        [Fact]
        public async Task Reports_OnDateTimeNowInLoggingContext()
        {
            const string source = @"
using System;

public class C
{
    public void M()
    {
        var timestamp = DateTime.Now;
        Console.WriteLine(timestamp);
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle()
                .Which.Id.Should().Be("DF0103");
        }

        [Fact]
        public async Task DoesNotReport_OnDateTimeUtcNow()
        {
            const string source = @"
using System;

public class C
{
    public void M()
    {
        var timestamp = DateTime.UtcNow;
        Console.WriteLine(timestamp);
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task DoesNotReport_WhenOutsideLoggingContext()
        {
            // Variable not named *log*/*timestamp*, no logging method, no catch — should not fire.
            const string source = @"
using System;

public class C
{
    public void M()
    {
        var x = DateTime.Now;
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
                MetadataReference.CreateFromFile(typeof(System.Console).Assembly.Location),
            };

            var compilation = CSharpCompilation.Create(
                "DF0103Test",
                new[] { tree },
                references,
                new CSharpCompilationOptions(OutputKind.DynamicallyLinkedLibrary));

            var compilationWithAnalyzers = compilation.WithAnalyzers(
                ImmutableArray.Create<DiagnosticAnalyzer>(new LocalTimeLoggingDriftAnalyzer()));
            var diagnostics = await compilationWithAnalyzers
                .GetAnalyzerDiagnosticsAsync()
                .ConfigureAwait(false);
            return diagnostics.Where(d => d.Id == "DF0103").ToImmutableArray();
        }
    }
}
