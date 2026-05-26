const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    // Build shared library
    const lib = b.addSharedLibrary(.{
        .name = "dinoforge_asset_pipeline_zig",
        .root_source_file = b.path("src/root.zig"),
        .target = target,
        .optimize = optimize,
    });
    b.installArtifact(lib);

    // Create test step
    const test_step = b.step("test", "Run unit tests");
    _ = test_step;

    // Note: For this early Zig version, use: zig test src/root.zig
}
