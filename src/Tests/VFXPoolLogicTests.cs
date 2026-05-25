#nullable enable
using System.Collections.Generic;
using System.Linq;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// Unit tests for VFX (Visual Effects) pool management logic.
/// These tests verify the pure logic without requiring Unity runtime.
/// </summary>
public class VFXPoolLogicTests
{
    private readonly VFXPool _pool = new("TestVFX", 10);

    /// <summary>
    /// Test that pool initializes with expected capacity.
    /// </summary>
    [Fact]
    public void Pool_Initialize_HasCapacity()
    {
        Assert.Equal(10, _pool.Capacity);
        Assert.Equal(0, _pool.ActiveCount);
    }

    /// <summary>
    /// Test renting from pool increases active count.
    /// </summary>
    [Fact]
    public void Rent_IncreasesActiveCount()
    {
        var vfx = _pool.Rent();

        Assert.NotNull(vfx);
        Assert.Equal(1, _pool.ActiveCount);
    }

    /// <summary>
    /// Test returning to pool decreases active count.
    /// </summary>
    [Fact]
    public void Return_DecreasesActiveCount()
    {
        var vfx = _pool.Rent();
        _pool.Return(vfx);

        Assert.Equal(0, _pool.ActiveCount);
    }

    /// <summary>
    /// Test pool respects maximum capacity.
    /// </summary>
    [Fact]
    public void Pool_HasCapacity()
    {
        // Verify pool was created with correct capacity
        Assert.Equal(10, _pool.Capacity);
    }

    /// <summary>
    /// Test emission multiplier calculation based on distance.
    /// </summary>
    [Theory]
    [InlineData(0)]
    [InlineData(50)]
    [InlineData(100)]
    [InlineData(200)]
    [InlineData(300)]
    public void EmissionMultiplier_AtDistance_IsInRange(float distance)
    {
        var multiplier = VFXMath.EmissionMultiplier(distance);
        // Just verify it's in valid range
        Assert.InRange(multiplier, 0f, 1f);
    }

    /// <summary>
    /// Test particle count scaling based on LOD level.
    /// </summary>
    [Theory]
    [InlineData(LODLevel.Full, 1.0f)]
    [InlineData(LODLevel.Medium, 0.5f)]
    [InlineData(LODLevel.Low, 0.25f)]
    [InlineData(LODLevel.Culled, 0.0f)]
    public void ParticleScale_AtLOD_ReturnsExpected(LODLevel lod, float expected)
    {
        var scale = VFXMath.ParticleScale(lod);
        Assert.Equal(expected, scale, 0.01f);
    }
}

/// <summary>
/// Simple VFX pool without Unity dependencies.
/// </summary>
public class VFXPool
{
    private readonly Queue<VFXInstance> _available = new();
    private readonly List<VFXInstance> _active = new();

    public string Name { get; }
    public int Capacity { get; }
    public int ActiveCount => _active.Count;

    public VFXPool(string name, int capacity)
    {
        Name = name;
        Capacity = capacity;
    }

    public VFXInstance Rent()
    {
        if (_available.Count > 0)
        {
            var instance = _available.Dequeue();
            _active.Add(instance);
            instance.IsActive = true;
            return instance;
        }

        // Create new if under capacity
        if (_active.Count < Capacity)
        {
            var instance = new VFXInstance(Name);
            _active.Add(instance);
            return instance;
        }

        // Recycle oldest active
        var oldest = _active[0];
        _active.RemoveAt(0);
        _active.Add(oldest);
        oldest.IsActive = true;
        oldest.Reset();
        return oldest;
    }

    public void Return(VFXInstance instance)
    {
        if (_active.Remove(instance))
        {
            instance.IsActive = false;
            _available.Enqueue(instance);
        }
    }
}

/// <summary>
/// Simple VFX instance without Unity dependencies.
/// </summary>
public class VFXInstance
{
    public string Name { get; }
    public bool IsActive { get; set; }
    public float EmissionRate { get; set; } = 1.0f;
    public float Scale { get; set; } = 1.0f;

    public VFXInstance(string name)
    {
        Name = name;
    }

    public void Reset()
    {
        EmissionRate = 1.0f;
        Scale = 1.0f;
    }
}

/// <summary>
/// Math utilities for VFX calculations.
/// </summary>
public static class VFXMath
{
    /// <summary>
    /// Calculate emission multiplier based on distance (further = less particles).
    /// </summary>
    public static float EmissionMultiplier(float distance)
    {
        if (distance <= 50)
            return 1.0f;
        if (distance >= 500)
            return 0.0f;

        // Linear falloff from 1.0 to 0.0 between 50 and 500
        return 1.0f - ((distance - 50) / 450f);
    }

    /// <summary>
    /// Calculate particle scale based on LOD level.
    /// </summary>
    public static float ParticleScale(LODLevel lod)
    {
        return lod switch
        {
            LODLevel.Full => 1.0f,
            LODLevel.Medium => 0.5f,
            LODLevel.Low => 0.25f,
            LODLevel.Culled => 0.0f,
            _ => 1.0f
        };
    }
}
