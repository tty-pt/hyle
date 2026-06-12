const std = @import("std");
const raw = @import("raw.zig");

const Value = raw.Value;
const Row = raw.Row;
const StringHashMap = raw.StringHashMap;

pub const HyleRowSet = extern struct {
    row_hd: u32,
    fields_hd: u32,
};

pub const HyleFieldFilter = extern struct {
    field: ?[*:0]u8,
    value: ?[*:0]u8,
};

pub const HyleQuery = extern struct {
    sort_field: ?[*:0]u8,
    sort_asc: bool,
    page: u32,
    per_page: u32,
    q: ?[*:0]u8,
    filters: ?*HyleFieldFilter,
    filter_count: u32,
    include: ?*const ?[*:0]u8,
    include_count: u32,
};

pub const HyleSourceRow = extern struct {
    id: ?[*:0]u8,
    field_names: ?*const ?[*:0]u8,
    field_values: ?*const ?[*:0]u8,
    field_count: usize,
};

pub const HyleField = extern struct {
    name: ?[*:0]u8,
    field_type: u32,
    writable: c_int,
    target_source: ?[*:0]u8,
    inverse_name: ?[*:0]u8,
    required: c_int,
    min: i64,
    max: i64,
    min_length: usize,
    max_length: usize,
    pattern: ?[*:0]u8,
};

pub const FieldType = enum(u32) {
    String = 0,
    Int = 1,
    Bool = 2,
    NullableString = 3,
    Reference = 4,
    MultiReference = 5,
    Inverse = 6,
};

pub const FieldDef = struct {
    name: []const u8,
    kind: FieldType,
};

extern fn hyle_source_register(
    source_id: [*:0]u8,
    fields: *const HyleField,
    field_count: usize,
    record_id: u32,
    flags: u32,
    user: ?*anyopaque,
) callconv(.C) u32;

extern fn hyle_source_put(
    source_id: [*:0]u8,
    row_id: [*:0]u8,
    names: *const ?[*:0]u8,
    values: *const ?[*:0]u8,
    count: usize,
) callconv(.C) c_int;

extern fn hyle_source_del(
    source_id: [*:0]u8,
    row_id: [*:0]u8,
) callconv(.C) void;

extern fn hyle_source_query(
    source_id: [*:0]u8,
    query: *const HyleQuery,
    out: *HyleRowSet,
    total_out: *usize,
) callconv(.C) c_int;

extern fn hyle_row_set_free(
    rs: *HyleRowSet,
) callconv(.C) void;

extern fn hyle_row_set_to_rows(
    rs: *const HyleRowSet,
    rows_out: *?*HyleSourceRow,
    count_out: *usize,
) callconv(.C) c_int;

extern fn hyle_source_rows_free(
    rows: *HyleSourceRow,
    count: usize,
) callconv(.C) void;

const Registration = struct {
    id_cs: [:0]u8,
    names: std.ArrayList([:0]u8),
    fields: std.ArrayList(HyleField),
};

pub fn registerSource(source_id: []const u8, fields: []const FieldDef, allocator: std.mem.Allocator) void {
    const id_cs = std.cstr.addNullByte(allocator, source_id) catch return;
    var names = std.ArrayList([:0]u8).init(allocator);
    var hyle_fields = std.ArrayList(HyleField).init(allocator);

    for (fields) |f| {
        const name_cs = std.cstr.addNullByte(allocator, f.name) catch return;
        names.append(name_cs) catch return;
        hyle_fields.append(.{
            .name = name_cs.ptr,
            .field_type = @intFromEnum(f.kind),
            .writable = 0,
            .target_source = null,
            .inverse_name = null,
            .required = 0,
            .min = 0,
            .max = 0,
            .min_length = 0,
            .max_length = 0,
            .pattern = null,
        }) catch return;
    }

    _ = hyle_source_register(
        id_cs.ptr,
        hyle_fields.items.ptr,
        hyle_fields.items.len,
        0, 0, null,
    );
}

pub fn sourcePut(source_id: []const u8, row: *const Row, allocator: std.mem.Allocator) void {
    const id_val = row.get("id") orelse return;
    const row_id_str = switch (id_val.*) {
        .String => |s| s,
        .Int => |n| std.fmt.allocPrint(allocator, "{d}", .{n}) catch return,
        else => return,
    };

    const id_cs = std.cstr.addNullByte(allocator, source_id) catch return;
    const row_cs = std.cstr.addNullByte(allocator, row_id_str) catch return;

    var names = std.ArrayList([:0]u8).init(allocator);
    var values = std.ArrayList([:0]u8).init(allocator);
    var name_ptrs = std.ArrayList(?[*:0]u8).init(allocator);
    var value_ptrs = std.ArrayList(?[*:0]u8).init(allocator);

    var it = row.iterator();
    while (it.next()) |entry| {
        const name_cs = std.cstr.addNullByte(allocator, entry.key_ptr.*) catch return;
        const val_str = valueToCString(entry.value_ptr.*, allocator) catch return;
        const val_cs = std.cstr.addNullByte(allocator, val_str) catch return;
        names.append(name_cs) catch return;
        values.append(val_cs) catch return;
        name_ptrs.append(name_cs.ptr) catch return;
        value_ptrs.append(val_cs.ptr) catch return;
    }

    _ = hyle_source_put(
        id_cs.ptr,
        row_cs.ptr,
        name_ptrs.items.ptr,
        value_ptrs.items.ptr,
        name_ptrs.items.len,
    );
}

pub fn sourceDel(source_id: []const u8, row_id: []const u8, allocator: std.mem.Allocator) void {
    const id_cs = std.cstr.addNullByte(allocator, source_id) catch return;
    const row_cs = std.cstr.addNullByte(allocator, row_id) catch return;
    hyle_source_del(id_cs.ptr, row_cs.ptr);
}

fn valueToCString(v: Value, allocator: std.mem.Allocator) ![]const u8 {
    return switch (v) {
        .String => |s| s,
        .Int => |n| try std.fmt.allocPrint(allocator, "{d}", .{n}),
        .Bool => |b| if (b) "true" else "false",
        .Float => |f| try std.fmt.allocPrint(allocator, "{d}", .{f}),
        .Array => |arr| {
            var parts = std.ArrayList(u8).init(allocator);
            for (arr, 0..) |item, i| {
                if (i > 0) try parts.append('\n');
                if (item == .String) {
                    try parts.appendSlice(item.String);
                } else {
                    try std.fmt.format(parts.writer(), "{any}", .{item});
                }
            }
            return parts.items;
        },
        .Bytes, .Map, .Null => "",
    };
}

pub fn queryBridge(_: *const anyopaque, allocator: std.mem.Allocator) struct { HyleQuery, std.ArrayList([:0]u8) } {
    return .{
        HyleQuery{
            .sort_field = null,
            .sort_asc = true,
            .page = 0,
            .per_page = 0,
            .q = null,
            .filters = null,
            .filter_count = 0,
            .include = null,
            .include_count = 0,
        },
        std.ArrayList([:0]u8).init(allocator),
    };
}
