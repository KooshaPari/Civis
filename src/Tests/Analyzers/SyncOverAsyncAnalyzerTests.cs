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
    /// Tests for <see cref="SyncOverAsyncAnalyzer"/> (DF0116).
    /// Covers metadata invariants and the marker-recognition matrix
    /// — same-line trailing trivia, previous-line leading trivia of the
    /// enclosing statement, and inline node-level trivia. Mirrors the
    /// DF0096 suppression semantics per Pattern #116 governance.
    /// </summary>
    public class SyncOverAsyncAnalyzerTests
    {
        private static DiagnosticAnalyzer GetAnalyzer() => new SyncOverAsyncAnalyzer();

        // ---------- Metadata invariants ----------

        [Fact]
        public void DF0116_HasCorrectId()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics.Should().HaveCount(1);
            diagnostics[0].Id.Should().Be("DF0116");
            SyncOverAsyncAnalyzer.DiagnosticId.Should().Be("DF0116");
        }

        [Fact]
        public void DF0116_HasWarningSeverity()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics[0].DefaultSeverity.Should().Be(DiagnosticSeverity.Warning);
        }

        [Fact]
        public void DF0116_HasReliabilityCategory()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics[0].Category.Should().Be("Reliability");
        }

        [Fact]
        public void DF0116_HasSuppressionMarker()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics[0].Description.ToString().Should().Contain("sync-over-async-unavoidable:");
        }

        // ---------- Firing behavior — .Result ----------

        [Fact]
        public async Task Result_OnTask_ReportsDF0116()
        {
            const string source = @"
using System.Threading.Tasks;

public class C
{
    public int M()
    {
        Task<int> task = Task.FromResult(42);
        return task.Result;
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle()
                .Which.Id.Should().Be("DF0116");
        }

        [Fact]
        public async Task Wait_OnTask_ReportsDF0116()
        {
            const string source = @"
using System.Threading.Tasks;

public class C
{
    public void M()
    {
        Task task = Task.CompletedTask;
        task.Wait();
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle()
                .Which.Id.Should().Be("DF0116");
        }

        [Fact]
        public async Task ResultType_PropertyAccess_DoesNotReport()
        {
            // .ResultType is a false-positive name and must not fire.
            const string source = @"
public class Outcome { public string ResultType { get; set; } = """"; }

public class C
{
    public string M(Outcome o) => o.ResultType;
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        // ---------- Marker recognition matrix ----------

        [Fact]
        public async Task Result_WithMarkerOnSameLine_DoesNotReport()
        {
            // Inline trailing-trivia marker on the .Result statement.
            const string source = @"
using System.Threading.Tasks;

public class C
{
    public int M()
    {
        Task<int> task = Task.FromResult(42);
        var x = task.Result; // sync-over-async-unavoidable: ECS-bound, main-thread-required
        return x;
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task Result_WithMarkerOnPreviousLine_DoesNotReport()
        {
            // Marker is on the previous line of the ENCLOSING STATEMENT.
            // This is the placement that the iter-143 #535 work used at the
            // 8 GameBridgeServer handler sites — previously not recognized
            // because the analyzer only read trivia of the MemberAccess node.
            const string source = @"
using System.Threading.Tasks;

public class C
{
    public int M()
    {
        Task<int> task = Task.FromResult(42);
        // sync-over-async-unavoidable: ECS-bound, main-thread-required
        var x = task.Result;
        return x;
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task Wait_WithMarkerOnPreviousLine_DoesNotReport()
        {
            const string source = @"
using System.Threading.Tasks;

public class C
{
    public void M()
    {
        Task task = Task.CompletedTask;
        // sync-over-async-unavoidable: framework boundary
        task.Wait();
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task Wait_WithMarkerOnSameLine_DoesNotReport()
        {
            const string source = @"
using System.Threading.Tasks;

public class C
{
    public void M()
    {
        Task task = Task.CompletedTask;
        task.Wait(); // sync-over-async-unavoidable: framework boundary
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task Result_WithoutMarker_StillReports()
        {
            // Negative control: confirm the firing path still fires when
            // no marker is present — i.e. our suppression logic didn't
            // accidentally start matching unrelated trivia.
            const string source = @"
using System.Threading.Tasks;

public class C
{
    public int M()
    {
        Task<int> task = Task.FromResult(42);
        // some unrelated comment
        var x = task.Result;
        return x;
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle()
                .Which.Id.Should().Be("DF0116");
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
                MetadataReference.CreateFromFile(typeof(System.Threading.Tasks.Task).Assembly.Location),
            };

            var compilation = CSharpCompilation.Create(
                "DF0116Test",
                new[] { tree },
                references,
                new CSharpCompilationOptions(
                    OutputKind.DynamicallyLinkedLibrary,
                    nullableContextOptions: NullableContextOptions.Enable));

            var compilationWithAnalyzers = compilation.WithAnalyzers(
                ImmutableArray.Create<DiagnosticAnalyzer>(new SyncOverAsyncAnalyzer()));
            var diagnostics = await compilationWithAnalyzers
                .GetAnalyzerDiagnosticsAsync()
                .ConfigureAwait(false);
            return diagnostics
                .Where(d => d.Id == "DF0116")
                .ToImmutableArray();
        }
    }
}
