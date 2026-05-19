#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using DINOForge.SDK.Models;

namespace DINOForge.SDK
{
    /// <summary>
    /// Manages vanilla → mod asset replacement mappings.
    /// Tracks texture, audio, and UI asset swaps for total conversion packs.
    /// Falls back to vanilla assets when mod asset is missing.
    /// </summary>
    public sealed class AssetReplacementEngine
    {
        private readonly Dictionary<string, string> _textureMap = new(StringComparer.OrdinalIgnoreCase);
        private readonly Dictionary<string, string> _audioMap = new(StringComparer.OrdinalIgnoreCase);
        private readonly Dictionary<string, string> _uiMap = new(StringComparer.OrdinalIgnoreCase);
        private string _assetRoot = "";

        /// <summary>Total number of registered replacements.</summary>
        public int TotalMappings => _textureMap.Count + _audioMap.Count + _uiMap.Count;

        /// <summary>
        /// Loads asset replacement mappings from a total conversion manifest.
        /// </summary>
        public void LoadFromManifest(TotalConversionManifest manifest, string packRootPath)
        {
            _assetRoot = packRootPath;

            foreach (var kvp in manifest.AssetReplacements.Textures)
                _textureMap[kvp.Key] = kvp.Value;

            foreach (var kvp in manifest.AssetReplacements.Audio)
                _audioMap[kvp.Key] = kvp.Value;

            foreach (var kvp in manifest.AssetReplacements.Ui)
                _uiMap[kvp.Key] = kvp.Value;
        }

        /// <summary>
        /// Resolves the path to use for a texture asset.
        /// Returns the mod path if mapped and file exists, otherwise returns vanilla path.
        /// </summary>
        public string ResolveTexture(string vanillaPath)
            => Resolve(_textureMap, vanillaPath, "texture");

        /// <summary>
        /// Resolves the path to use for an audio asset.
        /// </summary>
        public string ResolveAudio(string vanillaPath)
            => Resolve(_audioMap, vanillaPath, "audio");

        /// <summary>
        /// Resolves the path to use for a UI asset.
        /// </summary>
        public string ResolveUi(string vanillaPath)
            => Resolve(_uiMap, vanillaPath, "ui");

        /// <summary>
        /// Returns all registered texture replacements.
        /// </summary>
        public IReadOnlyDictionary<string, string> GetTextureMap() => _textureMap;

        /// <summary>
        /// Returns all registered audio replacements.
        /// </summary>
        public IReadOnlyDictionary<string, string> GetAudioMap() => _audioMap;

        /// <summary>
        /// Returns all registered UI replacements.
        /// </summary>
        public IReadOnlyDictionary<string, string> GetUiMap() => _uiMap;

        /// <summary>
        /// Clears all registered replacements.
        /// </summary>
        public void Clear()
        {
            _textureMap.Clear();
            _audioMap.Clear();
            _uiMap.Clear();
        }

        private string Resolve(Dictionary<string, string> map, string vanillaPath, string assetType)
        {
            if (!map.TryGetValue(vanillaPath, out string? modPath))
                return vanillaPath; // No replacement registered — use vanilla

            // If the replacement is a full path, use it directly
            if (Path.IsPathRooted(modPath))
                return File.Exists(modPath) ? modPath : vanillaPath;

            // Otherwise resolve relative to pack root
            string fullModPath = string.IsNullOrEmpty(_assetRoot)
                ? modPath
                : Path.Combine(_assetRoot, modPath);

            return File.Exists(fullModPath) ? fullModPath : vanillaPath;
        }
    }
}
