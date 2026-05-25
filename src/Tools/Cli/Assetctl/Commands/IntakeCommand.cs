#nullable enable
using System;
using System.CommandLine;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Assetctl.Commands;

/// <summary>
/// Intake command - creates raw intake artifacts for a search candidate.
/// </summary>
internal static class IntakeCommand
{
    public static Command Create()
    {
        Argument<string> candidateRefArg = new("candidateRef")
        {
            Description = "Candidate reference in format <source>:<externalId>"
        };
        Option<string> pipelineRootOption = AssetctlOptions.PipelineRootOption();
        Option<string> formatOption = AssetctlOptions.FormatOption();

        Command command = new("intake", "Create raw intake artifacts for a search candidate.");
        command.Add(candidateRefArg);
        command.Add(pipelineRootOption);
        command.Add(formatOption);

        command.SetAction((ParseResult parseResult, CancellationToken ct) =>
        {
            ct.ThrowIfCancellationRequested();
            string candidateRef = parseResult.GetRequiredValue(candidateRefArg);
            string outputFormat = parseResult.GetValue(formatOption) ?? "text";
            string pipelineRoot = parseResult.GetValue(pipelineRootOption) ?? "assets-pipeline";

            if (!TryParseCandidateRef(candidateRef, out string source, out string externalId, out string parseError))
            {
                AssetctlOutput.WriteError("intake", parseError, outputFormat);
                return Task.CompletedTask;
            }

            AssetctlPipeline pipeline = new();
            AssetctlIntakeResult result = pipeline.Intake(source, externalId, pipelineRoot);

            if (!result.Success)
            {
                AssetctlOutput.WriteError("intake", result.Message ?? "intake failed", outputFormat);
                return Task.CompletedTask;
            }

            if (!AssetctlOutput.IsJsonOutput(outputFormat))
            {
                AssetctlOutput.WriteSuccessWithDim(
                    "Intake created.",
                    ("Asset ID", result.AssetId ?? string.Empty),
                    ("Manifest", result.ManifestPath ?? string.Empty),
                    ("Directory", result.RawDir ?? string.Empty));
                return Task.CompletedTask;
            }

            AssetctlOutput.WriteJson(result);
            return Task.CompletedTask;
        });

        return command;
    }

    private static bool TryParseCandidateRef(string candidateRef, out string source, out string externalId, out string parseError)
    {
        source = string.Empty;
        externalId = string.Empty;
        parseError = string.Empty;

        if (string.IsNullOrWhiteSpace(candidateRef))
        {
            parseError = "candidate reference cannot be empty; expected <source>:<externalId>";
            return false;
        }

        int separatorIndex = candidateRef.IndexOf(':');
        if (separatorIndex <= 0 || separatorIndex == candidateRef.Length - 1)
        {
            parseError = "candidate reference must be in format <source>:<externalId>";
            return false;
        }

        string left = candidateRef[..separatorIndex].Trim();
        string right = candidateRef[(separatorIndex + 1)..].Trim();

        if (string.IsNullOrWhiteSpace(left) || string.IsNullOrWhiteSpace(right))
        {
            parseError = "candidate reference must be in format <source>:<externalId>";
            return false;
        }

        source = left;
        externalId = right;
        return true;
    }
}
