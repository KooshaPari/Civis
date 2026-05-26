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
    /// Tests for <see cref="UnboundedConstraintAnalyzer"/> (DF0094).
    /// Detects unbounded version constraint string literals such as <c>"&gt;=0.1.0"</c>,
    /// <c>"&gt;0.1.0"</c>, and <c>"*"</c> in source code. Constraints with explicit upper
    /// bounds (e.g., <c>"&gt;=0.1.0 &lt;1.0.0"</c>) and tilde/caret semver ranges are allowed.
    /// Suppression marker: <c>// unbounded-version-ok: &lt;reason&gt;</c>
    /// </summary>
    public class UnboundedConstraintAnalyzerTests
    {
        [Fact]
        public void HasCorrectId()
        {
            var analyzer = new UnboundedConstraintAnalyzer();
            analyzer.SupportedDiagnostics.Should().ContainSingle();
            analyzer.SupportedDiagnostics[0].Id.Should().Be("DF0094");
        }

        [Fact]
        public void HasExpectedSeverity()
        {
            var analyzer = new UnboundedConstraintAnalyzer();
            analyzer.SupportedDiagnostics[0].DefaultSeverity.Should().Be(DiagnosticSeverity.Warning);
            analyzer.SupportedDiagnostics[0].IsEnabledByDefault.Should().BeTrue();
        }

        [Fact]
        public void HasExpectedCategory()
        {
            var analyzer = new UnboundedConstraintAnalyzer();
            analyzer.SupportedDiagnostics[0].Category.Should().Be("Design");
        }

        [Fact]
        public async Task DetectsUnboundedGreaterOrEqual()
        {
            const string source = @"
public class C
{
    public string FrameworkVersion = "">=0.1.0"";
}";
            var diagnostics = await RunAsync(source);
            diagnostics.Should().NotBeEmpty();
            diagnostics[0].Id.Should().Be("DF0094");
        }

        [Fact]
        public async Task DetectsWildcardConstraint()
        {
            const string source = @"
public class C
{
    public string AnyVersion = ""*"";
}";
            var diagnostics = await RunAsync(source);
            diagnostics.Should().NotBeEmpty();
            diagnostics[0].Id.Should().Be("DF0094");
        }

        [Fact]
        public async Task IgnoresBoundedConstraint()
        {
            const string source = @"
public class C
{
    public string FrameworkVersion = "">=0.1.0 <1.0.0"";
}";
            var diagnostics = await RunAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task HonorsSuppressionMarker()
        {
            const string source = @"
public class C
{
    // unbounded-version-ok: intentional, accepts any future version
    public string FrameworkVersion = "">=0.1.0"";
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
            };
            var compilation = CSharpCompilation.Create(
                "DF0094Test",
                new[] { tree },
                references,
                new CSharpCompilationOptions(OutputKind.DynamicallyLinkedLibrary));

            var withAnalyzers = compilation.WithAnalyzers(
                ImmutableArray.Create<DiagnosticAnalyzer>(new UnboundedConstraintAnalyzer()));
            var diagnostics = await withAnalyzers.GetAnalyzerDiagnosticsAsync().ConfigureAwait(false);
            return diagnostics.Where(d => d.Id == "DF0094").ToImmutableArray();
        }
    }
}
