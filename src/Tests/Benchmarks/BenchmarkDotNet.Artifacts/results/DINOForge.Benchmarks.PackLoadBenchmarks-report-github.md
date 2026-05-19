```

BenchmarkDotNet v0.13.12, Windows 11 (10.0.28020.2075)
Unknown processor
.NET SDK 11.0.100-preview.2.26159.112
  [Host]     : .NET 8.0.25 (8.0.2526.11203), X64 RyuJIT AVX2
  Job-OPIYDZ : .NET 8.0.25 (8.0.2526.11203), X64 RyuJIT AVX2
  ShortRun   : .NET 8.0.25 (8.0.2526.11203), X64 RyuJIT AVX2

WarmupCount=3  

```
| Method                           | Job        | IterationCount | LaunchCount | Mean         | Error         | StdDev       | Rank | Gen0      | Gen1   | Allocated   |
|--------------------------------- |----------- |--------------- |------------ |-------------:|--------------:|-------------:|-----:|----------:|-------:|------------:|
| PackLoad_FullCycle               | Job-OPIYDZ | 5              | Default     |     38.14 μs |      7.043 μs |     1.090 μs |    1 |    1.6479 | 0.0610 |     27.7 KB |
| PackYaml_Parse_SinglePack        | ShortRun   | 3              | 1           |     39.42 μs |     21.573 μs |     1.183 μs |    2 |    1.4648 |      - |     27.7 KB |
| PackYaml_Parse_SinglePack        | Job-OPIYDZ | 5              | Default     |     39.89 μs |      2.804 μs |     0.728 μs |    2 |    1.4648 |      - |     27.7 KB |
| PackLoad_FullCycle               | ShortRun   | 3              | 1           |     44.77 μs |     70.634 μs |     3.872 μs |    2 |    1.4648 |      - |     27.7 KB |
| PackManifest_Validate_BulkPasses | Job-OPIYDZ | 5              | Default     | 42,715.52 μs |  8,122.755 μs | 2,109.454 μs |    3 | 1500.0000 |      - | 27699.16 KB |
| PackManifest_Validate_BulkPasses | ShortRun   | 3              | 1           | 46,544.28 μs | 37,190.943 μs | 2,038.561 μs |    4 | 1500.0000 |      - | 27699.16 KB |
