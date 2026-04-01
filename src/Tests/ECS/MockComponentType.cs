using System;

namespace DINOForge.Tests.ECS
{
    /// <summary>
    /// Mock implementation of Unity.Entities.ComponentType for testing.
    /// Represents a component type with read-only access mode tracking.
    /// </summary>
    public struct MockComponentType : IEquatable<MockComponentType>
    {
        public Type ComponentType { get; }
        public bool IsReadOnly { get; }

        public MockComponentType(Type componentType, bool isReadOnly = false)
        {
            if (componentType == null)
                throw new ArgumentNullException(nameof(componentType));

            ComponentType = componentType;
            IsReadOnly = isReadOnly;
        }

        /// <summary>Creates a read-only component type.</summary>
        public static MockComponentType ReadOnly(Type componentType)
            => new MockComponentType(componentType, isReadOnly: true);

        /// <summary>Creates a read-write component type.</summary>
        public static MockComponentType ReadWrite(Type componentType)
            => new MockComponentType(componentType, isReadOnly: false);

        public bool Equals(MockComponentType other)
            => ComponentType == other.ComponentType && IsReadOnly == other.IsReadOnly;

        public override bool Equals(object? obj)
            => obj is MockComponentType other && Equals(other);

        public override int GetHashCode()
            => HashCode.Combine(ComponentType, IsReadOnly);

        public override string ToString()
            => $"{(IsReadOnly ? "RO" : "RW")}<{ComponentType.Name}>";

        public static bool operator ==(MockComponentType left, MockComponentType right)
            => left.Equals(right);

        public static bool operator !=(MockComponentType left, MockComponentType right)
            => !left.Equals(right);
    }
}
