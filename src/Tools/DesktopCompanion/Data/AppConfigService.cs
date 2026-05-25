using System;
using System.IO;
using System.Threading.Tasks;
using Newtonsoft.Json;

namespace DINOForge.DesktopCompanion.Data
{
    /// <summary>
    /// Loads and saves <see cref="AppConfig"/> from a JSON file in the user's local app data.
    /// </summary>
    public sealed class AppConfigService
    {
        private static readonly string ConfigDirectory = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
            "DINOForge", "DesktopCompanion");

        private static readonly string ConfigFilePath =
            Path.Combine(ConfigDirectory, "AppConfig.json");

        /// <summary>
        /// Loads app config from disk. Returns a default config if the file does not exist.
        /// </summary>
        public async Task<AppConfig> LoadAsync()
        {
            if (!File.Exists(ConfigFilePath))
                return BuildDefault();

            try
            {
                string json = await File.ReadAllTextAsync(ConfigFilePath).ConfigureAwait(false);
                return JsonConvert.DeserializeObject<AppConfig>(json) ?? BuildDefault();
            }
            catch
            {
                return BuildDefault();
            }
        }

        /// <summary>
        /// Saves <paramref name="config"/> to disk.
        /// </summary>
        public async Task SaveAsync(AppConfig config)
        {
            Directory.CreateDirectory(ConfigDirectory);
            string json = JsonConvert.SerializeObject(config, Formatting.Indented);
            await File.WriteAllTextAsync(ConfigFilePath, json).ConfigureAwait(false);
        }

        private static AppConfig BuildDefault()
        {
            // Attempt to detect packs/ dir relative to exe location
            string exeDir = AppContext.BaseDirectory;
            string repoPacksGuess = Path.GetFullPath(Path.Combine(exeDir, "..", "..", "..", "..", "packs"));

            return new AppConfig
            {
                PacksDirectory = Directory.Exists(repoPacksGuess) ? repoPacksGuess : "",
                GameDirectory = "",
                ReloadIntervalMs = 2000
            };
        }
    }
}
