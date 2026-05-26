#nullable enable
using System.CommandLine;

namespace DINOForge.Tools.Cli.Assetctl
{
    /// <summary>
    /// Shared System.CommandLine option factories for assetctl subcommands.
    /// </summary>
    public static class AssetctlOptions
    {
        /// <summary>Returns the --pipeline-root option (path to the pack asset pipeline root).</summary>
        public static Option<string> PipelineRootOption() =>
            new("--pipeline-root")
            {
                DefaultValueFactory = _ => ".",
                Description = "Path to the pack asset pipeline root directory"
            };

        /// <summary>Returns the --format option (json | table).</summary>
        public static Option<string> FormatOption() =>
            new("--format")
            {
                DefaultValueFactory = _ => "table",
                Description = "Output format: 'json' for machine-readable, 'table' for human-readable (default)"
            };
    }
}
