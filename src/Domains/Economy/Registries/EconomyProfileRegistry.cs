using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.Domains.Economy.Models;

namespace DINOForge.Domains.Economy.Registries
{
    /// <summary>
    /// Registry of economy profile definitions. Supports custom profile registration and lookup.
    /// </summary>
    public class EconomyProfileRegistry
    {
        private readonly Dictionary<string, EconomyProfile> _profiles =
            new Dictionary<string, EconomyProfile>(StringComparer.OrdinalIgnoreCase);

        /// <summary>
        /// All registered economy profiles.
        /// </summary>
        public IReadOnlyList<EconomyProfile> All => _profiles.Values.ToList().AsReadOnly();

        /// <summary>
        /// Number of registered economy profiles.
        /// </summary>
        public int Count => _profiles.Count;

        /// <summary>
        /// Retrieve an economy profile by its identifier.
        /// </summary>
        /// <param name="id">Profile identifier.</param>
        /// <returns>The matching economy profile.</returns>
        /// <exception cref="KeyNotFoundException">Thrown when no profile with the given id exists.</exception>
        public EconomyProfile GetProfile(string id)
        {
            if (_profiles.TryGetValue(id, out EconomyProfile? profile))
                return profile;

            throw new KeyNotFoundException($"No economy profile registered with id '{id}'.");
        }

        /// <summary>
        /// Try to retrieve an economy profile by its identifier.
        /// </summary>
        /// <param name="id">Profile identifier.</param>
        /// <param name="profile">The matching economy profile, or null if not found.</param>
        /// <returns>True if found.</returns>
        public bool TryGetProfile(string id, out EconomyProfile? profile)
        {
            return _profiles.TryGetValue(id, out profile);
        }

        /// <summary>
        /// Check if an economy profile with the given identifier is registered.
        /// </summary>
        /// <param name="id">Profile identifier.</param>
        /// <returns>True if registered.</returns>
        public bool Contains(string id)
        {
            return _profiles.ContainsKey(id);
        }

        /// <summary>
        /// Register a custom economy profile.
        /// </summary>
        /// <param name="profile">The economy profile to register.</param>
        public void Register(EconomyProfile profile)
        {
            if (profile == null) throw new ArgumentNullException(nameof(profile));
            if (string.IsNullOrWhiteSpace(profile.Id)) throw new ArgumentException("Profile ID cannot be empty.", nameof(profile));
            _profiles[profile.Id] = profile;
        }

        /// <summary>
        /// Unregister an economy profile by identifier.
        /// </summary>
        /// <param name="id">Profile identifier.</param>
        /// <returns>True if a profile was removed; false if not found.</returns>
        public bool Unregister(string id)
        {
            return _profiles.Remove(id);
        }
    }
}
