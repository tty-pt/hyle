const std = @import("std");
const raw = @import("raw.zig");

const Source = raw.Source;
const Row = raw.Row;
const Value = raw.Value;
const StringHashMap = raw.StringHashMap;
const ModelResult = raw.ModelResult;

pub const SourceProvider = struct {
    ptr: *anyopaque,
    vtable: *const SourceProviderVTable,

    pub const SourceProviderVTable = struct {
        loadSource: *const fn (ctx: *anyopaque, id: []const u8, allocator: std.mem.Allocator) ?Source,
    };

    pub fn loadSource(self: SourceProvider, id: []const u8, allocator: std.mem.Allocator) ?Source {
        return self.vtable.loadSource(self.ptr, id, allocator);
    }
};

var global_provider: ?SourceProvider = null;

pub fn setProvider(provider: SourceProvider) void {
    global_provider = provider;
}

pub fn getProvider() ?SourceProvider {
    return global_provider;
}

pub fn loadSource(id: []const u8, allocator: std.mem.Allocator) ?Source {
    const provider = getProvider() orelse return null;
    return provider.loadSource(id, allocator);
}

pub fn findItem(source: *const Source, model_id: []const u8, item_id: []const u8) ?Row {
    const mdl = if (std.mem.endsWith(u8, model_id, ".items"))
        model_id[0 .. model_id.len - 6]
    else
        model_id;

    const result = source.get(mdl) orelse return null;
    const rows = result.result.rows() catch return null;
    for (rows) |row| {
        const id_val = row.get("id") orelse continue;
        if (id_val == .String and std.mem.eql(u8, id_val.String, item_id)) {
            return row;
        }
    }
    return null;
}
