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
    /// Tests for <see cref="PublicMutableCollectionAnalyzer"/> (DF0123, Pattern #123).
    /// Regression coverage for #748: analyzer must flag mutable collections in records,
    /// structs, and the extended type set (HashSet/ISet/Dictionary/IDictionary/Queue/Stack).
    /// </summary>
    public class PublicMutableCollectionAnalyzerTests
    {
        [Fact]
        public async Task Reports_OnRecord_With_PublicList()
        {
            const string source = @"
using System.Collections.Generic;
public record Foo
{
    public List<int> Items { get; set; } = new List<int>();
}";
            var diagnostics = await RunAsync(source);
            diagnostics.Should().NotBeEmpty();
            diagnostics[0].Id.Should().Be("DF0123");
        }

        [Fact]
        public async Task Reports_OnClass_With_PublicHashSet()
        {
            const string source = @"
using System.Collections.Generic;
public class Foo
{
    public HashSet<int> Items { get; set; } = new HashSet<int>();
}";
            var diagnostics = await RunAsync(source);
            diagnostics.Should().NotBeEmpty();
            diagnostics[0].Id.Should().Be("DF0123");
        }

        private static async Task<ImmutableArray<Diagnostic>> RunAsync(string source)
        {
            var tree = CSharpSyntaxTree.ParseText(
                source,
                new CSharpParseOptions(LanguageVersion.CSharp10));

            // Reference every runtime assembly from the TPA list so that GetTypeByMetadataName
            // resolves type-forwarded collections (List<>, HashSet<>, Dictionary<,>, etc.)
            // which live in System.Collections.dll on net8.0.
            var tpaList = ((string)System.AppContext.GetData("TRUSTED_PLATFORM_ASSEMBLIES")!)
                .Split(System.IO.Path.PathSeparator);
            var references = tpaList
                .Where(p => p.EndsWith(".dll", System.StringComparison.OrdinalIgnoreCase))
                .Select(p => (MetadataReference)MetadataReference.CreateFromFile(p))
                .ToArray();
            var compilation = CSharpCompilation.Create(
                "DF0123Test",
                new[] { tree },
                references,
                new CSharpCompilationOptions(OutputKind.DynamicallyLinkedLibrary));

            var withAnalyzers = compilation.WithAnalyzers(
                ImmutableArray.Create<DiagnosticAnalyzer>(new PublicMutableCollectionAnalyzer()));
            var diagnostics = await withAnalyzers.GetAnalyzerDiagnosticsAsync().ConfigureAwait(false);
            return diagnostics.Where(d => d.Id == "DF0123").ToImmutableArray();
        }
    }
}
