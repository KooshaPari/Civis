#nullable enable
using System;
using System.IO;

namespace DINOForge.Tests.GameLaunch;

/// <summary>Resolves <see cref="WarfareStarwarsPackUnits.PackId"/> on disk for offline registry checks in GameLaunch tests.</summary>
internal static class WarfareStarwarsPackPaths
{
    public static string ResolvePackRoot()
    {
        string? current = Path.GetDirectoryName(typeof(WarfareStarwarsPackPaths).Assembly.Location);
        while (current != null)
        {
            string candidate = Path.Combine(current, "packs", WarfareStarwarsPackUnits.PackId);
            if (File.Exists(Path.Combine(candidate, "pack.yaml")))
            {
                return candidate;
            }

            current = Path.GetDirectoryName(current);
        }

        current = Directory.GetCurrentDirectory();
        while (current != null)
        {
            string candidate = Path.Combine(current, "packs", WarfareStarwarsPackUnits.PackId);
            if (File.Exists(Path.Combine(candidate, "pack.yaml")))
            {
                return candidate;
            }

            current = Path.GetDirectoryName(current);
        }

        throw new DirectoryNotFoundException(
            $"Could not locate packs/{WarfareStarwarsPackUnits.PackId}/pack.yaml from test output or cwd");
    }
}
