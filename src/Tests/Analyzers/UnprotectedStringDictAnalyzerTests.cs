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
    /// Tests for <see cref="UnprotectedStringDictAnalyzer"/> (DF0099, Pattern #99).
    /// NOTE: The analyzer source has NO inline suppression-marker mechanism — suppression
    /// is implicit via passing a StringComparer.Ordinal / .OrdinalIgnoreCase argument, OR
    /// via an external allowlist (`docs/qa/string-dict-allowlist.txt`). The
    /// "DoesNotReport_WhenSuppressionMarkerPresent" case is therefore implemented as
    /// "DoesNotReport_WhenExplicitComparerPassed".
    /// </summary>
    public class UnprotectedStringDictAnalyzerTests
    {
        // NOTE: Analyzer matches when the first type-argument identifier is the literal
        // text "string" (IdentifierNameSyntax). C# 'string' keyword parses as
        // PredefinedTypeSyntax — to exercise the analyzer we either alias String to string
        // or use the bare 'String' identifier. We use a fully-named identifier form below.

        [Fact]
        public async Task Reports_OnTriggerPattern()
        {
            const string source = @"
using System.Collections.Generic;
using @string = System.String;
public class C
{
    private Dictionary<@string, int> _map = new Dictionary<@string, int>();
}";
            var diagnostics = await RunAsync(source);
            diagnostics.Should().NotBeEmpty();
            diagnostics[0].Id.Should().Be("DF0099");
        }

        [Fact]
        public async Task DoesNotReport_WhenExplicitComparerPassed()
        {
            const string source = @"
using System;
using System.Collections.Generic;
using @string = System.String;
public class C
{
    private Dictionary<@string, int> _map = new Dictionary<@string, int>(StringComparer.Ordinal);
}";
            var diagnostics = await RunAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task Reports_WhenUsingStringKeyword()
        {
            // The C# 'string' keyword parses as PredefinedTypeSyntax (not IdentifierNameSyntax).
            // This is the common form and must be flagged.
            const string source = @"
using System.Collections.Generic;
public class C
{
    private Dictionary<string, int> _map = new Dictionary<string, int>();
}";
            var diagnostics = await RunAsync(source);
            diagnostics.Should().NotBeEmpty();
            diagnostics[0].Id.Should().Be("DF0099");
        }

        [Fact]
        public async Task DoesNotReport_When2ArgFormWithComparerInPos1()
        {
            // Regression for #719: 2-arg form `new Dictionary<string,T>(sourceDict, StringComparer.Ordinal)`
            // — comparer is at index 1, must be recognized.
            const string source = @"
using System;
using System.Collections.Generic;
using @string = System.String;
public class C
{
    private Dictionary<@string, int> _src = new Dictionary<@string, int>(StringComparer.Ordinal);
    private Dictionary<@string, int> _map = new Dictionary<@string, int>(_src, StringComparer.Ordinal);
}";
            var diagnostics = await RunAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task DoesNotReport_WhenNonStringKey()
        {
            // Dictionary<int, T> is outside the analyzer's scope (only string keys).
            const string source = @"
using System.Collections.Generic;
public class C
{
    private Dictionary<int, int> _map = new Dictionary<int, int>();
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
                MetadataReference.CreateFromFile(typeof(System.Collections.Generic.Dictionary<,>).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Linq.Enumerable).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Runtime.CompilerServices.RuntimeHelpers).Assembly.Location),
            };
            var compilation = CSharpCompilation.Create(
                "DF0099Test",
                new[] { tree },
                references,
                new CSharpCompilationOptions(OutputKind.DynamicallyLinkedLibrary));

            var withAnalyzers = compilation.WithAnalyzers(
                ImmutableArray.Create<DiagnosticAnalyzer>(new UnprotectedStringDictAnalyzer()));
            var diagnostics = await withAnalyzers.GetAnalyzerDiagnosticsAsync().ConfigureAwait(false);
            return diagnostics.Where(d => d.Id == "DF0099").ToImmutableArray();
        }
    }
}
