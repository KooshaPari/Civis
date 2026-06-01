#nullable enable
using System;
using System.IO;
using BepInEx;
using DINOForge.Runtime.Diagnostics;
using UnityEngine;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Applies hardware-style mouse cursors from the Star Wars pack.
    /// The cursor is re-applied on scene transitions (menu + gameplay loads) and can
    /// switch between default and attack/target states for input feedback.
    /// </summary>
    internal static class TcCursorApplicator
    {
        private const string PackageId = "warfare-starwars";
        private const string CursorRelativeDefault = "assets/ui/cursor_default.png";
        private const string CursorRelativeAttack = "assets/ui/cursor_attack.png";
        private const string CursorRelativeTarget = "assets/ui/cursor_target.png";

        private const string DefaultCursorTag = "default";
        private const string AttackCursorTag = "attack";
        private const string TargetCursorTag = "target";

        private static bool _initialized;
        private static string _resolvedPacksDirectory = string.Empty;
        private static string _resolvedPackUiDirectory = string.Empty;

        private static string _activeCursorTag = string.Empty;
        private static bool _isAttackMode;

        private static Texture2D? _defaultCursor;
        private static Texture2D? _attackCursor;
        private static Texture2D? _targetCursor;

        /// <summary>
        /// Apply a state cursor for the current scene on scene transition.
        /// Gameplay and menu scenes both route through this method.
        /// </summary>
        public static void ApplyForScene(string sceneName, string source, string? packsDirectory = null)
        {
            Initialize(packsDirectory);
            if (string.IsNullOrEmpty(_resolvedPackUiDirectory))
            {
                DebugLog.Write("TcCursorApplicator",
                    $"[TcCursorApplicator] {source}: pack ui directory unavailable; scene='{sceneName}'.");
                return;
            }

            bool isMenuScene = IsMenuScene(sceneName);
            Vector2 hotspot = isMenuScene ? MenuHotspot : GameplayHotspot;
            // Gameplay keeps default/tactical states; menu currently uses default cursor for readability.
            CursorRole desired = _isAttackMode ? CursorRole.Attack : CursorRole.Default;
            SetCursor(desired, hotspot, source);
        }

        /// <summary>
        /// Re-apply cursor each update frame if input state changes.
        /// This keeps a dedicated "attack/target" cursor available while pressing buttons.
        /// </summary>
        public static void UpdateFromInput(string sceneName, string? packsDirectory = null)
        {
            Initialize(packsDirectory);
            if (string.IsNullOrEmpty(_resolvedPackUiDirectory))
                return;

            bool shouldAttack = Input.GetMouseButton(0) || Input.GetMouseButton(1);
            if (shouldAttack == _isAttackMode)
                return;

            _isAttackMode = shouldAttack;
            ApplyForScene(sceneName, "UpdateFromInput", _resolvedPacksDirectory);
        }

        private static void Initialize(string? packsDirectory)
        {
            if (_initialized && string.Equals(_resolvedPacksDirectory, packsDirectory ?? string.Empty, StringComparison.Ordinal))
            {
                return;
            }

            _resolvedPacksDirectory = !string.IsNullOrEmpty(packsDirectory)
                ? packsDirectory!
                : Path.Combine(Paths.BepInExRootPath, "dinoforge_packs");
            _resolvedPackUiDirectory = ResolvePackUiDirectory(_resolvedPacksDirectory);
            _initialized = true;

            if (string.IsNullOrEmpty(_resolvedPackUiDirectory))
            {
                DebugLog.Write("TcCursorApplicator",
                    $"[TcCursorApplicator] No cursor source found. Expected: {Path.Combine(_resolvedPacksDirectory, PackageId, "assets/ui")}");
                return;
            }

            _defaultCursor ??= LoadCursorTexture(CursorRelativeDefault, out _);
            _attackCursor ??= LoadCursorTexture(CursorRelativeAttack, out _);
            _targetCursor ??= LoadCursorTexture(CursorRelativeTarget, out _);

            if (_defaultCursor == null && _attackCursor == null && _targetCursor == null)
            {
                DebugLog.Write("TcCursorApplicator",
                    $"[TcCursorApplicator] No cursor textures loaded from '{_resolvedPackUiDirectory}'.");
            }
        }

        private static void SetCursor(CursorRole role, Vector2 hotspot, string source)
        {
            Texture2D? cursorTexture = role switch
            {
                CursorRole.Attack => _attackCursor ?? _defaultCursor ?? _targetCursor,
                CursorRole.Target => _targetCursor ?? _defaultCursor ?? _attackCursor,
                _ => _defaultCursor ?? _attackCursor ?? _targetCursor,
            };

            if (cursorTexture == null)
            {
                return;
            }

            string tag = role switch
            {
                CursorRole.Attack => AttackCursorTag,
                CursorRole.Target => TargetCursorTag,
                _ => DefaultCursorTag,
            };

            if (_activeCursorTag == tag)
            {
                return;
            }

            Cursor.SetCursor(cursorTexture, hotspot, CursorMode.Auto);
            _activeCursorTag = tag;
            DebugLog.Write("TcCursorApplicator",
                $"[TcCursorApplicator] {source}: applied '{tag}' cursor from '{GetSourcePath(role)}'.");
        }

        private static Texture2D? LoadCursorTexture(string relativePath, out bool loaded)
        {
            loaded = false;
            try
            {
                if (string.IsNullOrEmpty(_resolvedPackUiDirectory))
                    return null;

                string fullPath = Path.Combine(_resolvedPackUiDirectory, relativePath.Replace('/', Path.DirectorySeparatorChar));
                if (!File.Exists(fullPath))
                {
                    return null;
                }

                byte[] bytes = File.ReadAllBytes(fullPath);
                Texture2D tex = new Texture2D(2, 2, TextureFormat.ARGB32, mipChain: false)
                {
                    filterMode = FilterMode.Point,
                    wrapMode = TextureWrapMode.Clamp
                };

                if (!tex.LoadImage(bytes))
                {
                    return null;
                }

                loaded = true;
                DebugLog.Write("TcCursorApplicator", $"[TcCursorApplicator] Loaded '{fullPath}'.");
                return tex;
            }
            catch (Exception ex)
            {
                DebugLog.Write("TcCursorApplicator", $"[TcCursorApplicator] Load failed '{relativePath}': {ex.Message}");
                return null;
            }
        }

        private static string GetSourcePath(CursorRole role)
        {
            return role switch
            {
                CursorRole.Attack => CursorRelativeAttack,
                CursorRole.Target => CursorRelativeTarget,
                _ => CursorRelativeDefault,
            };
        }

        private static bool IsMenuScene(string sceneName)
        {
            return string.Equals(sceneName, "MainMenu", StringComparison.OrdinalIgnoreCase)
                || string.Equals(sceneName, "PrimeMenu", StringComparison.OrdinalIgnoreCase)
                || sceneName.IndexOf("menu", StringComparison.OrdinalIgnoreCase) >= 0;
        }

        private static Vector2 GameplayHotspot => new Vector2(2f, 30f);
        private static Vector2 MenuHotspot => new Vector2(2f, 2f);

        private static string ResolvePackUiDirectory(string packsDirectory)
        {
            try
            {
                string candidate = Path.Combine(packsDirectory, PackageId, "assets", "ui");
                if (Directory.Exists(candidate))
                {
                    return candidate;
                }
            }
            catch (Exception ex)
            {
                DebugLog.Write("TcCursorApplicator",
                    $"[TcCursorApplicator] ResolvePackUiDirectory failed: {ex.Message}");
            }

            return string.Empty;
        }

        private enum CursorRole
        {
            Default,
            Attack,
            Target
        }
    }
}
