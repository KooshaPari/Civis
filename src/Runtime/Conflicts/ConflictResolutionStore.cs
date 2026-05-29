#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Text;

namespace DINOForge.Runtime.Conflicts
{
    /// <summary>
    /// Persists per-pack-pair conflict resolution preferences to
    /// <c>BepInEx/dinoforge-conflict-resolutions.json</c>.
    ///
    /// Resolution values:
    /// <list type="bullet">
    ///   <item><term>prefer_self</term><description>Favour the currently-selected pack's definition.</description></item>
    ///   <item><term>prefer_other</term><description>Favour the other pack's definition.</description></item>
    ///   <item><term>merge</term><description>Keep both (no active preference; default / no-op).</description></item>
    /// </list>
    ///
    /// Key format: <c>{packA}__{packB}</c> where the two IDs are sorted alphabetically so
    /// the key is symmetric regardless of which pack is "self".
    /// </summary>
    public sealed class ConflictResolutionStore
    {
        // ── Persistence constants ──────────────────────────────────────────────────
        private const string FileName = "dinoforge-conflict-resolutions.json";

        // ── State ─────────────────────────────────────────────────────────────────
        private readonly string _filePath;
        private readonly Dictionary<string, string> _resolutions;

        // ── Constructor ───────────────────────────────────────────────────────────

        private ConflictResolutionStore(string filePath, Dictionary<string, string> resolutions)
        {
            _filePath = filePath;
            _resolutions = resolutions;
        }

        // ── Factory ───────────────────────────────────────────────────────────────

        /// <summary>
        /// Loads (or creates) the resolution store from the given BepInEx directory.
        /// Never throws — returns an empty store on any I/O or parse error.
        /// </summary>
        /// <param name="bepInExDirectory">
        /// Absolute path to the BepInEx folder
        /// (e.g. <c>G:\SteamLibrary\...\BepInEx</c>).
        /// </param>
        public static ConflictResolutionStore Load(string bepInExDirectory)
        {
            string filePath = Path.Combine(bepInExDirectory, FileName);
            Dictionary<string, string> resolutions = new Dictionary<string, string>(StringComparer.Ordinal);

            try
            {
                if (File.Exists(filePath))
                {
                    string json = File.ReadAllText(filePath, Encoding.UTF8);
                    ParseJson(json, resolutions);
                }
            }
            catch
            {
                // safe-swallow: resolution store load is best-effort; start empty
            }

            return new ConflictResolutionStore(filePath, resolutions);
        }

        // ── Public API ────────────────────────────────────────────────────────────

        /// <summary>
        /// Returns the stored resolution for the given pack pair, or <c>null</c> if none is set
        /// (implying the default "merge" / keep-both behaviour).
        /// </summary>
        public string? GetResolution(string packA, string packB)
        {
            string key = MakeKey(packA, packB);
            return _resolutions.TryGetValue(key, out string? value) ? value : null;
        }

        /// <summary>
        /// Stores a resolution preference for the given pack pair.
        /// Pass <c>"merge"</c> (or <c>null</c>) to remove a previously-stored preference.
        /// Does not persist to disk — call <see cref="Save"/> explicitly.
        /// </summary>
        public void SetResolution(string packA, string packB, string? resolution)
        {
            string key = MakeKey(packA, packB);
            if (string.IsNullOrEmpty(resolution) ||
                string.Equals(resolution, "merge", StringComparison.Ordinal))
            {
                _resolutions.Remove(key);
            }
            else
            {
                _resolutions[key] = resolution!;
            }
        }

        /// <summary>
        /// Saves the current state to disk (overwrites the file).
        /// Never throws — silently no-ops on I/O error.
        /// </summary>
        public void Save()
        {
            try
            {
                string json = SerializeJson(_resolutions);
                File.WriteAllText(_filePath, json, Encoding.UTF8);
            }
            catch
            {
                // safe-swallow: save is best-effort
            }
        }

        // ── Helpers ───────────────────────────────────────────────────────────────

        /// <summary>Returns the canonical symmetric key for a pack pair.</summary>
        private static string MakeKey(string packA, string packB)
        {
            // Sort alphabetically so (A,B) and (B,A) produce the same key.
            return string.Compare(packA, packB, StringComparison.Ordinal) <= 0
                ? packA + "__" + packB
                : packB + "__" + packA;
        }

