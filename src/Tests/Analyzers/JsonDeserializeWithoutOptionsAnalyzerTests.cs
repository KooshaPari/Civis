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
    /// Tests for <see cref="JsonDeserializeWithoutOptionsAnalyzer"/> (DF0120, Pattern #120).
    /// NOTE: The analyzer source has NO inline suppression-marker mechanism — suppression
    /// is implicit by passing a canonical JsonSerializerOptions argument, OR via an external
    /// allowlist (`docs/qa/unguarded-json-deserialize-allowlist.txt`). The
    /// "DoesNotReport_WhenSuppressionMarkerPresent" case is therefore implemented as
    /// "DoesNotReport_WhenExplicitOptionsPassed".
    /// </summary>
    public class JsonDeserializeWithoutOptionsAnalyzerTests
    {
        [Fact]
        public async Task Reports_OnTriggerPattern()
        {
            const string source = @"
using System.Text.Json;
public class C
{
    public Foo Parse(string s) => JsonSerializer.Deserialize<Foo>(s);
}
public class Foo { }";
            var diagnostics = await RunAsync(source);
            diagnostics.Should().NotBeEmpty();
            diagnostics[0].Id.Should().Be("DF0120");
        }

        [Fact]
        public async Task DoesNotReport_WhenExplicitOptionsPassed()
        {
            const string source = @"
using System.Text.Json;
public class C
{
    private static readonly JsonSerializerOptions Opts = new JsonSerializerOptions();
    public Foo Parse(string s) => JsonSerializer.Deserialize<Foo>(s, Opts);
}
public class Foo { }";
            var diagnostics = await RunAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task DoesNotReport_When3ArgFormWithJsonTypeInfo()
        {
            // Source-gen overload: JsonSerializer.Deserialize(json, JsonTypeInfo<T>)
            // Passing a JsonTypeInfo means caller is intentionally opting into the
            // source-gen pathway; options are baked into the type info.
            const string source = @"
using System.Text.Json;
using System.Text.Json.Serialization.Metadata;
public class C
{
    private static readonly JsonTypeInfo<Foo> Info = null!;
    public Foo Parse(string s) => JsonSerializer.Deserialize(s, Info);
}
public class Foo { }";
            var diagnostics = await RunAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task Reports_WhenQualifiedSystemTextJsonJsonSerializer()
        {
            const string source = @"
public class C
{
    public Foo Parse(string s) => System.Text.Json.JsonSerializer.Deserialize<Foo>(s);
}
public class Foo { }";
            var diagnostics = await RunAsync(source);
            diagnostics.Should().NotBeEmpty();
            diagnostics[0].Id.Should().Be("DF0120");
        }

        [Fact]
        public async Task DoesNotReport_WhenNonDeserializeCall()
        {
            const string source = @"
using System.Text.Json;
public class C
{
    public string Stringify(Foo f) => JsonSerializer.Serialize(f);
}
public class Foo { }";
            var diagnostics = await RunAsync(source);
            diagnostics.Should().BeEmpty();
        }

        [Fact]
        public async Task DoesNotReport_OnBareCustomDeserialize()
        {
            // Regression: #780 — bare `Deserialize<T>(...)` on a non-JsonSerializer type
            // (e.g. YamlLoader.Deserialize) was being flagged because the syntactic
            // fast-path accepted any GenericNameSyntax named "Deserialize" without
            // checking the receiver.
            const string source = @"
public class YamlLoader { public static T Deserialize<T>(string s) => default!; }
public class C { void M() { YamlLoader.Deserialize<int>(""x""); } }";
            var diagnostics = await RunAsync(source);
            diagnostics.Should().BeEmpty();
        }

        private static async Task<ImmutableArray<Diagnostic>> RunAsync(string source)
        {
            var tree = CSharpSyntaxTree.ParseText(source);
            var references = new[]
            {
                MetadataReference.CreateFromFile(typeof(object).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Text.Json.JsonSerializer).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Linq.Enumerable).Assembly.Location),
                MetadataReference.CreateFromFile(typeof(System.Runtime.CompilerServices.RuntimeHelpers).Assembly.Location),
            };
            var compilation = CSharpCompilation.Create(
                "DF0120Test",
                new[] { tree },
                references,
                new CSharpCompilationOptions(OutputKind.DynamicallyLinkedLibrary));

            var withAnalyzers = compilation.WithAnalyzers(
                ImmutableArray.Create<DiagnosticAnalyzer>(new JsonDeserializeWithoutOptionsAnalyzer()));
            var diagnostics = await withAnalyzers.GetAnalyzerDiagnosticsAsync().ConfigureAwait(false);
            return diagnostics.Where(d => d.Id == "DF0120").ToImmutableArray();
        }
    }
}
