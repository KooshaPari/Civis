#nullable enable
using System;
using System.Collections.Generic;
using UnityEngine;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// IMGUI-based mod menu overlay. Toggle with F10.
    /// Attached to a DontDestroyOnLoad GameObject so it persists across scenes.
    /// Shows loaded packs, enable/disable toggles, search, status badges, and a reload button.
    /// Hosts an inline Settings button that delegates to <see cref="ModSettingsPanel"/>.
    /// </summary>
    public class ModMenuOverlay : MonoBehaviour, IModMenuHost
    {
        private bool _visible;
        private Rect _windowRect = new Rect(20, 20, 560, 640);
        private Vector2 _packListScroll;
        private Vector2 _detailsScroll;
        private string _searchText = "";
        private IModSettingsHost? _settingsPanel;
        private readonly ModMenuPresenter _presenter = new ModMenuPresenter();
        private readonly List<int> _filteredIndices = new List<int>();

        /// <summary>Callback invoked when the user clicks the Reload Packs button.</summary>
        public Action? OnReloadRequested { get; set; }

        /// <summary>Callback invoked when a pack is toggled enabled/disabled (packId, isEnabled).</summary>
        public Action<string, bool>? OnPackToggled { get; set; }

        /// <summary>Whether the overlay is currently visible.</summary>
        public bool IsVisible => _visible;

        /// <summary>The currently selected pack index in the current presenter list, or -1 if none.</summary>
        public int SelectedPackIndex => _presenter.SelectedIndex;

        // ── Public API ─────────────────────────────────────────────────────────────

        /// <summary>
        /// Updates the list of packs displayed in the overlay.
        /// Virtual so adapters can forward to other menu host implementations.
        /// </summary>
        /// <param name="packs">Pack display info objects to show.</param>
        public virtual void SetPacks(IEnumerable<PackDisplayInfo> packs)
        {
            _presenter.SetPacks(packs);
            RebuildFilter();
        }

        /// <summary>
        /// Updates the status bar message.
        /// Virtual so adapters can forward to other menu host implementations.
        /// </summary>
        /// <param name="message">Status text to display.</param>
        /// <param name="errorCount">Number of errors to show in the status bar.</param>
        public virtual void SetStatus(string message, int errorCount = 0)
        {
            _presenter.SetStatus(message, errorCount);
        }

        /// <summary>
        /// Toggles the overlay visibility.
        /// </summary>
        public virtual void Toggle()
        {
            _visible = !_visible;
        }

        /// <inheritdoc />
        public virtual void Show()
        {
            _visible = true;
        }

        /// <inheritdoc />
        public virtual void Hide()
        {
            _visible = false;
        }

        /// <summary>
        /// Wires the settings panel reference so the Settings button inside this overlay can show/hide it.
        /// </summary>
        /// <param name="panel">The settings panel instance.</param>
        public void SetSettingsPanel(IModSettingsHost? panel)
        {
            _settingsPanel = panel;
        }

        // ── Unity lifecycle ────────────────────────────────────────────────────────
        // NOTE: F10 toggling has been moved to RuntimeDriver.Update() so that the
        // key always works regardless of which UI layer is active.
        // Escape is kept here as a convenience when the overlay is already visible.

        private void Update()
        {
            if (_visible && Input.GetKeyDown(KeyCode.Escape))
            {
                _visible = false;
            }
        }

        private void OnGUI()
        {
            if (!_visible) return;

            string title = BuildWindowTitle();
            _windowRect = GUI.Window(9001, _windowRect, DrawWindow, title, DinoForgeStyle.WindowStyle);
        }

        // ── Window drawing ─────────────────────────────────────────────────────────

        private string BuildWindowTitle()
        {
            string base_ = $"DINOForge v{PluginInfo.VERSION}  —  Mod Manager  [{_presenter.Packs.Count} packs]";
            return _presenter.ErrorCount > 0 ? $"{base_}  ⚠ {_presenter.ErrorCount} errors" : base_;
        }

        private void DrawWindow(int windowId)
        {
            GUILayout.BeginVertical();

            // ── Header strip (status + settings button) ──────────────────────────
            DrawHeader();

            GUILayout.Space(6);

            // ── Search field ──────────────────────────────────────────────────────
            DrawSearchField();

            GUILayout.Space(6);

            // ── Main two-column layout ────────────────────────────────────────────
            GUILayout.BeginHorizontal();
            DrawPackList();

            PackDisplayInfo? selected = _presenter.SelectedPack;
            if (selected != null)
            {
                DrawPackDetails(selected);
            }
            else
            {
                GUILayout.BeginVertical(DinoForgeStyle.BoxStyle, GUILayout.MinWidth(300));
                GUILayout.Label("Select a pack to view details.", DinoForgeStyle.BodyLabelStyle);
                GUILayout.EndVertical();
            }

            GUILayout.EndHorizontal();

            GUILayout.Space(6);

            // ── Footer buttons ────────────────────────────────────────────────────
            DrawFooter();

            GUILayout.EndVertical();

            GUI.DragWindow(new Rect(0, 0, _windowRect.width, 22));
        }

        private void DrawHeader()
        {
            GUILayout.BeginHorizontal(DinoForgeStyle.BoxStyle);

            // Pack / error summary
            GUILayout.Label($"Packs: {_presenter.Packs.Count}", DinoForgeStyle.SectionLabelStyle, GUILayout.Width(90));

            GUILayout.Space(6);

            if (_presenter.ErrorCount > 0)
            {
                DinoForgeStyle.StatusBadge($"ERR {_presenter.ErrorCount}", DinoForgeStyle.Error, 60f);
            }
            else
            {
                DinoForgeStyle.StatusBadge("OK", DinoForgeStyle.Success, 40f);
            }

            GUILayout.FlexibleSpace();

            if (!string.IsNullOrEmpty(_presenter.StatusMessage))
            {
                GUILayout.Label(_presenter.StatusMessage, DinoForgeStyle.BodyLabelStyle);
                GUILayout.Space(8);
            }

            // Settings toggle button
            bool settingsActive = _settingsPanel != null && _settingsPanel.IsVisible;
            string settingsLabel = settingsActive ? "▼ Settings" : "▶ Settings";
            if (GUILayout.Button(settingsLabel, DinoForgeStyle.ButtonStyle, GUILayout.Width(90), GUILayout.Height(22)))
            {
                _settingsPanel?.Toggle();
            }

            GUILayout.EndHorizontal();
        }

        private void DrawSearchField()
        {
            GUILayout.BeginHorizontal();
            GUILayout.Label("Search:", DinoForgeStyle.FieldLabelStyle, GUILayout.Width(52));
            string newSearch = GUILayout.TextField(_searchText, GUILayout.ExpandWidth(true));
            if (newSearch != _searchText)
            {
                _searchText = newSearch;
                RebuildFilter();
            }
            if (GUILayout.Button("x", DinoForgeStyle.ButtonStyle, GUILayout.Width(26)))
            {
                _searchText = "";
                RebuildFilter();
            }
            GUILayout.EndHorizontal();
        }

        private void DrawPackList()
        {
            GUILayout.BeginVertical(DinoForgeStyle.BoxStyle, GUILayout.Width(215));
            GUILayout.Label("Loaded Packs", DinoForgeStyle.HeaderStyle);

            _packListScroll = GUILayout.BeginScrollView(_packListScroll, GUILayout.Height(430));

            if (_filteredIndices.Count == 0)
            {
                GUILayout.Label("No packs match.", DinoForgeStyle.BodyLabelStyle);
            }

            foreach (int realIndex in _filteredIndices)
            {
                PackDisplayInfo pack = _presenter.Packs[realIndex];
                bool isSelected = realIndex == _presenter.SelectedIndex;

                if (isSelected)
                {
                    GUILayout.BeginHorizontal(DinoForgeStyle.SelectedRowStyle);
                }
                else
                {
                    GUILayout.BeginHorizontal();
                }

                // Enable toggle
                bool newEnabled = GUILayout.Toggle(pack.IsEnabled, "", GUILayout.Width(18));
                if (newEnabled != pack.IsEnabled)
                {
                    if (_presenter.TryToggleEnabled(realIndex, out PackDisplayInfo updated))
                    {
                        OnPackToggled?.Invoke(updated.Id, updated.IsEnabled);
                    }
                    RebuildFilter();
                }

                // Pack name button
                GUIStyle nameStyle = isSelected ? DinoForgeStyle.PackNameSelectedStyle : DinoForgeStyle.PackNameStyle;
                if (GUILayout.Button(pack.Name, nameStyle, GUILayout.ExpandWidth(true)))
                {
                    _presenter.SelectIndex(realIndex);
                }

                // Status badge
                DrawPackStatusBadge(pack);

                GUILayout.EndHorizontal();
            }

            GUILayout.EndScrollView();
            GUILayout.EndVertical();
        }

        private void DrawPackStatusBadge(PackDisplayInfo pack)
        {
            if (pack.Errors.Count > 0)
            {
                DinoForgeStyle.StatusBadge("ERR", DinoForgeStyle.Error, 30f);
            }
            else if (pack.Conflicts.Count > 0)
            {
                DinoForgeStyle.StatusBadge("CONF", DinoForgeStyle.Warning, 34f);
            }
            else if (pack.IsEnabled)
            {
                DinoForgeStyle.StatusBadge("OK", DinoForgeStyle.Success, 22f);
            }
            else
            {
                DinoForgeStyle.StatusBadge("OFF", DinoForgeStyle.TextMuted, 24f);
            }
        }

        private void DrawPackDetails(PackDisplayInfo pack)
        {
            GUILayout.BeginVertical(DinoForgeStyle.BoxStyle, GUILayout.MinWidth(310));
            GUILayout.Label("Pack Details", DinoForgeStyle.HeaderStyle);

            _detailsScroll = GUILayout.BeginScrollView(_detailsScroll, GUILayout.ExpandHeight(true));

            DrawField("ID", pack.Id);
            DrawField("Name", pack.Name);
            DrawField("Version", pack.Version);
            DrawField("Author", pack.Author);
            DrawField("Type", pack.Type);
            DrawField("Load Order", pack.LoadOrder.ToString());

            if (!string.IsNullOrEmpty(pack.Description))
            {
                GUILayout.Space(6);
                GUILayout.Label("Description", DinoForgeStyle.FieldLabelStyle);
                GUILayout.Label(pack.Description, DinoForgeStyle.BodyLabelStyle);
            }

            if (pack.Dependencies.Count > 0)
            {
                GUILayout.Space(6);
                GUILayout.BeginHorizontal();
                GUILayout.Label("Dependencies", DinoForgeStyle.FieldLabelStyle);
                GUILayout.Space(4);
                DinoForgeStyle.StatusBadge(pack.Dependencies.Count.ToString(), DinoForgeStyle.AccentDim, 22f);
                GUILayout.EndHorizontal();
                foreach (string dep in pack.Dependencies)
                {
                    GUILayout.Label($"  - {dep}", DinoForgeStyle.BodyLabelStyle);
                }
            }

            if (pack.Conflicts.Count > 0)
            {
                GUILayout.Space(6);
                GUILayout.BeginHorizontal();
                GUILayout.Label("Conflicts", DinoForgeStyle.FieldLabelStyle);
                GUILayout.Space(4);
                DinoForgeStyle.StatusBadge(pack.Conflicts.Count.ToString(), DinoForgeStyle.Warning, 22f);
                GUILayout.EndHorizontal();
                foreach (string conflict in pack.Conflicts)
                {
                    GUILayout.Label($"  - {conflict}", DinoForgeStyle.WarningLabelStyle);
                }
            }

            if (pack.Errors.Count > 0)
            {
                GUILayout.Space(6);
                GUILayout.BeginHorizontal();
                GUILayout.Label("Errors", DinoForgeStyle.FieldLabelStyle);
                GUILayout.Space(4);
                DinoForgeStyle.StatusBadge(pack.Errors.Count.ToString(), DinoForgeStyle.Error, 22f);
                GUILayout.EndHorizontal();
                foreach (string error in pack.Errors)
                {
                    GUILayout.Label($"  - {error}", DinoForgeStyle.ErrorLabelStyle);
                }
            }

            GUILayout.EndScrollView();
            GUILayout.EndVertical();
        }

        private static void DrawField(string label, string value)
        {
            GUILayout.BeginHorizontal();
            GUILayout.Label($"{label}:", DinoForgeStyle.FieldLabelStyle, GUILayout.Width(80));
            GUILayout.Label(value, DinoForgeStyle.BodyLabelStyle);
            GUILayout.EndHorizontal();
        }

        private void DrawFooter()
        {
            GUILayout.BeginHorizontal();
            if (GUILayout.Button("Reload Packs", DinoForgeStyle.ButtonStyle, GUILayout.Height(28)))
            {
                OnReloadRequested?.Invoke();
            }
            GUILayout.FlexibleSpace();
            GUILayout.Label("[F10] toggle  |  [Esc] close", DinoForgeStyle.SectionLabelStyle);
            GUILayout.EndHorizontal();
        }

        // ── Filter helpers ─────────────────────────────────────────────────────────

        private void RebuildFilter()
        {
            _filteredIndices.Clear();
            for (int i = 0; i < _presenter.Packs.Count; i++)
            {
                if (PackMatchesSearch(_presenter.Packs[i]))
                    _filteredIndices.Add(i);
            }

            // If the selected pack is no longer in the filtered list, reset selection
            if (_presenter.SelectedIndex >= 0 && !_filteredIndices.Contains(_presenter.SelectedIndex))
            {
                _presenter.SelectIndex(_filteredIndices.Count > 0 ? _filteredIndices[0] : -1);
            }
        }

        private bool PackMatchesSearch(PackDisplayInfo pack)
        {
            if (string.IsNullOrWhiteSpace(_searchText)) return true;
            string lower = _searchText.ToLowerInvariant();
            return pack.Name.ToLowerInvariant().Contains(lower)
                || pack.Id.ToLowerInvariant().Contains(lower);
        }
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // PackDisplayInfo — unchanged public API
    // ─────────────────────────────────────────────────────────────────────────────

    /// <summary>
    /// Immutable display data for a pack in the mod menu.
    /// </summary>
    public sealed class PackDisplayInfo
    {
        /// <summary>The unique pack identifier.</summary>
        public string Id { get; }

        /// <summary>Human-readable pack name.</summary>
        public string Name { get; }

        /// <summary>SemVer version string.</summary>
        public string Version { get; }

        /// <summary>Pack author name.</summary>
        public string Author { get; }

        /// <summary>Pack type (content, balance, ruleset, etc.).</summary>
        public string Type { get; }

        /// <summary>Optional description of the pack.</summary>
        public string? Description { get; }

        /// <summary>Load order priority.</summary>
        public int LoadOrder { get; }

        /// <summary>Whether the pack is currently enabled.</summary>
        public bool IsEnabled { get; }

        /// <summary>Pack dependency IDs.</summary>
        public IReadOnlyList<string> Dependencies { get; }

        /// <summary>Pack conflict IDs.</summary>
        public IReadOnlyList<string> Conflicts { get; }

        /// <summary>Pack-specific error messages.</summary>
        public IReadOnlyList<string> Errors { get; }

        /// <summary>Content type counts (e.g. factions:1, units:3).</summary>
        public IReadOnlyDictionary<string, int> ContentSummary { get; }

        /// <summary>Auto-detected content-type overlaps with other packs.</summary>
        public IReadOnlyList<string> DetectedConflicts { get; }

        /// <summary>
        /// Creates a new pack display info instance.
        /// </summary>
        public PackDisplayInfo(
            string id,
            string name,
            string version,
            string author,
            string type,
            string? description,
            int loadOrder,
            bool isEnabled,
            IReadOnlyList<string> dependencies,
            IReadOnlyList<string> conflicts,
            IReadOnlyList<string>? errors = null,
            IReadOnlyDictionary<string, int>? contentSummary = null,
            IReadOnlyList<string>? detectedConflicts = null)
        {
            Id = id;
            Name = name;
            Version = version;
            Author = author;
            Type = type;
            Description = description;
            LoadOrder = loadOrder;
            IsEnabled = isEnabled;
            Dependencies = dependencies;
            Conflicts = conflicts;
            Errors = errors ?? new List<string>().AsReadOnly();
            ContentSummary = contentSummary ?? (IReadOnlyDictionary<string, int>)new Dictionary<string, int>(StringComparer.Ordinal);
            DetectedConflicts = detectedConflicts ?? (IReadOnlyList<string>)new List<string>().AsReadOnly();
        }

        /// <summary>Returns a copy with the enabled state changed.</summary>
        public PackDisplayInfo WithEnabled(bool enabled)
            => new PackDisplayInfo(Id, Name, Version, Author, Type, Description, LoadOrder, enabled, Dependencies, Conflicts, Errors, ContentSummary, DetectedConflicts);
    }
}
