#nullable enable
using System;
using System.IO;

namespace DINOForge.SDK.HotReload
{
    /// <summary>
    /// Resolves pack roots by walking parent directories until a <c>pack.yaml</c> manifest is found.
    /// </summary>
    internal sealed class FileSystemPackRootResolver : IPackRootResolver
    {
        public string? ResolvePackRoot(string changedPath, string packsRootDirectory)
        {
            string? dir = Path.GetDirectoryName(changedPath);
            while (dir != null && !string.Equals(dir, packsRootDirectory, StringComparison.OrdinalIgnoreCase))
            {
                if (File.Exists(Path.Combine(dir, "pack.yaml")))
                {
                    return dir;
                }

                dir = Path.GetDirectoryName(dir);
            }

            if (dir != null && File.Exists(Path.Combine(dir, "pack.yaml")))
            {
                return dir;
            }

            return null;
        }
    }
}
