#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Text.RegularExpressions;
using BepInEx.Logging;
using DINOForge.SDK;
using Newtonsoft.Json;
using Newtonsoft.Json.Serialization;

namespace DINOForge.Runtime.Profiles
{
    /// <summary>
    /// Thunderstore-style mod profile manager.
    /// Profiles are saved as JSON files inside <c>BepInEx/dinoforge-profiles/</c>.
    ///
    /// All file names are sanitised (illegal chars → underscore) and given a <c>.json</c>
    /// extension so they are unambiguous on any filesystem.
    ///
    /// Thread-safety: methods are not thread-safe. Call only from the Unity main thread
    /// (or the deferred-work coroutine in RuntimeDriver).
    /// </summary>
    internal sealed class ProfileManager
    {
        private static readonly JsonSerializerSettings SerializerSettings = new JsonSerializerSettings
        {
            Formatting = Formatting.Indented,
            ContractResolver = new CamelCasePropertyNamesContractResolver(),
            DateParseHandling = DateParseHandling.DateTimeOffset,
            NullValueHandling = NullValueHandling.Ignore,
        };

        // Regex of chars illegal in Windows / Linux file names.
        private static readonly Regex IllegalFileNameChars =
            new Regex(@"[<>:""/\\|?*\x00-\x1F]", RegexOptions.Compiled);

        private readonly string _profilesDir;
        private readonly ManualLogSource _log;

        /// <summary>
        /// Initialises the manager. Creates <paramref name="profilesDir"/> if it does not exist.
        /// </summary>
        internal ProfileManager(string profilesDir, ManualLogSource log)
        {
            _profilesDir = profilesDir;
            _log = log;

            try
            {
                if (!Directory.Exists(_profilesDir))
                    Directory.CreateDirectory(_profilesDir);
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[ProfileManager] Could not create profiles directory '{_profilesDir}': {ex.Message}");
            }
        }

        // ── Public API ────────────────────────────────────────────────────────────

        /// <summary>Returns the names of all saved profiles (without file extension), sorted alphabetically.</summary>
        public IReadOnlyList<string> ListProfiles()
        {
            var names = new List<string>();
            try
            {
                if (!Directory.Exists(_profilesDir)) return names;
                foreach (string file in Directory.GetFiles(_profilesDir, "*.json"))
                {
                    names.Add(Path.GetFileNameWithoutExtension(file));
                }
                names.Sort(StringComparer.OrdinalIgnoreCase);
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[ProfileManager] ListProfiles failed: {ex.Message}");
            }
            return names;
        }

        /// <summary>
        /// Loads and returns the named profile, or <c>null</c> if it does not exist or is malformed.
        /// </summary>
        public ModProfile? Load(string name)
        {
            string path = FilePath(name);
            if (!File.Exists(path))
            {
                _log.LogWarning($"[ProfileManager] Profile '{name}' not found at '{path}'.");
                return null;
            }

            try
            {
                string json = File.ReadAllText(path, System.Text.Encoding.UTF8);
                ModProfile? profile = JsonConvert.DeserializeObject<ModProfile>(json, SerializerSettings);
                if (profile == null)
                {
                    _log.LogWarning($"[ProfileManager] Profile '{name}' deserialized to null — file may be empty.");
                    return null;
                }

                _log.LogInfo($"[ProfileManager] Loaded profile '{name}' — {profile.EnabledPacks.Count} pack(s) enabled.");
                return profile;
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[ProfileManager] Failed to load profile '{name}': {ex.Message}");
                return null;
            }
        }

