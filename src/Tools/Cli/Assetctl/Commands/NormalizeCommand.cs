#nullable enable
using System.CommandLine;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Assetctl.Commands;

/// <summary>
/// Normalize command - normalizes a candidate into working output.
/// </summary>
internal static class NormalizeCommand
{
    public static Command Create()
    {
        Argument<string> assetIdArg = new("assetId") { Description = "Asset pipeline identifier" };
        Option<string> pipelineRootOption = AssetctlOptions.PipelineRootOption();
        Option<string> formatOption = AssetctlOptions.FormatOption();

        Command command = new("normalize", "Normalize a candidate into working output.");
        command.Add(assetIdArg);
        command.Add(pipelineRootOption);
        command.Add(formatOption);

        command.SetAction((ParseResult parseResult, CancellationToken ct) =>
        {
            ct.ThrowIfCancellationRequested();
            string assetId = parseResult.GetRequiredValue(assetIdArg);
            string pipelineRoot = parseResult.GetValue(pipelineRootOption) ?? "assets-pipeline";
            string outputFormat = parseResult.GetValue(formatOption) ?? "text";

            AssetctlPipeline pipeline = new();
            AssetctlNormalizeResult result = pipeline.Normalize(assetId, pipelineRoot);

            if (!result.Success)
            {
                AssetctlOutput.WriteError("normalize", result.Message ?? "normalize failed", outputFormat);
                return Task.CompletedTask;
            }

            if (!AssetctlOutput.IsJsonOutput(outputFormat))
            {
                AssetctlOutput.WriteSuccessWithDim(
                    "Asset normalized.",
                    ("Asset ID", result.AssetId ?? string.Empty),
                    ("Working Dir", result.WorkingDir ?? string.Empty));
                return Task.CompletedTask;
            }

            AssetctlOutput.WriteJson(result);
            return Task.CompletedTask;
        });

        return command;
    }
}
