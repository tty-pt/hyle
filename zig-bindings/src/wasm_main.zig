const std = @import("std");
const raw = @import("raw.zig");
const blueprint = @import("blueprint.zig");
const view = @import("view.zig");
const forma = @import("forma.zig");
const query_mod = @import("query.zig");

export fn hyle_manifest(input_ptr: [*]u8, input_len: usize) usize {
    _ = input_ptr;
    _ = input_len;
    return 0;
}

export fn hyle_resolve(input_ptr: [*]u8, input_len: usize) usize {
    _ = input_ptr;
    _ = input_len;
    return 0;
}

export fn hyle_derive_columns(input_ptr: [*]u8, input_len: usize) usize {
    _ = input_ptr;
    _ = input_len;
    return 0;
}

export fn hyle_display_value(input_ptr: [*]u8, input_len: usize) usize {
    _ = input_ptr;
    _ = input_len;
    return 0;
}

export fn hyle_forma_to_query(input_ptr: [*]u8, input_len: usize) usize {
    _ = input_ptr;
    _ = input_len;
    return 0;
}
