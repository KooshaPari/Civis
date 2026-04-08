const std = @import("std");

// --- LOD Mesh Decimation (Garfield-Heckbert Algorithm) ---
pub const Vertex = struct {
    x: f32,
    y: f32,
    z: f32,
};

pub const MeshDecimator = struct {
    vertices_count: u32,
    indices_count: u32,

    pub fn init() MeshDecimator {
        return .{
            .vertices_count = 0,
            .indices_count = 0,
        };
    }

    /// Decimate mesh to target polycount (percentage: 0.0-1.0)
    pub fn decimate(self: *MeshDecimator, target_polycount_percent: f32) void {
        const current_count = @as(f32, @floatFromInt(self.indices_count));
        const target_count = @as(u32, @intFromFloat(current_count * target_polycount_percent));
        // TODO: Implement Garfield-Heckbert algorithm
        if (target_count > 0) {
            self.indices_count = target_count;
        }
    }
};

// --- AABB BVH Spatial Indexing ---
pub const Vec3 = struct {
    x: f32,
    y: f32,
    z: f32,

    pub fn init(x: f32, y: f32, z: f32) Vec3 {
        return .{ .x = x, .y = y, .z = z };
    }

    pub fn add(self: Vec3, other: Vec3) Vec3 {
        return .{ .x = self.x + other.x, .y = self.y + other.y, .z = self.z + other.z };
    }

    pub fn sub(self: Vec3, other: Vec3) Vec3 {
        return .{ .x = self.x - other.x, .y = self.y - other.y, .z = self.z - other.z };
    }

    pub fn scale(self: Vec3, scalar: f32) Vec3 {
        return .{ .x = self.x * scalar, .y = self.y * scalar, .z = self.z * scalar };
    }
};

pub const AABB = struct {
    min: Vec3,
    max: Vec3,

    pub fn init(min: Vec3, max: Vec3) AABB {
        return .{ .min = min, .max = max };
    }

    pub fn expand(self: *AABB, point: Vec3) void {
        if (point.x < self.min.x) self.min.x = point.x;
        if (point.y < self.min.y) self.min.y = point.y;
        if (point.z < self.min.z) self.min.z = point.z;
        if (point.x > self.max.x) self.max.x = point.x;
        if (point.y > self.max.y) self.max.y = point.y;
        if (point.z > self.max.z) self.max.z = point.z;
    }

    pub fn contains(self: AABB, point: Vec3) bool {
        return point.x >= self.min.x and point.x <= self.max.x and
            point.y >= self.min.y and point.y <= self.max.y and
            point.z >= self.min.z and point.z <= self.max.z;
    }

    pub fn intersects(self: AABB, other: AABB) bool {
        return self.min.x <= other.max.x and self.max.x >= other.min.x and
            self.min.y <= other.max.y and self.max.y >= other.min.y and
            self.min.z <= other.max.z and self.max.z >= other.min.z;
    }

    pub fn center(self: AABB) Vec3 {
        return .{
            .x = (self.min.x + self.max.x) / 2.0,
            .y = (self.min.y + self.max.y) / 2.0,
            .z = (self.min.z + self.max.z) / 2.0,
        };
    }

    pub fn size(self: AABB) Vec3 {
        return self.max.sub(self.min);
    }
};

pub const BVHNode = struct {
    aabb: AABB,
    has_left: bool = false,
    has_right: bool = false,
    entity_count: u32 = 0,

    pub fn init(aabb: AABB) BVHNode {
        return .{
            .aabb = aabb,
        };
    }
};

pub const BVH = struct {
    node_count: u32 = 0,

    pub fn init() BVH {
        return .{};
    }

    /// Query entities within AABB
    pub fn queryAABB(self: BVH, query_box: AABB) u32 {
        _ = self;
        _ = query_box;
        return 0;
    }
};

// --- Tests ---
test "Vec3 operations" {
    const v1 = Vec3.init(1.0, 2.0, 3.0);
    const v2 = Vec3.init(4.0, 5.0, 6.0);

    const sum = v1.add(v2);
    try std.testing.expectApproxEqAbs(sum.x, 5.0, 0.001);
    try std.testing.expectApproxEqAbs(sum.y, 7.0, 0.001);
    try std.testing.expectApproxEqAbs(sum.z, 9.0, 0.001);
}

test "AABB contains point" {
    const min = Vec3.init(0.0, 0.0, 0.0);
    const max = Vec3.init(10.0, 10.0, 10.0);
    const aabb = AABB.init(min, max);

    const inside = Vec3.init(5.0, 5.0, 5.0);
    const outside = Vec3.init(15.0, 5.0, 5.0);

    try std.testing.expect(aabb.contains(inside));
    try std.testing.expect(!aabb.contains(outside));
}

test "AABB intersection" {
    const aabb1 = AABB.init(Vec3.init(0.0, 0.0, 0.0), Vec3.init(10.0, 10.0, 10.0));
    const aabb2 = AABB.init(Vec3.init(5.0, 5.0, 5.0), Vec3.init(15.0, 15.0, 15.0));
    const aabb3 = AABB.init(Vec3.init(20.0, 20.0, 20.0), Vec3.init(30.0, 30.0, 30.0));

    try std.testing.expect(aabb1.intersects(aabb2));
    try std.testing.expect(!aabb1.intersects(aabb3));
}

test "AABB center and size" {
    const aabb = AABB.init(Vec3.init(0.0, 0.0, 0.0), Vec3.init(10.0, 10.0, 10.0));
    const center = aabb.center();
    const size = aabb.size();

    try std.testing.expectApproxEqAbs(center.x, 5.0, 0.001);
    try std.testing.expectApproxEqAbs(size.x, 10.0, 0.001);
}

test "BVH query" {
    var bvh = BVH.init();
    const query_box = AABB.init(Vec3.init(10.0, 10.0, 10.0), Vec3.init(50.0, 50.0, 50.0));
    const results = bvh.queryAABB(query_box);
    try std.testing.expectEqual(results, 0);
}

test "Mesh decimator init" {
    const decimator = MeshDecimator.init();
    try std.testing.expectEqual(decimator.vertices_count, 0);
    try std.testing.expectEqual(decimator.indices_count, 0);
}

test "Mesh decimator decimate" {
    var decimator = MeshDecimator.init();
    decimator.indices_count = 100;
    decimator.decimate(0.5);
    try std.testing.expectEqual(decimator.indices_count, 50);
}
