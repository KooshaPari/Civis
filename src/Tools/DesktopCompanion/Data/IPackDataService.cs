using System.Threading.Tasks;

namespace DINOForge.DesktopCompanion.Data
{
    /// <summary>
    /// Abstraction for loading pack metadata from a directory.
    /// Implementations read the filesystem; test doubles can return canned data.
    /// </summary>
    public interface IPackDataService
    {
        /// <summary>
        /// Scans <paramref name="packsDir"/> for packs, parses their manifests,
        /// and returns a result containing all discovered packs and any errors.
        /// </summary>
        /// <param name="packsDir">Absolute path to the packs root directory.</param>
        Task<LoadResultViewModel> LoadPacksAsync(string packsDir);
    }
}