        /// <summary>
        /// Saves the current set of enabled pack IDs as a named profile.
        /// Overwrites any existing profile with the same name.
        /// </summary>
        /// <param name="name">Profile name (will be sanitised for use as a file name).</param>
        /// <param name="enabledPackIds">The pack IDs that are currently enabled.</param>
        public void SaveCurrent(string name, IEnumerable<string> enabledPackIds)
        {
            if (string.IsNullOrWhiteSpace(name))
            {
                _log.LogWarning("[ProfileManager] SaveCurrent: name is null or whitespace — profile not saved.");
                return;
            }

            var profile = new ModProfile
            {
                Name = name,
                Version = "1",
                DinoForgeVersion = PluginInfo.VERSION,
                CreatedAt = DateTimeOffset.UtcNow,
                EnabledPacks = new List<string>(enabledPackIds),
                PackSettings = new Dictionary<string, Dictionary<string, string>>(StringComparer.Ordinal),
            };

            string path = FilePath(name);
            try
            {
                if (!Directory.Exists(_profilesDir))
                    Directory.CreateDirectory(_profilesDir);

                string json = JsonConvert.SerializeObject(profile, SerializerSettings);
                File.WriteAllText(path, json, System.Text.Encoding.UTF8);
                _log.LogInfo($"[ProfileManager] Saved profile '{name}' → {path} ({profile.EnabledPacks.Count} pack(s)).");
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[ProfileManager] SaveCurrent '{name}' failed: {ex.Message}");
            }
        }

        /// <summary>
        /// Exports the named profile to a JSON string.
        /// Returns an empty string if the profile does not exist.
        /// </summary>
        public string ExportJson(string name)
        {
            ModProfile? profile = Load(name);
            if (profile == null) return string.Empty;

            try
            {
                return JsonConvert.SerializeObject(profile, SerializerSettings);
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[ProfileManager] ExportJson '{name}' failed: {ex.Message}");
                return string.Empty;
            }
        }

        /// <summary>
        /// Imports a profile from a JSON string. Rejects profiles whose Version field is
        /// greater than <c>"1"</c> (unknown future schema).
        /// Throws <see cref="InvalidOperationException"/> on validation failure.
        /// </summary>
        /// <exception cref="InvalidOperationException">Thrown for unknown schema version or missing name.</exception>
        public void ImportJson(string json)
        {
            if (string.IsNullOrWhiteSpace(json))
                throw new InvalidOperationException("Import JSON is empty.");

            ModProfile? profile;
            try
            {
                profile = JsonConvert.DeserializeObject<ModProfile>(json, SerializerSettings);
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Profile JSON is malformed: {ex.Message}", ex);
            }

            if (profile == null)
                throw new InvalidOperationException("Profile JSON deserialized to null.");

            if (string.IsNullOrWhiteSpace(profile.Name))
                throw new InvalidOperationException("Profile JSON has no 'name' field.");

            // Version gate: reject future schema versions
            if (string.Compare(profile.Version ?? "1", "1", StringComparison.Ordinal) > 0)
                throw new InvalidOperationException(
                    $"Profile version '{profile.Version}' is newer than this DINOForge build supports (max '1').");

            // Persist (overwrites if same name)
            SaveCurrent(profile.Name, profile.EnabledPacks);
            _log.LogInfo($"[ProfileManager] Imported profile '{profile.Name}' from clipboard.");
        }

        /// <summary>
        /// Deletes the named profile.
        /// Returns <c>true</c> if the file was deleted, <c>false</c> if it did not exist.
        /// </summary>
        public bool Delete(string name)
        {
            string path = FilePath(name);
            if (!File.Exists(path)) return false;

            try
            {
                File.Delete(path);
                _log.LogInfo($"[ProfileManager] Deleted profile '{name}'.");
                return true;
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[ProfileManager] Delete '{name}' failed: {ex.Message}");
                return false;
            }
        }

        // ── Helpers ────────────────────────────────────────────────────────────────

        /// <summary>Returns the absolute file path for a given profile name.</summary>
        private string FilePath(string name)
        {
            string safeName = IllegalFileNameChars.Replace(name, "_");
            if (string.IsNullOrWhiteSpace(safeName)) safeName = "profile";
            return Path.Combine(_profilesDir, safeName + ".json");
        }
    }
}
