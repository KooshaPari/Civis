// #831: Partial-class split scaffold. Methods to migrate: ReadStatFromEcs,
// QueryEntitiesOnMainThread, BuildCatalogSnapshot, GetActiveWorld.
// Migrated: MappingToEntry (phase-2A).
#nullable enable
using DINOForge.Bridge.Protocol;

namespace DINOForge.Runtime.Bridge
{
    public sealed partial class GameBridgeServer
    {
        /// <summary>
        /// Converts a ComponentMapping to a protocol ComponentMapEntry.
        /// </summary>
        private static ComponentMapEntry MappingToEntry(ComponentMapping mapping)
        {
            return new ComponentMapEntry
            {
                SdkPath = mapping.SdkModelPath,
                EcsType = mapping.EcsComponentType,
                FieldName = mapping.TargetFieldName ?? "",
                Resolved = mapping.ResolvedType != null,
                Description = mapping.Description ?? ""
            };
        }
    }
}
