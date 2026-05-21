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
    /// Tests for <see cref="LockAroundAwaitAnalyzer"/> (DF1003).
    /// Concurrency hazard: <c>await</c> inside a <c>lock</c> block risks
    /// IllegalMonitorStateException since the continuation may resume on a
    /// different thread than the one that entered the monitor.
    /// Supports inline suppression marker
    /// <c>// lock-await-ok: &lt;reason&gt;</c>.
    /// </summary>
    public class LockAroundAwaitAnalyzerTests
    {
        private static DiagnosticAnalyzer GetAnalyzer() => new LockAroundAwaitAnalyzer();

        // ---------- Metadata invariants ----------

        [Fact]
        public void DF1003_HasCorrectId()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics.Should().HaveCount(1);
            diagnostics[0].Id.Should().Be("DF1003");
            LockAroundAwaitAnalyzer.DiagnosticId.Should().Be("DF1003");
        }

        [Fact]
        public void DF1003_HasWarningSeverity()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics[0].DefaultSeverity.Should().Be(DiagnosticSeverity.Warning);
        }

        [Fact]
        public void DF1003_HasConcurrencyCategory()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics[0].Category.Should().Be("Concurrency");
        }

        [Fact]
        public void DF1003_HasLockAwaitOkMarker()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics[0].Description.ToString().Should().Contain("lock-await-ok:");
        }

        // ---------- Firing behavior — positive cases ----------

        [Fact]
        public async Task AwaitInsideLock_ReportsDF1003()
        {
            const string source = @"
using System.Threading.Tasks;

public class C
{
    private readonly object _gate = new object();

    public async Task M()
    {
        lock (_gate)
        {
            await Task.Delay(1);
        }
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle()
                .Which.Id.Should().Be("DF1003");
        }

        [Fact]
        public async Task AwaitDeepNestedInLock_StillReports()
        {
            // Ancestor walk should find the lock even past intermediate scopes.
            const string source = @"
using System.Threading.Tasks;

public class C
{
    private readonly object _gate = new object();

    public async Task M(bool flag)
    {
        lock (_gate)
        {
            if (flag)
            {
                await Task.Delay(1);
            }
        }
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle()
                .Which.Id.Should().Be("DF1003");
        }

        // ---------- Firing behavior — negative cases ----------

        [Fact]
        public async Task AwaitOutsideLock_DoesNotReport()
        {
            const string source = @"
using System.Threading.Tasks;

public class C
{
    public async Task M()
    {
        await Task.Delay(1);
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task AwaitInsideLock_WithLockAwaitOkMarker_DoesNotReport()
        {
            const string source = @"
using System.Threading.Tasks;

public class C
{
    private readonly object _gate = new object();

    public async Task M()
    {
        lock (_gate)
        {
            // lock-await-ok: hot path is rarely contended
            await Task.Delay(1);
        }
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task AwaitAfterLockReleased_DoesNotReport()
        {
            const string source = @"
using System.Threading.Tasks;

public class C
{
    private readonly object _gate = new object();
    private int _state;

    public async Task M()
    {
        int snapshot;
        lock (_gate)
        {
            snapshot = _state;
        }
        await Task.Delay(snapshot);
    }
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
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
                "DF1003Test",
                new[] { tree },
                references,
                new CSharpCompilationOptions(
                    OutputKind.DynamicallyLinkedLibrary,
                    nullableContextOptions: NullableContextOptions.Enable));

            var compilationWithAnalyzers = compilation.WithAnalyzers(
                ImmutableArray.Create<DiagnosticAnalyzer>(new LockAroundAwaitAnalyzer()));
            var diagnostics = await compilationWithAnalyzers
                .GetAnalyzerDiagnosticsAsync()
                .ConfigureAwait(false);
            return diagnostics
                .Where(d => d.Id == "DF1003")
                .ToImmutableArray();
        }
    }
}
