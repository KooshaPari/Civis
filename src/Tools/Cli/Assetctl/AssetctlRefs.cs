#nullable enable

namespace DINOForge.Tools.Cli.Assetctl;

/// <summary>
/// Shared parsing for assetctl candidate and model references (<c>source:externalId</c>).
/// </summary>
internal static class AssetctlRefs
{
    internal static bool TryParseCandidateRef(
        string candidateRef,
        out string source,
        out string externalId,
        out string parseError)
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

    internal static bool TryParseModelRef(string modelRef, out string source, out string modelId, out string parseError) =>
        TryParseCandidateRef(modelRef, out source, out modelId, out parseError);
}
