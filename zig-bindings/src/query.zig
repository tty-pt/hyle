const std = @import("std");
const raw = @import("raw.zig");
const Value = raw.Value;
const StringHashMap = raw.StringHashMap;

pub const Sort = struct {
    field: []const u8,
    ascending: bool,
};

pub const Query = struct {
    model: []const u8,
    select: [][]const u8,
    where_: StringHashMap(Value),
    filters: [][][]const u8,
    page: ?usize,
    per_page: ?usize,
    sort: ?Sort,
    method: ?[]const u8,

    pub fn init(model: []const u8, allocator: std.mem.Allocator) Query {
        return .{
            .model = model,
            .select = &.{},
            .where_ = StringHashMap(Value).init(allocator),
            .filters = &.{},
            .page = null,
            .per_page = null,
            .sort = null,
            .method = null,
        };
    }

    pub fn withSelect(self: *Query, fields: []const []const u8, allocator: std.mem.Allocator) void {
        self.select = tryAllocSlice(allocator, fields);
    }

    pub fn withWhere(self: *Query, field: []const u8, value: Value) void {
        self.where_.put(field, value) catch {};
    }

    pub fn withFilterLayout(self: *Query, rows: []const []const []const u8, allocator: std.mem.Allocator) void {
        _ = allocator;
        self.filters = rows;
    }

    pub fn withPage(self: *Query, page: usize, per_page: usize) void {
        self.page = page;
        self.per_page = per_page;
    }

    pub fn withSort(self: *Query, field: []const u8, ascending: bool) void {
        self.sort = Sort{ .field = field, .ascending = ascending };
    }

    pub fn withMethod(self: *Query, method: []const u8) void {
        self.method = method;
    }
};

pub const Manifest = struct {
    base: []const u8,
    id: ?Value,
    fields: [][]const u8,
    filter: StringHashMap(Value),
    lookups: [][]const u8,
    inlines: [][]const u8,
    page: ?usize,
    per_page: ?usize,
    sort: ?Sort,
    method: ?[]const u8,
    filter_fields: [][][]const u8,
};

pub const MutateInput = struct {
    model: []const u8,
    id: ?Value,
    data: StringHashMap([]const u8),
};

pub fn parseQueryParams(
    query_str: []const u8,
    default_per_page: usize,
    allocator: std.mem.Allocator,
) !struct { page: usize, per_page: usize, filters: StringHashMap([]const u8) } {
    var page: usize = 1;
    var per_page: usize = default_per_page;
    var filters = StringHashMap([]const u8).init(allocator);

    var parts = std.mem.splitScalar(u8, query_str, '&');
    while (parts.next()) |part| {
        if (part.len == 0) continue;
        var kv = std.mem.splitScalar(u8, part, '=');
        const k = kv.first();
        const v = percentDecode(kv.next() orelse "", allocator);

        if (std.mem.eql(u8, k, "page")) {
            page = std.fmt.parseUnsigned(usize, v, 10) catch 1;
            if (page == 0) page = 1;
        } else if (std.mem.eql(u8, k, "per_page")) {
            per_page = std.fmt.parseUnsigned(usize, v, 10) catch default_per_page;
            if (per_page == 0) per_page = 1;
        } else {
            if (v.len > 0) {
                const entry = filters.getOrPut(k) catch continue;
                if (entry.found_existing) {
                    const existing = entry.value_ptr.*;
                    const joined = try std.fmt.allocPrint(allocator, "{s},{s}", .{ existing, v });
                    entry.value_ptr.* = joined;
                } else {
                    entry.value_ptr.* = v;
                }
            }
        }
    }

    return .{ .page = page, .per_page = per_page, .filters = filters };
}

fn percentDecode(s: []const u8, allocator: std.mem.Allocator) []const u8 {
    var result = std.ArrayList(u8).init(allocator);
    var i: usize = 0;
    while (i < s.len) : (i += 1) {
        if (s[i] == '+') {
            result.append(' ') catch {};
        } else if (s[i] == '%' and i + 2 < s.len) {
            const hex = s[i + 1 .. i + 3];
            const byte = std.fmt.parseInt(u8, hex, 16) catch {
                result.append(s[i]) catch {};
                continue;
            };
            result.append(byte) catch {};
            i += 2;
        } else {
            result.append(s[i]) catch {};
        }
    }
    return result.items;
}

fn tryAllocSlice(allocator: std.mem.Allocator, items: []const []const u8) [][]const u8 {
    const result = allocator.alloc([]const u8, items.len) catch return &.{};
    @memcpy(result, items);
    return result;
}

test "parseQueryParams empty" {
    const a = std.testing.allocator;
    const r = try parseQueryParams("", 5, a);
    try std.testing.expectEqual(@as(usize, 1), r.page);
    try std.testing.expectEqual(@as(usize, 5), r.per_page);
}

test "parseQueryParams page and per_page" {
    const a = std.testing.allocator;
    const r = try parseQueryParams("page=3&per_page=10", 5, a);
    try std.testing.expectEqual(@as(usize, 3), r.page);
    try std.testing.expectEqual(@as(usize, 10), r.per_page);
}

test "parseQueryParams filters" {
    const a = std.testing.allocator;
    const r = try parseQueryParams("name=Ali&role=admin", 5, a);
    try std.testing.expectEqualStrings("Ali", r.filters.get("name").?);
    try std.testing.expectEqualStrings("admin", r.filters.get("role").?);
}

test "parseQueryParams percent encoding" {
    const a = std.testing.allocator;
    const r = try parseQueryParams("name=Ali%20Smith", 5, a);
    try std.testing.expectEqualStrings("Ali Smith", r.filters.get("name").?);
}

test "parseQueryParams repeated key joins with comma" {
    const a = std.testing.allocator;
    const r = try parseQueryParams("tags=rust&tags=web", 5, a);
    try std.testing.expectEqualStrings("rust,web", r.filters.get("tags").?);
}
