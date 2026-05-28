using System.Collections.Generic;
using YamlDotNet.Serialization;

namespace DINOForge.SDK.Patching
{
    /// <summary>
    /// A single patch operation to apply to a target pack's content.
    /// Follows RimWorld-style PatchOperations semantics: declarative, path-based mutations
    /// that let one pack modify another pack's definitions without forking them.
    /// </summary>
    public sealed class PatchOperation
    {
        /// <summary>
        /// The operation type. Valid values: replace, set, add, remove, multiply.
        /// - replace / set: Set the value at <see cref="Path"/> to <see cref="Value"/>.
        /// - add: Append <see cref="Value"/> to the list at <see cref="Path"/>, or set a new key.
        /// - remove: Delete the key or list element at <see cref="Path"/>.
        /// - multiply: Multiply the numeric value at <see cref="Path"/> by <see cref="Factor"/>.
        /// </summary>
        [YamlMember(Alias = "op")]
        public string Op { get; set; } = "replace";

        /// <summary>
        /// JSONPath-like selector for the target field.
        /// Supported syntax:
        ///   /units/{id}/stats/hp              — concrete key path
        ///   /units/*/stats/hp                 — wildcard: applies to all entries under 'units'
        ///   /factions/{id}/...                — any top-level content section
        /// Path segments are separated by '/'. A leading '/' is required.
        /// </summary>
        [YamlMember(Alias = "path")]
        public string Path { get; set; } = "";

        /// <summary>
        /// The value to set or append. Required for replace/set/add operations.
        /// May be a scalar (string, int, double, bool) or a list.
        /// </summary>
        [YamlMember(Alias = "value")]
        public object? Value { get; set; }

        /// <summary>
        /// Multiplication factor for the 'multiply' operation.
        /// The target field must resolve to a numeric value.
        /// </summary>
        [YamlMember(Alias = "factor")]
        public double? Factor { get; set; }
    }

    /// <summary>
    /// A collection of <see cref="PatchOperation"/>s targeting a single other pack.
    /// Declared inside the <c>patches:</c> section of a pack's <c>pack.yaml</c>.
    /// </summary>
    public sealed class PatchSet
    {
        /// <summary>
        /// The pack ID whose loaded content dictionary should be patched.
        /// If the target pack is not loaded, this PatchSet is silently skipped.
        /// </summary>
        [YamlMember(Alias = "target_pack")]
        public string TargetPack { get; set; } = "";

        /// <summary>
        /// Operations to apply in order. If any operation fails the rest in this set are skipped.
        /// </summary>
        [YamlMember(Alias = "operations")]
        public List<PatchOperation> Operations { get; set; } = new List<PatchOperation>(); // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet
    }
}
