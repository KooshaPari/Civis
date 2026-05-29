#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using BepInEx.Logging;
using DINOForge.Runtime.Json;

namespace DINOForge.Runtime.Settings
{
    /// <summary>
    /// Manages per-pack runtime settings stored in <c>BepInEx/dinoforge-pack-settings.json</c>.
    /// Provides thread-safe get/set operations with automatic persistence.
    /// </summary>
    public sealed class PackSettingsStore : IDisposable
    {
        // ── Singleton ────────────────────────────────────────────────────────────
        private static PackSettingsStore? _instance;
        private static readonly object _lockObj = new object();

        /// <summary>
        /// Gets or creates the singleton instance, initialised with the BepInEx root path.
        /// Must be called once from Plugin.Awake() before any other access.
        /// </summary>
        public static PackSettingsStore GetOrCreate(string bepInExRootPath)
        {
            if (_instance == null)
            {
                lock (_lockObj)
                {
                    if (_instance == null)
                    {
                        _instance = new PackSettingsStore(bepInExRootPath);
                    }
                }
            }
            return _instance;
        }

        /// <summary>
        /// Gets the singleton instance. Throws if <see cref="GetOrCreate"/> has not been called yet.
        /// </summary>
        public static PackSettingsStore Instance
        {
            get
            {
                if (_instance == null)
                    throw new InvalidOperationException("[PackSettingsStore] Instance not initialised — call GetOrCreate(bepInExRootPath) first.");
                return _instance;
            }
        }

        // ── Fields ───────────────────────────────────────────────────────────────
        private readonly string _settingsPath;
        private readonly Dictionary<string, Dictionary<string, object>> _settings = new Dictionary<string, Dictionary<string, object>>(StringComparer.Ordinal);
        private readonly object _settingsLock = new object();
        private ManualLogSource? _log;

        // ── Constructor ──────────────────────────────────────────────────────────

        /// <summary>
        /// Initializes the settings store using the BepInEx root path for storage.
        /// </summary>
        /// <param name="bepInExRootPath">
        /// The BepInEx root directory (e.g. <c>BepInEx.Paths.BepInExRootPath</c>).
        /// Settings are stored alongside other DINOForge persistence files under this directory.
        /// </param>
        public PackSettingsStore(string bepInExRootPath)
        {
            if (string.IsNullOrEmpty(bepInExRootPath))
                throw new ArgumentNullException(nameof(bepInExRootPath), "[PackSettingsStore] BepInEx root path must not be null or empty.");

            _settingsPath = Path.Combine(bepInExRootPath, "dinoforge-pack-settings.json");
            Load();
        }

        /// <summary>
        /// Parameterless constructor retained for unit-test scenarios where the BepInEx
        /// environment is unavailable. Falls back to a temp directory rather than the
        /// game executable directory (avoids writing alongside the game EXE).
        /// Do NOT use this constructor in BepInEx plugin code — use <see cref="GetOrCreate"/>.
        /// </summary>
        public PackSettingsStore()
        {
            _settingsPath = Path.Combine(Path.GetTempPath(), "DINOForge", "dinoforge-pack-settings.json");
            Directory.CreateDirectory(Path.GetDirectoryName(_settingsPath)!);
            Load();
        }

        /// <summary>
        /// Sets the logger for diagnostic output.
        /// </summary>
        public void SetLogger(ManualLogSource log)
        {
            _log = log;
        }

        // ── Public API ───────────────────────────────────────────────────────────

