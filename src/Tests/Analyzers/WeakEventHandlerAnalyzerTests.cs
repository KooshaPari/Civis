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
    /// Tests for <see cref="WeakEventHandlerAnalyzer"/> (DF1002).
    /// Detects subscriptions to known long-lived static events (e.g., SceneManager.sceneLoaded,
    /// AppDomain.UnhandledException) from instance objects without weak-event guarding.
    /// Suppression marker: <c>// weak-event-ok: &lt;reason&gt;</c>
    /// </summary>
    public class WeakEventHandlerAnalyzerTests
    {
        [Fact]
        public void HasCorrectId()
        {
            var analyzer = new WeakEventHandlerAnalyzer();
            analyzer.SupportedDiagnostics.Should().ContainSingle();
            analyzer.SupportedDiagnostics[0].Id.Should().Be("DF1002");
        }

        [Fact]
        public void HasExpectedSeverity()
        {
            var analyzer = new WeakEventHandlerAnalyzer();
            analyzer.SupportedDiagnostics[0].DefaultSeverity.Should().Be(DiagnosticSeverity.Info);
            analyzer.SupportedDiagnostics[0].IsEnabledByDefault.Should().BeTrue();
        }

        [Fact]
        public void HasExpectedCategory()
        {
            var analyzer = new WeakEventHandlerAnalyzer();
            analyzer.SupportedDiagnostics[0].Category.Should().Be("Resource Management");
        }

        [Fact]
        public async Task DetectsBareEventSubscription()
        {
            // Subscribing to a known long-lived static event without weak-event guard.
            const string source = @"
public static class SceneManager
{
    public static event System.Action<int> sceneLoaded;
}
public class MyComponent
{
    public MyComponent()
    {
        SceneManager.sceneLoaded += OnSceneLoaded;
    }
    private void OnSceneLoaded(int i) { }
}";
            var diagnostics = await RunAsync(source);
            diagnostics.Should().NotBeEmpty();
            diagnostics[0].Id.Should().Be("DF1002");
        }

        [Fact]
        public async Task IgnoresUnknownEventName()
        {
            // Event name is not in the KnownLongLivedEvents set.
            const string source = @"
public static class SomeOther
{
    public static event System.Action myCustomEvent;
}
public class MyComponent
{
    public MyComponent()
    {
        SomeOther.myCustomEvent += OnEvent;
    }
    private void OnEvent() { }
}";
            var diagnostics = await RunAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task HonorsSuppressionMarker()
        {
            const string source = @"
public static class SceneManager
{
    public static event System.Action<int> sceneLoaded;
}
public class MyComponent
{
    public MyComponent()
    {
        // weak-event-ok: cleaned up in OnDestroy
        SceneManager.sceneLoaded += OnSceneLoaded;
    }
    private void OnSceneLoaded(int i) { }
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
                "DF1002Test",
                new[] { tree },
                references,
                new CSharpCompilationOptions(OutputKind.DynamicallyLinkedLibrary));

            var withAnalyzers = compilation.WithAnalyzers(
                ImmutableArray.Create<DiagnosticAnalyzer>(new WeakEventHandlerAnalyzer()));
            var diagnostics = await withAnalyzers.GetAnalyzerDiagnosticsAsync().ConfigureAwait(false);
            return diagnostics.Where(d => d.Id == "DF1002").ToImmutableArray();
        }
    }
}
