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
    /// Tests for <see cref="DisposableFieldNotDisposedAnalyzer"/> (DF1006).
    /// Suppression marker: <c>// disposable-field-ok: &lt;reason&gt;</c>.
    /// Implicit exclusion: classes implementing IDisposable/IAsyncDisposable.
    /// </summary>
    public class DisposableFieldNotDisposedAnalyzerTests
    {
        [Fact]
        public async Task Reports_OnTriggerPattern()
        {
            const string source = @"
using System.Net.Http;
public class C
{
    private HttpClient _client = new HttpClient();
}";
            var diagnostics = await RunAsync(source);
            diagnostics.Should().NotBeEmpty();
            diagnostics[0].Id.Should().Be("DF1006");
        }

        [Fact]
        public async Task DoesNotReport_WhenSuppressionMarkerPresent()
        {
            const string source = @"
using System.Net.Http;
public class C
{
    // disposable-field-ok: shared static client
    private HttpClient _client = new HttpClient();
}";
            var diagnostics = await RunAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task DoesNotReport_WhenClassImplementsIDisposable()
        {
            const string source = @"
using System;
using System.Net.Http;
public class C : IDisposable
{
    private HttpClient _client = new HttpClient();
    public void Dispose() { _client.Dispose(); }
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
                MetadataReference.CreateFromFile(typeof(System.Net.Http.HttpClient).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Linq.Enumerable).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Runtime.CompilerServices.RuntimeHelpers).Assembly.Location),
            };
            var compilation = CSharpCompilation.Create(
                "DF1006Test",
                new[] { tree },
                references,
                new CSharpCompilationOptions(OutputKind.DynamicallyLinkedLibrary));

            var withAnalyzers = compilation.WithAnalyzers(
                ImmutableArray.Create<DiagnosticAnalyzer>(new DisposableFieldNotDisposedAnalyzer()));
            var diagnostics = await withAnalyzers.GetAnalyzerDiagnosticsAsync().ConfigureAwait(false);
            return diagnostics.Where(d => d.Id == "DF1006").ToImmutableArray();
        }
    }
}
