using System;
using System.Globalization;
using System.IO;
using System.Linq;
using BenchmarkDotNet.Reports;
using BenchmarkDotNet.Running;

namespace DINOForge.Tests.Perf;

public static class Program
{
    public static int Main(string[] args)
    {
        Summary summary = BenchmarkRunner.Run<BridgeReceiptHmacBenchmark>();
        BenchmarkReport report = summary.Reports.Single();

        double meanNsPerOp = report.ResultStatistics?.Mean ?? double.NaN;
        string resultPath = Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "Result.txt"));
        string resultText = string.Format(
            CultureInfo.InvariantCulture,
            "Benchmark: {0}{1}MeanNsPerOp: {2:F2}{1}",
            report.BenchmarkCase.Descriptor.WorkloadMethod.Name,
            Environment.NewLine,
            meanNsPerOp);

        File.WriteAllText(resultPath, resultText);
        return 0;
    }
}
