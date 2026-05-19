```

BenchmarkDotNet v0.13.12, Windows 11 (10.0.28020.2075)
Unknown processor
.NET SDK 11.0.100-preview.2.26159.112
  [Host]     : .NET 8.0.25 (8.0.2526.11203), X64 RyuJIT AVX2
  ShortRun   : .NET 8.0.25 (8.0.2526.11203), X64 RyuJIT AVX2
  Job-HVPBWK : .NET 8.0.25 (8.0.2526.11203), X64 RyuJIT AVX2

WarmupCount=3  

```
| Method                             | Job        | IterationCount | LaunchCount | Mean     | Error     | StdDev    | Rank | Gen0   | Allocated |
|----------------------------------- |----------- |--------------- |------------ |---------:|----------:|----------:|-----:|-------:|----------:|
| BridgeReceipt_HmacCompute          | ShortRun   | 3              | 1           | 2.475 μs | 2.9387 μs | 0.1611 μs |    1 | 0.0801 |   1.34 KB |
| BridgeReceipt_HmacCompute          | Job-HVPBWK | 5              | Default     | 2.719 μs | 0.2982 μs | 0.0774 μs |    1 | 0.0801 |   1.34 KB |
| JsonRpcRequest_Serialize_Roundtrip | Job-HVPBWK | 5              | Default     | 6.751 μs | 1.0362 μs | 0.2691 μs |    2 | 0.3357 |   5.55 KB |
| JsonRpcRequest_Serialize_Roundtrip | ShortRun   | 3              | 1           | 6.913 μs | 6.1989 μs | 0.3398 μs |    2 | 0.3357 |   5.55 KB |
| CanonicalJson_Sort                 | ShortRun   | 3              | 1           | 6.938 μs | 6.3284 μs | 0.3469 μs |    2 | 0.5493 |   9.09 KB |
| CanonicalJson_Sort                 | Job-HVPBWK | 5              | Default     | 7.274 μs | 1.1117 μs | 0.2887 μs |    2 | 0.5493 |   9.09 KB |
