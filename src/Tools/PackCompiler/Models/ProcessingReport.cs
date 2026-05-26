#nullable enable
using System;
using System.Collections.Generic;

namespace DINOForge.Tools.PackCompiler.Models
{
    /// <summary>
    /// Report generated after asset pipeline execution.
    /// Contains success/failure status and detailed metrics.
    /// </summary>
    public class ProcessingReport
    {
        public required string PackId { get; init; }
        public required string Phase { get; init; }
        public required DateTime ExecutionTime { get; init; }
        public required double DurationSeconds { get; init; }

        public ProcessingStatus Status { get; set; } = ProcessingStatus.Pending;
        public List<string> Errors { get; init; } = new();
        public List<string> Warnings { get; init; } = new();

        public List<AssetProcessingResult> Assets { get; init; } = new();
        public PipelineMetrics Metrics { get; init; } = new();

        public bool IsSuccess => Status == ProcessingStatus.Success && Errors.Count == 0;
    }

    /// <summary>Processing status</summary>
    public enum ProcessingStatus
    {
        Pending,
        Running,
        Success,
        PartialSuccess,
        Failed
    }

    /// <summary>Result for a single asset</summary>
    public class AssetProcessingResult
    {
        public required string AssetId { get; init; }
        public required string Stage { get; init; }  // import | validate | optimize | generate

        public bool Success { get; set; }
        public string? Error { get; set; }

        public AssetMetrics? Metrics { get; set; }
        public double DurationSeconds { get; set; }
    }

    /// <summary>Metrics for a processed asset</summary>
    public class AssetMetrics
    {
        public int OriginalPolyCount { get; set; }
        public int LOD0PolyCount { get; set; }
        public int LOD1PolyCount { get; set; }
        public int LOD2PolyCount { get; set; }

        public float LOD1Percent => LOD0PolyCount > 0 ? (LOD1PolyCount / (float)LOD0PolyCount) * 100f : 0f;
        public float LOD2Percent => LOD0PolyCount > 0 ? (LOD2PolyCount / (float)LOD0PolyCount) * 100f : 0f;

        public int VertexCount { get; set; }
        public int TextureCount { get; set; }
        public int MaterialCount { get; set; }

        public string? OutputPath { get; set; }
        public string? AddressableKey { get; set; }
    }

    /// <summary>Overall pipeline metrics</summary>
    public class PipelineMetrics
    {
        public int TotalAssets { get; set; }
        public int SuccessfulAssets { get; set; }
        public int FailedAssets { get; set; }

        public double ImportTimeSeconds { get; set; }
        public double ValidateTimeSeconds { get; set; }
        public double OptimizeTimeSeconds { get; set; }
        public double GenerateTimeSeconds { get; set; }

        public int TotalPolysOriginal { get; set; }
        public int TotalPolysLOD0 { get; set; }
        public int TotalPolysLOD1 { get; set; }
        public int TotalPolysLOD2 { get; set; }

        public float SuccessRate => TotalAssets > 0 ? (SuccessfulAssets / (float)TotalAssets) * 100f : 0f;
        public double AverageTimePerAsset => TotalAssets > 0 ? (ImportTimeSeconds + ValidateTimeSeconds + OptimizeTimeSeconds + GenerateTimeSeconds) / TotalAssets : 0;
    }
}
