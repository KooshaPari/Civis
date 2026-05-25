using System;

namespace DINOForge.Tests.ECS
{
    /// <summary>
    /// Mock implementation of a Unity ECS Entity for testing purposes.
    /// </summary>
    public struct MockEntity : IEquatable<MockEntity>
    {
        public int Index { get; }
        public int Version { get; }

        public MockEntity(int index, int version = 0)
        {
            Index = index;
            Version = version;
        }

        public bool Equals(MockEntity other) => Index == other.Index && Version == other.Version;
        public override bool Equals(object? obj) => obj is MockEntity other && Equals(other);
        public override int GetHashCode() => HashCode.Combine(Index, Version);
        public override string ToString() => $"Entity(index:{Index}, version:{Version})";

        public static bool operator ==(MockEntity left, MockEntity right) => left.Equals(right);
        public static bool operator !=(MockEntity left, MockEntity right) => !left.Equals(right);
    }
}
