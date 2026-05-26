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
    /// Tests for <see cref="AsyncVoidAnalyzer"/> (DF1005).
    /// Suppression marker: <c>// async-void-ok: &lt;reason&gt;</c>.
    /// Implicit exclusion: methods matching the (object, EventArgs) event-handler pattern.
    /// </summary>
    public class AsyncVoidAnalyzerTests
    {
        [Fact]
        public async Task Reports_OnTriggerPattern()
        {
            const string source = @"
using System.Threading.Tasks;
public class C
{
    public async void M()
    {
        await Task.Delay(1);
    }
}";
            var diagnostics = await RunAsync(source);
            diagnostics.Should().ContainSingle().Which.Id.Should().Be("DF1005");
        }

        [Fact]
        public async Task DoesNotReport_WhenSuppressionMarkerPresent()
        {
            const string source = @"
using System.Threading.Tasks;
public class C
{
    // async-void-ok: legitimate fire-and-forget
    public async void M()
    {
        await Task.Delay(1);
    }
}";
            var diagnostics = await RunAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task DoesNotReport_WhenEventHandlerSignature()
        {
            const string source = @"
using System;
using System.Threading.Tasks;
public class C
{
    public async void OnClick(object sender, EventArgs e)
    {
        await Task.Delay(1);
    }
}";
            var diagnostics = await RunAsync(source);
            diagnostics.Should().BeEmpty();
        }

        private static async Task<ImmutableArray<Diagnostic>> RunAsync(string source)
        {
            var tree = CSharpSyntaxTree.ParseText(source);
            var references = new[]
            {
                MetadataReference.CreateFromFile(typeof(object).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(Task).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Linq.Enumerable).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Runtime.CompilerServices.RuntimeHelpers).Assembly.Location),
            };
            var compilation = CSharpCompilation.Create(
                "DF1005Test",
                new[] { tree },
                references,
                new CSharpCompilationOptions(OutputKind.DynamicallyLinkedLibrary));

            var withAnalyzers = compilation.WithAnalyzers(
                ImmutableArray.Create<DiagnosticAnalyzer>(new AsyncVoidAnalyzer()));
            var diagnostics = await withAnalyzers.GetAnalyzerDiagnosticsAsync().ConfigureAwait(false);
            return diagnostics.Where(d => d.Id == "DF1005").ToImmutableArray();
        }
    }
}
