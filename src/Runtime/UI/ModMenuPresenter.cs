#nullable enable
using System.Collections.Generic;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Shared mod-menu state and commands used by both IMGUI and UGUI hosts.
    /// </summary>
    public sealed class ModMenuPresenter
    {
        private readonly List<PackDisplayInfo> _packs = new List<PackDisplayInfo>();

        /// <summary>Current pack list.</summary>
        public IReadOnlyList<PackDisplayInfo> Packs => _packs;

        /// <summary>Selected pack index into <see cref="Packs"/>, or -1 if none.</summary>
        public int SelectedIndex { get; private set; } = -1;

        /// <summary>Current status message.</summary>
        public string StatusMessage { get; private set; } = string.Empty;

        /// <summary>Current error count shown in status.</summary>
        public int ErrorCount { get; private set; }

        /// <summary>Current selected pack if selection is valid.</summary>
        public PackDisplayInfo? SelectedPack
        {
            get
            {
                if (!IsValidIndex(SelectedIndex)) return null;
                return _packs[SelectedIndex];
            }
        }

        /// <summary>Replaces the pack list and resets selection.</summary>
        public void SetPacks(IEnumerable<PackDisplayInfo> packs)
        {
            _packs.Clear();
            _packs.AddRange(packs);
            SelectedIndex = _packs.Count > 0 ? 0 : -1;
        }

        /// <summary>Updates status text and error count.</summary>
        public void SetStatus(string message, int errorCount = 0)
        {
            StatusMessage = message;
            ErrorCount = errorCount;
        }

        /// <summary>Selects a pack index; invalid indexes clear selection.</summary>
        public void SelectIndex(int index)
        {
            SelectedIndex = IsValidIndex(index) ? index : -1;
        }

        /// <summary>Toggles enabled state for a pack by index.</summary>
        public bool TryToggleEnabled(int index, out PackDisplayInfo updated)
        {
            updated = null!;
            if (!IsValidIndex(index)) return false;

            PackDisplayInfo current = _packs[index];
            updated = current.WithEnabled(!current.IsEnabled);
            _packs[index] = updated;
            return true;
        }

        /// <summary>Returns true if the index is inside the current pack list.</summary>
        public bool IsValidIndex(int index)
        {
            return index >= 0 && index < _packs.Count;
        }
    }
}