        /// <summary>
        /// Gets a setting value for a pack, with fallback to a default if not set.
        /// </summary>
        /// <typeparam name="T">The expected type of the value.</typeparam>
        /// <param name="packId">Unique pack identifier.</param>
        /// <param name="key">Setting key within the pack.</param>
        /// <param name="defaultValue">Fallback value if the setting is not found.</param>
        /// <returns>The setting value cast to T, or defaultValue if not found or cast fails.</returns>
        public T Get<T>(string packId, string key, T defaultValue)
        {
            lock (_settingsLock)
            {
                if (!_settings.TryGetValue(packId, out var packSettings))
                {
                    return defaultValue;
                }

                if (!packSettings.TryGetValue(key, out var value))
                {
                    return defaultValue;
                }

                // Attempt safe cast
                try
                {
                    if (value is T typedValue)
                    {
                        return typedValue;
                    }

                    // Try JSON round-trip for numeric conversions (e.g., long to float)
                    if (value is JsonElement je)
                    {
                        return JsonSerializer.Deserialize<T>(je.GetRawText(), RuntimeJsonOptions.PackSettings) ?? defaultValue;
                    }

                    return defaultValue;
                }
                catch (Exception ex)
                {
                    _log?.LogWarning($"[PackSettingsStore] Failed to cast {packId}/{key} to {typeof(T).Name}: {ex.Message}");
                    return defaultValue;
                }
            }
        }

        /// <summary>
        /// Sets a setting value for a pack and persists it to disk.
        /// </summary>
        /// <param name="packId">Unique pack identifier.</param>
        /// <param name="key">Setting key within the pack.</param>
        /// <param name="value">The value to store.</param>
        public void Set(string packId, string key, object value)
        {
            if (string.IsNullOrEmpty(packId) || string.IsNullOrEmpty(key))
            {
                _log?.LogWarning($"[PackSettingsStore] Set called with empty packId or key");
                return;
            }

            lock (_settingsLock)
            {
                if (!_settings.TryGetValue(packId, out var packSettings))
                {
                    packSettings = new Dictionary<string, object>(StringComparer.Ordinal);
                    _settings[packId] = packSettings;
                }

                packSettings[key] = value;
                _log?.LogDebug($"[PackSettingsStore] Set {packId}/{key} = {value}");
            }

            Save();
        }

        /// <summary>
        /// Checks whether a pack has any settings configured.
        /// </summary>
        public bool HasPack(string packId)
        {
            lock (_settingsLock)
            {
                return _settings.ContainsKey(packId);
            }
        }

        /// <summary>
        /// Clears all settings for a pack.
        /// </summary>
        public void ClearPack(string packId)
        {
            lock (_settingsLock)
            {
                _settings.Remove(packId);
            }
            Save();
        }

        // ── I/O ──────────────────────────────────────────────────────────────────

        /// <summary>
        /// Loads settings from disk. Called automatically on construction.
        /// </summary>
        private void Load()
        {
            lock (_settingsLock)
            {
                _settings.Clear();

                if (!File.Exists(_settingsPath))
                {
                    _log?.LogDebug($"[PackSettingsStore] No settings file at {_settingsPath}, starting with empty store");
                    return;
                }

                try
                {
                    var json = File.ReadAllText(_settingsPath);
                    var data = JsonSerializer.Deserialize<Dictionary<string, Dictionary<string, object>>>(json, RuntimeJsonOptions.PackSettings);

                    if (data != null)
                    {
                        foreach (var kvp in data)
                        {
                            _settings[kvp.Key] = new Dictionary<string, object>(kvp.Value, StringComparer.Ordinal);
                        }
                        _log?.LogDebug($"[PackSettingsStore] Loaded {_settings.Count} packs from {_settingsPath}");
                    }
                }
                catch (Exception ex)
                {
                    _log?.LogWarning($"[PackSettingsStore] Failed to load settings: {ex.Message}");
                }
            }
        }

        /// <summary>
        /// Saves settings to disk. Called automatically after any Set operation.
        /// </summary>
        private void Save()
        {
            lock (_settingsLock)
            {
                try
                {
                    var json = JsonSerializer.Serialize(_settings, RuntimeJsonOptions.PackSettings);
                    File.WriteAllText(_settingsPath, json);
                    _log?.LogDebug($"[PackSettingsStore] Saved settings to {_settingsPath}");
                }
                catch (Exception ex)
                {
                    _log?.LogWarning($"[PackSettingsStore] Failed to save settings: {ex.Message}");
                }
            }
        }

        // ── IDisposable ──────────────────────────────────────────────────────────

        /// <summary>
        /// Disposes the settings store and ensures settings are saved.
        /// </summary>
        public void Dispose()
        {
            Save();
        }
    }
}
