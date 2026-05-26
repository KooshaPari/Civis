using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// Describes a single mapping between a DINOForge SDK model field path
    /// and a DINO ECS component type. This is the atomic unit of the bridge layer.
    /// </summary>
    public sealed class ComponentMapping
    {
        /// <summary>
        /// Full type name of the DINO ECS component (e.g. "Components.Health").
        /// </summary>
        public string EcsComponentType { get; }

        /// <summary>
        /// Dot-separated path into the SDK model (e.g. "unit.stats.hp").
        /// </summary>
        public string SdkModelPath { get; }

        /// <summary>
        /// Optional human-readable description of what this mapping covers.
        /// </summary>
        public string? Description { get; }

        /// <summary>
        /// Name of the specific field to target for stat modifications.
        /// Null means the component is a tag or requires blob-level access.
        /// </summary>
        public string? TargetFieldName { get; }

        /// <summary>
        /// Lazily resolved CLR Type for the ECS component. Null if the type
        /// cannot be found in loaded assemblies (game DLL not present).
        /// </summary>
        public Type? ResolvedType => _resolvedType ?? (_resolvedType = ResolveType());

        private Type? _resolvedType;
        private bool _resolutionAttempted;

        public ComponentMapping(string ecsComponentType, string sdkModelPath,
            string? description = null, string? targetFieldName = null)
        {
            EcsComponentType = ecsComponentType ?? throw new ArgumentNullException(nameof(ecsComponentType));
            SdkModelPath = sdkModelPath ?? throw new ArgumentNullException(nameof(sdkModelPath));
            Description = description;
            TargetFieldName = targetFieldName;
        }

        private Type? ResolveType()
        {
            if (_resolutionAttempted) return null;
            _resolutionAttempted = true;

            foreach (Assembly assembly in AppDomain.CurrentDomain.GetAssemblies())
            {
                try
                {
                    Type? type = assembly.GetType(EcsComponentType, throwOnError: false);
                    if (type != null) return type;
                }
                catch
                {
                    // safe-swallow: assembly type lookup can fail on partially loadable game DLLs; skip this assembly
                }
            }

            return null;
        }

        public override string ToString() => $"{SdkModelPath} -> {EcsComponentType}";
    }
}
