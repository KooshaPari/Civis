```

BenchmarkDotNet v0.13.12, Windows 11 (10.0.28020.2075)
Unknown processor
.NET SDK 11.0.100-preview.2.26159.112
  [Host]     : .NET 8.0.25 (8.0.2526.11203), X64 RyuJIT AVX2
  Job-VLXZPM : .NET 8.0.25 (8.0.2526.11203), X64 RyuJIT AVX2
  ShortRun   : .NET 8.0.25 (8.0.2526.11203), X64 RyuJIT AVX2

WarmupCount=3  

```
| Method                   | Job        | IterationCount | LaunchCount | Mean     | Error     | StdDev    | Rank | Gen0   | Gen1   | Allocated |
|------------------------- |----------- |--------------- |------------ |---------:|----------:|----------:|-----:|-------:|-------:|----------:|
| Optimized_HintedCapacity | Job-VLXZPM | 5              | Default     | 2.026 μs | 0.1078 μs | 0.0280 μs |    1 | 0.7095 | 0.0191 |   11.6 KB |
| Optimized_HintedCapacity | ShortRun   | 3              | 1           | 2.057 μs | 1.0686 μs | 0.0586 μs |    1 | 0.7095 | 0.0191 |   11.6 KB |
| Baseline_DefaultCapacity | Job-VLXZPM | 5              | Default     | 2.157 μs | 0.1113 μs | 0.0172 μs |    2 | 0.5646 | 0.0114 |   9.23 KB |
| Baseline_DefaultCapacity | ShortRun   | 3              | 1           | 2.161 μs | 0.6742 μs | 0.0370 μs |    2 | 0.5646 | 0.0114 |   9.23 KB |
