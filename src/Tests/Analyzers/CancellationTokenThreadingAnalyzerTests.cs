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
    /// Tests for <see cref="CancellationTokenThreadingAnalyzer"/> (DF0114).
    /// Verifies the analyzer fires when an async method with a
    /// <c>CancellationToken</c> parameter awaits an inner invocation that
    /// does NOT pass the token, and stays silent when the token is threaded
    /// through (or when no CT parameter exists).
    ///
    /// Per iter-144 #648: DF0114 does NOT support inline suppression markers
    /// — the only "negative" path is correct CT threading.
    /// </summary>
    public class CancellationTokenThreadingAnalyzerTests
    {
        private static DiagnosticAnalyzer GetAnalyzer() => new CancellationTokenThreadingAnalyzer();

        // ---------- Metadata invariants ----------

        [Fact]
        public void DF0114_HasCorrectId()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics.Should().HaveCount(1);
            diagnostics[0].Id.Should().Be("DF0114");
            CancellationTokenThreadingAnalyzer.DiagnosticId.Should().Be("DF0114");
        }

        [Fact]
        public void DF0114_HasWarningSeverity()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics[0].DefaultSeverity.Should().Be(DiagnosticSeverity.Warning);
        }

        [Fact]
        public void DF0114_HasAsyncCategory()
        {
            var diagnostics = GetAnalyzer().SupportedDiagnostics;
            diagnostics[0].Category.Should().Be("Async");
        }

        // ---------- Firing behavior — positive cases ----------

        [Fact]
        public async Task AsyncMethodWithCT_AwaitsInnerWithoutThreading_ReportsDF0114()
        {
            const string source = @"
using System.Threading;
using System.Threading.Tasks;

public class C
{
    public async Task OuterAsync(CancellationToken ct)
    {
        await InnerAsync();
    }

    public Task InnerAsync() => Task.CompletedTask;
    public Task InnerAsync(CancellationToken ct) => Task.CompletedTask;
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().ContainSingle()
                .Which.Id.Should().Be("DF0114");
        }

        // ---------- Firing behavior — negative cases ----------

        [Fact]
        public async Task AsyncMethodWithCT_ThreadsTokenToInner_DoesNotReport()
        {
            const string source = @"
using System.Threading;
using System.Threading.Tasks;

public class C
{
    public async Task OuterAsync(CancellationToken ct)
    {
        await InnerAsync(ct);
    }

    public Task InnerAsync(CancellationToken ct) => Task.CompletedTask;
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task AsyncMethodWithoutCT_NoDiagnostic()
        {
            // No CancellationToken parameter — analyzer should not fire at all.
            const string source = @"
using System.Threading.Tasks;

public class C
{
    public async Task OuterAsync()
    {
        await InnerAsync();
    }

    public Task InnerAsync() => Task.CompletedTask;
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task AsyncMethodWithCT_PassesLinkedCtsToken_DoesNotReport()
        {
            // The analyzer recognizes any `.Token` member access as a CT
            // derivative (heuristic for linked CTS / external CT sources).
            const string source = @"
using System.Threading;
using System.Threading.Tasks;

public class C
{
    public async Task OuterAsync(CancellationToken ct)
    {
        using var linkedCts = CancellationTokenSource.CreateLinkedTokenSource(ct);
        await InnerAsync(linkedCts.Token);
    }

    public Task InnerAsync(CancellationToken ct) => Task.CompletedTask;
}";
            var diagnostics = await RunAnalyzerAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task NonAsyncMethodWithCT_DoesNotReport()
        {
            // Analyzer only fires on async methods returning Task/Task<T>.
            const string source = @"
using System.Threading;
using System.Threading.Tasks;

public class C
{
    public void Outer(CancellationToken ct)
    {
        InnerAsync().Wait();
    }

    public Task InnerAsync() => Task.CompletedTask;
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
                MetadataReference.CreateFromFile(typeof(System.Threading.CancellationToken).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Threading.CancellationTokenSource).Assembly.Location),
            };

            var compilation = CSharpCompilation.Create(
                "DF0114Test",
                new[] { tree },
                references,
                new CSharpCompilationOptions(
                    OutputKind.DynamicallyLinkedLibrary,
                    nullableContextOptions: NullableContextOptions.Enable));

            var compilationWithAnalyzers = compilation.WithAnalyzers(
                ImmutableArray.Create<DiagnosticAnalyzer>(new CancellationTokenThreadingAnalyzer()));
            var diagnostics = await compilationWithAnalyzers
                .GetAnalyzerDiagnosticsAsync()
                .ConfigureAwait(false);
            return diagnostics
                .Where(d => d.Id == "DF0114")
                .ToImmutableArray();
        }
    }
}
