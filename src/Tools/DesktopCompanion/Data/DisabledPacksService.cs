using System;
using System.Collections.Generic;
using System.IO;
using System.Threading.Tasks;
using Newtonsoft.Json;

namespace DINOForge.DesktopCompanion.Data
{
    /// <summary>
    /// Reads and writes <c>disabled_packs.json</c> — a simple JSON array of pack ID strings.
    /// Format is identical to the in-game <c>ModPlatform</c> persistence so files are interchangeable.
    /// </summary>
    public sealed class DisabledPacksService
    {
        private const string FileName = "disabled_packs.json";

        /// <summary>
        /// Returns the set of disabled pack IDs stored in <paramref name="packsDir"/>/disabled_packs.json.
        /// Returns an empty set if the file does not exist or cannot be read.
        /// </summary>
        public async Task<HashSet<string>> LoadAsync(string packsDir)
        {
            string filePath = Path.Combine(packsDir, FileName);
            if (!File.Exists(filePath))
                return new HashSet<string>(StringComparer.OrdinalIgnoreCase);

            try
            {
                string json = await File.ReadAllTextAsync(filePath).ConfigureAwait(false);
                List<string>? ids = JsonConvert.DeserializeObject<List<string>>(json);
                return new HashSet<string>(
                    ids ?? new List<string>(),
                    StringComparer.OrdinalIgnoreCase);
            }
            catch
            {
                return new HashSet<string>(StringComparer.OrdinalIgnoreCase);
            }
        }

        /// <summary>
        /// Persists <paramref name="disabledIds"/> to <paramref name="packsDir"/>/disabled_packs.json.
        /// </summary>
        public async Task SaveAsync(string packsDir, IEnumerable<string> disabledIds)
        {
            string filePath = Path.Combine(packsDir, FileName);
            string json = JsonConvert.SerializeObject(new List<string>(disabledIds), Formatting.Indented);
            await File.WriteAllTextAsync(filePath, json).ConfigureAwait(false);
        }
    }
}