        /// <summary>
        /// Minimal hand-rolled JSON serialiser — avoids pulling in System.Text.Json
        /// for a netstandard2.0 target (BepInEx Mono runtime).
        /// Output: <c>{ "key": "value", ... }</c> with no indentation.
        /// </summary>
        private static string SerializeJson(Dictionary<string, string> dict)
        {
            StringBuilder sb = new StringBuilder(256);
            sb.Append('{');
            bool first = true;
            foreach (KeyValuePair<string, string> kv in dict)
            {
                if (!first) sb.Append(',');
                first = false;
                sb.Append('"');
                AppendEscaped(sb, kv.Key);
                sb.Append("\":\"");
                AppendEscaped(sb, kv.Value);
                sb.Append('"');
            }
            sb.Append('}');
            return sb.ToString();
        }

        /// <summary>
        /// Minimal hand-rolled JSON parser for the simple flat string-to-string format we write.
        /// Handles the subset: <c>{ "key": "value", ... }</c>.
        /// </summary>
        private static void ParseJson(string json, Dictionary<string, string> target)
        {
            // Strip outer braces and whitespace, then split on "," and parse key:value pairs.
            // This is intentionally minimal — it handles only the format we produce.
            string trimmed = json.Trim();
            if (trimmed.Length < 2) return;
            if (trimmed[0] == '{') trimmed = trimmed.Substring(1);
            if (trimmed[trimmed.Length - 1] == '}') trimmed = trimmed.Substring(0, trimmed.Length - 1);

            // Tokenise: find "key":"value" pairs
            int pos = 0;
            while (pos < trimmed.Length)
            {
                // Skip whitespace and commas
                while (pos < trimmed.Length && (trimmed[pos] == ' ' || trimmed[pos] == '\t' ||
                       trimmed[pos] == '\r' || trimmed[pos] == '\n' || trimmed[pos] == ','))
                    pos++;

                if (pos >= trimmed.Length) break;
                if (trimmed[pos] != '"') break;

                string? key = ReadJsonString(trimmed, ref pos);
                if (key == null) break;

                // Skip whitespace and colon
                while (pos < trimmed.Length && (trimmed[pos] == ' ' || trimmed[pos] == ':'))
                    pos++;

                string? value = ReadJsonString(trimmed, ref pos);
                if (value == null) break;

                target[key] = value;
            }
        }

        /// <summary>
        /// Reads a JSON-quoted string starting at <paramref name="pos"/> (which must point to the
        /// opening quote), advances pos past the closing quote, and returns the unescaped value.
        /// Returns <c>null</c> if the input is malformed.
        /// </summary>
        private static string? ReadJsonString(string json, ref int pos)
        {
            if (pos >= json.Length || json[pos] != '"') return null;
            pos++; // skip opening quote

            StringBuilder sb = new StringBuilder(64);
            while (pos < json.Length)
            {
                char c = json[pos++];
                if (c == '"') return sb.ToString(); // closing quote
                if (c == '\\' && pos < json.Length)
                {
                    char esc = json[pos++];
                    switch (esc)
                    {
                        case '"': sb.Append('"'); break;
                        case '\\': sb.Append('\\'); break;
                        case '/': sb.Append('/'); break;
                        case 'n': sb.Append('\n'); break;
                        case 'r': sb.Append('\r'); break;
                        case 't': sb.Append('\t'); break;
                        default: sb.Append(esc); break;
                    }
                }
                else
                {
                    sb.Append(c);
                }
            }
            return null; // unterminated string
        }

        /// <summary>Appends a JSON-escaped version of <paramref name="s"/> to <paramref name="sb"/>.</summary>
        private static void AppendEscaped(StringBuilder sb, string s)
        {
            foreach (char c in s)
            {
                switch (c)
                {
                    case '"': sb.Append("\\\""); break;
                    case '\\': sb.Append("\\\\"); break;
                    case '\n': sb.Append("\\n"); break;
                    case '\r': sb.Append("\\r"); break;
                    case '\t': sb.Append("\\t"); break;
                    default: sb.Append(c); break;
                }
            }
        }
    }
}
