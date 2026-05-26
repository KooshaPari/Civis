# Bridge Protocol Benchmarks Baseline (v0.24.0)

**Date**: 2026-05-18  
**BenchmarkDotNet Version**: 0.13.12  
**Runtime**: .NET 8.0.25 (RyuJIT AVX2)  
**Platform**: Windows 11 (10.0.28020.2075)  
**Job Configuration**: ShortRun + Job-HVPBWK (3 + 5 iterations)

## Benchmark Suite

Three hot-path operations in the Bridge/GameClient protocol layer:

1. **JsonRpcRequest_Serialize_Roundtrip** — JSON-RPC message serialization + deserialization roundtrip (typical request/response cycle through named pipe)
2. **BridgeReceipt_HmacCompute** — HMAC-SHA256 signature computation over canonical receipt (Phase 4a+, called on every response in Strict mode)
3. **CanonicalJson_Sort** — Canonical JSON canonicalization (RFC 8785-style, alphabetical key sorting, no whitespace)

## Results Summary

| Benchmark | Mean (μs) | StdDev (μs) | Allocated | Rank |
|-----------|-----------|-------------|-----------|------|
| BridgeReceipt_HmacCompute (Job-HVPBWK) | 2.719 | 0.077 | 1.34 KB | 1 (fastest) |
| JsonRpcRequest_Serialize_Roundtrip (Job-HVPBWK) | 6.751 | 0.269 | 5.55 KB | 2 |
| CanonicalJson_Sort (Job-HVPBWK) | 7.274 | 0.289 | 9.09 KB | 2 |

## Individual Benchmark Details

### BridgeReceipt_HmacCompute

**Purpose**: Hot path for receipt verification on every response in Strict mode.

**Job-HVPBWK Results**:
- **Mean**: 2.719 μs (2.475 μs ShortRun)
- **StdDev**: 0.077 μs
- **Allocated**: 1.34 KB per operation
- **Gen0 Collections**: 0.0801 per 1000 ops

**Assessment**: Exceeds target. HMAC-SHA256 is cryptographic workload (CPU-bound, ~2-3 μs on modern hardware). 1.34 KB allocation per op is acceptable (session key + output buffer).

---

### JsonRpcRequest_Serialize_Roundtrip

**Purpose**: Message serialization/deserialization roundtrip (mimics GameClient request → pipe → GameBridgeServer → response).

**Job-HVPBWK Results**:
- **Mean**: 6.751 μs (6.913 μs ShortRun)
- **StdDev**: 0.269 μs
- **Allocated**: 5.55 KB per operation
- **Gen0 Collections**: 0.3357 per 1000 ops

**Assessment**: Within acceptable range for IPC workload. Newtonsoft.Json (Newtonsoft.Json.Linq) serialization inherently allocates strings + JObject graph. 5.55 KB per roundtrip is reasonable for a request with 3 entities + metadata.

**Optimization Opportunities**:
- Switch to System.Text.Json (faster, lower allocation) if deserialization order preservation not critical
- Consider pooling JObject allocations for high-frequency query paths

---

### CanonicalJson_Sort

**Purpose**: RFC 8785-style canonical JSON (deterministic, sorted keys, no whitespace) for tamper-proof state hashing.

**Job-HVPBWK Results**:
- **Mean**: 7.274 μs (6.938 μs ShortRun)
- **StdDev**: 0.289 μs
- **Allocated**: 9.09 KB per operation
- **Gen0 Collections**: 0.5493 per 1000 ops

**Assessment**: Dominates memory allocation due to StringBuilder + recursive token traversal. 9.09 KB per canonicalization is high but acceptable for tamper-proof hashing (not per-frame hot path, only on receipt verification).

**Optimization Opportunities**:
- Pre-allocate StringBuilder with estimated size (recursive depth hints)
- Cache canonical output for identical payloads (rare in practice, but possible for repeated queries)

---

## Regression Detection

Future benchmark runs should capture results in this same format. **Regression thresholds**:

| Metric | Threshold | Severity |
|--------|-----------|----------|
| Mean time increase > 20% | Block PR | CRITICAL |
| Mean time increase > 10% | Warn | MEDIUM |
| Allocated increase > 30% | Block PR | CRITICAL |
| Allocated increase > 15% | Warn | MEDIUM |

## CI Integration

Baseline snapshot committed to `docs/benchmarks/`. CI workflow (`benchmarks.yml`) compares future runs against this baseline and fails if thresholds are exceeded.

**Invocation**:
```bash
dotnet run --project src/Tests/Benchmarks/DINOForge.Benchmarks.csproj -c Release -- --filter "*BridgeProtocol*" --job short
```

---

## Notes

- **Warmup**: 3 iterations per job (standard BenchmarkDotNet)
- **Iterations**: 5 for Job-HVPBWK, 3 for ShortRun
- **GC**: Concurrent Workstation (default .NET 8 behavior)
- **Allocation Reporting**: Managed heap only (inclusive, post-GC consolidation)

This baseline serves as the reference point for performance regression detection in v0.24.0+.
