using BenchmarkDotNet.Running;
using DINOForge.Benchmarks;
using System.Reflection;

// Run all benchmarks in the assembly
var summary = BenchmarkRunner.Run(Assembly.GetExecutingAssembly());
