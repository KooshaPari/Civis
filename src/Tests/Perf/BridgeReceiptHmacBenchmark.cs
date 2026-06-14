using System.Security.Cryptography;
using BenchmarkDotNet.Attributes;
using BenchmarkDotNet.Order;
using DINOForge.Bridge.Client;

namespace DINOForge.Tests.Perf;

[MemoryDiagnoser]
[Orderer(SummaryOrderPolicy.FastestToSlowest)]
[SimpleJob(warmupCount: 1, iterationCount: 3)]
public class BridgeReceiptHmacBenchmark
{
    private static readonly byte[] SessionKey =
    [
        0x10, 0x21, 0x32, 0x43, 0x54, 0x65, 0x76, 0x87,
        0x98, 0xA9, 0xBA, 0xCB, 0xDC, 0xED, 0xFE, 0x0F,
        0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78,
        0x89, 0x9A, 0xAB, 0xBC, 0xCD, 0xDE, 0xEF, 0xF0
    ];

    private const string TimestampUtc = "2026-06-08T00:00:00.000Z";
    private const long WorldFrame = 42L;
    private const string StateSha256Hex = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    [Benchmark]
    public string ComputeReceiptHmac()
    {
        return BridgeReceiptVerifier.ComputeReceiptHmac(SessionKey, TimestampUtc, WorldFrame, StateSha256Hex);
    }
}
