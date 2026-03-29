using BenchmarkDotNet.Running;

namespace DINOForge.Benchmarks;

/// <summary>
/// Entry point for DINOForge performance benchmarks.
/// Run with: dotnet run --project src/Tests/Benchmarks/DINOForge.Benchmarks.csproj -- --configuration Release
/// </summary>
public class Program
{
    public static void Main(string[] args)
    {
        BenchmarkSwitcher.FromAssembly(typeof(Program).Assembly).Run(args);
    }
}
