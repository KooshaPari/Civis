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
    /// Tests for <see cref="TcsSyncContinuationAnalyzer"/> (DF0097).
    /// Covers metadata invariants and positive/negative firing behavior for
    /// <c>TaskCompletionSource</c> construction with/without
    /// <c>TaskCreationOptions.RunContinuationsAsynchronously</c>.
    ///
    /// Per iter-144 #648 P3 followup: DF0097 now supports a <c>// tcs-sync-ok: &lt;reason&gt;</c>
    /// inline suppression marker, symmetric with DF0111's <c>safe-swallow:</c> pattern.
    /// The trailing colon + reason are REQUIRED — bare <c>// tcs-sync-ok</c> is rejected.
    /// </summary>
    public class TcsSyncContinuationAnalyzerTests
    {
        private static DiagnosticAnalyzer GetAnalyzer() => new TcsSyncContinuationAnalyzer();

        // ---------- Metadata invariants ----------

        [Fact]
        public void DF0097_HasCorrectId()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics.Should().HaveCount(1);
            diagnostics[0].Id.Should().Be("DF0097");
            TcsSyncContinuationAnalyzer.DiagnosticId.Should().Be("DF0097");
        }

        [Fact]
        public void DF0097_HasWarningSeverity()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics[0].DefaultSeverity.Should().Be(DiagnosticSeverity.Warning);
        }

        [Fact]
        public void DF0097_HasConcurrencyCategory()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics[0].Category.Should().Be("Concurrency");
        }

        // ---------- Firing behavior — positive cases ----------

        [Fact]
        public async Task TaskCompletionSource_NoArgs_ReportsDF0097()
        {
            const string source = @"
using System.Threading.Tasks;

public class C
{
    public void M()
    {
        var tcs = new TaskCompletionSource<int>();
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle()
                .Which.Id.Should().Be("DF0097");
        }

        [Fact]
        public async Task TaskCompletionSource_NonGeneric_NoArgs_ReportsDF0097()
        {
            const string source = @"
using System.Threading.Tasks;

public class C
{
    public void M()
    {
        var tcs = new TaskCompletionSource();
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle()
                .Which.Id.Should().Be("DF0097");
        }

        // ---------- Firing behavior — negative cases ----------

        [Fact]
        public async Task TaskCompletionSource_WithRunContinuationsAsynchronously_DoesNotReport()
        {
            const string source = @"
using System.Threading.Tasks;

public class C
{
    public void M()
    {
        var tcs = new TaskCompletionSource<int>(TaskCreationOptions.RunContinuationsAsynchronously);
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task NonTcs_ConstructorCall_DoesNotReport()
        {
            const string source = @"
using System.Collections.Generic;

public class C
{
    public void M()
    {
        var list = new List<int>();
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        // ---------- iter-144 #648 P3 followup: tcs-sync-ok marker support ----------

        [Fact]
        public async Task TaskCompletionSource_WithReasonedTcsSyncOkMarker_DoesNotReport()
        {
            // Per iter-144 #648 P3 followup: a `// tcs-sync-ok: <reason>` marker
            // on the leading trivia of the containing statement suppresses DF0097,
            // symmetric with DF0111's `safe-swallow:` recognition.
            const string source = @"
using System.Threading.Tasks;

public class C
{
    public void M()
    {
        // tcs-sync-ok: producer always runs on the thread-pool, no marshalling risk
        var tcs = new TaskCompletionSource<int>();
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task TaskCompletionSource_WithReasonedTcsSyncOkMarker_TrailingSameLine_DoesNotReport()
        {
            // Trailing same-line marker should also suppress, mirroring DF0111's
            // closing-brace trailing-trivia scan.
            const string source = @"
using System.Threading.Tasks;

public class C
{
    public void M()
    {
        var tcs = new TaskCompletionSource<int>(); // tcs-sync-ok: synchronous-only test fixture
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task TaskCompletionSource_WithBareTcsSyncOkMarker_NoColon_StillReports()
        {
            // The trailing colon + reason are REQUIRED (Pattern #111 convention) —
            // bare `// tcs-sync-ok` without colon MUST NOT silence the diagnostic.
            // This forces authors to document why the sync continuation is safe.
            const string source = @"
using System.Threading.Tasks;

public class C
{
    public void M()
    {
        // tcs-sync-ok no colon means no recognition
        var tcs = new TaskCompletionSource<int>();
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle()
                .Which.Id.Should().Be("DF0097");
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
                MetadataReference.CreateFromFile(typeof(System.Threading.Tasks.Task).Assembly.Location),
            };

            var compilation = CSharpCompilation.Create(
                "DF0097Test",
                new[] { tree },
                references,
                new CSharpCompilationOptions(
                    OutputKind.DynamicallyLinkedLibrary,
                    nullableContextOptions: NullableContextOptions.Enable));

            var compilationWithAnalyzers = compilation.WithAnalyzers(
                ImmutableArray.Create<DiagnosticAnalyzer>(new TcsSyncContinuationAnalyzer()));
            var diagnostics = await compilationWithAnalyzers
                .GetAnalyzerDiagnosticsAsync()
                .ConfigureAwait(false);
            return diagnostics
                .Where(d => d.Id == "DF0097")
                .ToImmutableArray();
        }
    }
}
