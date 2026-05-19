const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const hyle_inc = b.path("../include");
    const qmap_inc = b.path("../../libqmap/include");
    const hyle_lib = b.path("../lib");
    const qmap_lib = b.path("../../libqmap/lib");

    const is_wasm = target.result.cpu.arch == .wasm32;

    // ---- Native static library (for SSR) ----
    const native_lib = b.addStaticLibrary(.{
        .name = "hyle_zig",
        .root_source_file = b.path("src/lib.zig"),
        .target = target,
        .optimize = optimize,
    });

    if (!is_wasm) {
        native_lib.addIncludePath(hyle_inc);
        native_lib.addIncludePath(qmap_inc);
        native_lib.addLibraryPath(hyle_lib);
        native_lib.addLibraryPath(qmap_lib);
        native_lib.linkSystemLibrary("hyle");
        native_lib.linkSystemLibrary("qmap");
        native_lib.linkLibC();
    }
    b.installArtifact(native_lib);

    // ---- WASM module (for browser client) ----
    const wasm_target = b.resolveTargetQuery(.{
        .cpu_arch = .wasm32,
        .os_tag = .freestanding,
    });
    const wasm_mod = b.addExecutable(.{
        .name = "hyle_wasm",
        .root_source_file = b.path("src/wasm_main.zig"),
        .target = wasm_target,
        .optimize = optimize,
    });
    wasm_mod.entry = .disabled;
    wasm_mod.root_module.export_symbol_names = &.{ "hyle_manifest", "hyle_resolve", "hyle_derive_columns", "hyle_display_value", "hyle_forma_to_query" };
    b.installArtifact(wasm_mod);

    // ---- Tests (native only) ----
    const main_tests = b.addTest(.{
        .root_source_file = b.path("src/lib.zig"),
        .target = target,
        .optimize = optimize,
    });
    if (!is_wasm) {
        main_tests.addIncludePath(hyle_inc);
        main_tests.addIncludePath(qmap_inc);
        main_tests.addLibraryPath(hyle_lib);
        main_tests.addLibraryPath(qmap_lib);
        main_tests.linkSystemLibrary("hyle");
        main_tests.linkSystemLibrary("qmap");
        main_tests.linkLibC();
    }

    const test_step = b.step("test", "Run tests");
    test_step.dependOn(&main_tests.step);
}
