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
    /// Tests for <see cref="SilentCatchAnalyzer"/> (DF0111).
    /// Per iter-144 #648 finding: DF0111 DOES support marker-based
    /// suppression via <c>// safe-swallow: &lt;reason&gt;</c> or
    /// <c>// test-cleanup-ok</c>. Marker is matched in leading trivia,
    /// trailing trivia of the close brace, and inline trivia of the
    /// catch block braces.
    /// </summary>
    public class SilentCatchAnalyzerTests
    {
        private static DiagnosticAnalyzer GetAnalyzer() => new SilentCatchAnalyzer();

        // ---------- Metadata invariants ----------

        [Fact]
        public void DF0111_HasCorrectId()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics.Should().HaveCount(1);
            diagnostics[0].Id.Should().Be("DF0111");
            SilentCatchAnalyzer.DiagnosticId.Should().Be("DF0111");
        }

        [Fact]
        public void DF0111_HasWarningSeverity()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics[0].DefaultSeverity.Should().Be(DiagnosticSeverity.Warning);
        }

        [Fact]
        public void DF0111_HasObservabilityCategory()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics[0].Category.Should().Be("Observability");
        }

        [Fact]
        public void DF0111_HasSafeSwallowMarker()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics[0].Description.ToString().Should().Contain("safe-swallow:");
        }

        // ---------- Firing behavior — positive cases ----------

        [Fact]
        public async Task EmptyCatch_NoMarker_ReportsDF0111()
        {
            const string source = @"
public class C
{
    public void M()
    {
        try
        {
            System.Console.WriteLine();
        }
        catch
        {
        }
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle()
                .Which.Id.Should().Be("DF0111");
        }

        [Fact]
        public async Task EmptyTypedCatch_NoMarker_ReportsDF0111()
        {
            const string source = @"
using System;

public class C
{
    public void M()
    {
        try { System.Console.WriteLine(); }
        catch (Exception) { }
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle()
                .Which.Id.Should().Be("DF0111");
        }

        // ---------- Firing behavior — negative cases ----------

        [Fact]
        public async Task CatchWithBody_DoesNotReport()
        {
            const string source = @"
using System;

public class C
{
    public void M()
    {
        try { System.Console.WriteLine(); }
        catch (Exception ex) { System.Console.WriteLine(ex.Message); }
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task EmptyCatch_WithSafeSwallowMarker_LeadingTrivia_DoesNotReport()
        {
            const string source = @"
public class C
{
    public void M()
    {
        try
        {
            System.Console.WriteLine();
        }
        // safe-swallow: intentional cleanup
        catch
        {
        }
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task EmptyCatch_WithSafeSwallowMarker_InsideBraces_DoesNotReport()
        {
            const string source = @"
public class C
{
    public void M()
    {
        try { System.Console.WriteLine(); }
        catch
        {
            // safe-swallow: dispose path
        }
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task EmptyCatch_WithTestCleanupOkMarker_DoesNotReport()
        {
            const string source = @"
public class C
{
    public void M()
    {
        try { System.Console.WriteLine(); }
        catch
        {
            // test-cleanup-ok
        }
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        // ---------- #848 Gap Classes ----------

        // Gap A: discard pattern `_ = ex;`
        [Fact]
        public async Task DiscardPatternBody_ReportsDF0111()
        {
            const string source = @"
using System;

public class C
{
    public void M()
    {
        try { System.Console.WriteLine(); }
        catch (Exception ex) { _ = ex; }
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle().Which.Id.Should().Be("DF0111");
        }

        // Gap E: `catch when (false) { ... }` — always-dead filter
        [Fact]
        public async Task FilterWhenFalse_ReportsDF0111()
        {
            const string source = @"
using System;

public class C
{
    public void M()
    {
        try { System.Console.WriteLine(); }
        catch (Exception) when (false) { System.Console.WriteLine(""dead""); }
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle().Which.Id.Should().Be("DF0111");
        }

        // Gap C: `catch { return null; }` — silent-erasure (Pattern #104 overlap)
        [Fact]
        public async Task ReturnNullBody_ReportsDF0111()
        {
            const string source = @"
using System;

public class C
{
    public string? M()
    {
        try { return ""ok""; }
        catch (Exception) { return null; }
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle().Which.Id.Should().Be("DF0111");
        }

        // Gap C: `catch { return default; }`
        [Fact]
        public async Task ReturnDefaultBody_ReportsDF0111()
        {
            const string source = @"
using System;

public class C
{
    public int M()
    {
        try { return 1; }
        catch (Exception) { return default; }
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle().Which.Id.Should().Be("DF0111");
        }

        // Gap F: `catch { ; }` — empty statement only
        [Fact]
        public async Task EmptyStatementBody_ReportsDF0111()
        {
            const string source = @"
using System;

public class C
{
    public void M()
    {
        try { System.Console.WriteLine(); }
        catch (Exception) { ; }
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle().Which.Id.Should().Be("DF0111");
        }

        // Gap F: `catch { { } }` — nested empty block only
        [Fact]
        public async Task NestedEmptyBlockBody_ReportsDF0111()
        {
            const string source = @"
using System;

public class C
{
    public void M()
    {
        try { System.Console.WriteLine(); }
        catch (Exception) { { } }
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle().Which.Id.Should().Be("DF0111");
        }

        // Negative: `catch when (true) { log(); }` — always-true filter with real body is OK
        [Fact]
        public async Task FilterWhenTrue_WithBody_DoesNotReport()
        {
            const string source = @"
using System;

public class C
{
    public void M()
    {
        try { System.Console.WriteLine(); }
        catch (Exception ex) when (true) { System.Console.WriteLine(ex.Message); }
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        // Gap D documentation: `// TODO` placeholder comment inside catch — currently NOT flagged
        // by syntactic heuristic (statements.Count == 0 already covers truly empty body with
        // any trivia). Documented here to prevent future regression: a `catch { /* TODO */ }`
        // hits Case A (empty statements list) and IS flagged.
        [Fact]
        public async Task PlaceholderCommentOnlyBody_ReportsDF0111()
        {
            const string source = @"
using System;

public class C
{
    public void M()
    {
        try { System.Console.WriteLine(); }
        catch (Exception) { /* TODO: handle */ }
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle().Which.Id.Should().Be("DF0111");
        }

        // ---------- Helpers ----------

        private static async Task<ImmutableArray<Diagnostic>> RunAnalyzerAsync(string source)
        {
            var tree = CSharpSyntaxTree.ParseText(source);
            var references = new[]
            {
                MetadataReference.CreateFromFile(typeof(object).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Linq.Enumerable).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Runtime.CompilerServices.RuntimeHelpers).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Collections.Generic.List<>).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Console).Assembly.Location),
            };

            var compilation = CSharpCompilation.Create(
                "DF0111Test",
                new[] { tree },
                references,
                new CSharpCompilationOptions(
                    OutputKind.DynamicallyLinkedLibrary,
                    nullableContextOptions: NullableContextOptions.Enable));

            var compilationWithAnalyzers = compilation.WithAnalyzers(
                ImmutableArray.Create<DiagnosticAnalyzer>(new SilentCatchAnalyzer()));
            var diagnostics = await compilationWithAnalyzers
                .GetAnalyzerDiagnosticsAsync()
                .ConfigureAwait(false);
            return diagnostics
                .Where(d => d.Id == "DF0111")
                .ToImmutableArray();
        }
    }
}
