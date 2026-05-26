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
    /// Tests for <see cref="StaticMutableCollectionAnalyzer"/> (DF1001).
    /// Detects static mutable collection fields (List, Dictionary, HashSet, etc.) that
    /// are not readonly, which risk thread-safety races.
    /// Suppression marker: <c>// static-mutable-ok: &lt;reason&gt;</c>
    /// </summary>
    public class StaticMutableCollectionAnalyzerTests
    {
        [Fact]
        public void HasCorrectId()
        {
            var analyzer = new StaticMutableCollectionAnalyzer();
            analyzer.SupportedDiagnostics.Should().ContainSingle();
            analyzer.SupportedDiagnostics[0].Id.Should().Be("DF1001");
        }

        [Fact]
        public void HasExpectedSeverity()
        {
            var analyzer = new StaticMutableCollectionAnalyzer();
            analyzer.SupportedDiagnostics[0].DefaultSeverity.Should().Be(DiagnosticSeverity.Warning);
            analyzer.SupportedDiagnostics[0].IsEnabledByDefault.Should().BeTrue();
        }

        [Fact]
        public void HasExpectedCategory()
        {
            var analyzer = new StaticMutableCollectionAnalyzer();
            analyzer.SupportedDiagnostics[0].Category.Should().Be("Concurrency");
        }

        [Fact]
        public async Task DetectsStaticMutableDictionary()
        {
            // Static mutable Dictionary field without readonly is flagged.
            const string source = @"
using System.Collections.Generic;
public class Cache
{
    static Dictionary<string, int> _entries = new Dictionary<string, int>();
}";
            var diagnostics = await RunAsync(source);
            diagnostics.Should().NotBeEmpty();
            diagnostics[0].Id.Should().Be("DF1001");
        }

        [Fact]
        public async Task IgnoresStaticReadonlyCollection()
        {
            // Static readonly collection is OK (immutable reference).
            const string source = @"
using System.Collections.Generic;
public class Cache
{
    static readonly Dictionary<string, int> _entries = new Dictionary<string, int>();
}";
            var diagnostics = await RunAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task HonorsSuppressionMarker()
        {
            const string source = @"
using System.Collections.Generic;
public class Cache
{
    // static-mutable-ok: single-threaded init only
    static Dictionary<string, int> _entries = new Dictionary<string, int>();
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
                MetadataReference.CreateFromFile(typeof(System.Linq.Enumerable).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Runtime.CompilerServices.RuntimeHelpers).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Collections.Generic.Dictionary<,>).Assembly.Location),
            };
            var compilation = CSharpCompilation.Create(
                "DF1001Test",
                new[] { tree },
                references,
                new CSharpCompilationOptions(OutputKind.DynamicallyLinkedLibrary));

            var withAnalyzers = compilation.WithAnalyzers(
                ImmutableArray.Create<DiagnosticAnalyzer>(new StaticMutableCollectionAnalyzer()));
            var diagnostics = await withAnalyzers.GetAnalyzerDiagnosticsAsync().ConfigureAwait(false);
            return diagnostics.Where(d => d.Id == "DF1001").ToImmutableArray();
        }
    }
}
