const std = @import("std");

pub const StringHashMap = std.StringHashMap;

pub const ValueTag = enum {
    Null,
    Bool,
    Int,
    Float,
    String,
    Array,
    Bytes,
    Map,
};

pub const Value = union(ValueTag) {
    Null,
    Bool: bool,
    Int: i64,
    Float: f64,
    String: []const u8,
    Array: []const Value,
    Bytes: []const u8,
    Map: StringHashMap(Value),

    pub fn format(
        self: Value,
        comptime _: []const u8,
        _: std.fmt.FormatOptions,
        writer: anytype,
    ) @TypeOf(writer).Error!void {
        switch (self) {
            .Null => {},
            .Bool => |b| try writer.print("{}", .{b}),
            .Int => |n| try writer.print("{d}", .{n}),
            .Float => |f| try writer.print("{d}", .{f}),
            .String => |s| try writer.writeAll(s),
            .Bytes => |b| {
                for (b) |byte| try writer.print("{x:02}", .{byte});
            },
            .Array => |arr| {
                for (arr, 0..) |v, i| {
                    if (i > 0) try writer.writeAll(", ");
                    try writer.print("{}", .{v});
                }
            },
            .Map => |m| {
                try writer.writeAll("{");
                var it = m.iterator();
                var first = true;
                while (it.next()) |entry| {
                    if (!first) try writer.writeAll(", ");
                    first = false;
                    try writer.print("{s}={}", .{ entry.key_ptr.*, entry.value_ptr.* });
                }
                try writer.writeAll("}");
            },
        }
    }

    pub fn eql(self: Value, other: Value) bool {
        if (@as(ValueTag, self) != @as(ValueTag, other)) return false;
        return switch (self) {
            .Null => true,
            .Bool => |a| a == other.Bool,
            .Int => |a| a == other.Int,
            .Float => |a| a == other.Float,
            .String => |a| std.mem.eql(u8, a, other.String),
            .Bytes => |a| std.mem.eql(u8, a, other.Bytes),
            .Array => |a| {
                const b = other.Array;
                if (a.len != b.len) return false;
                for (a, 0..) |v, i| { if (!v.eql(b[i])) return false; }
                return true;
            },
            .Map => |a| {
                const b = other.Map;
                if (a.count() != b.count()) return false;
                var it = a.iterator();
                while (it.next()) |entry| {
                    const found = b.get(entry.key_ptr.*) orelse return false;
                    if (!entry.value_ptr.eql(found)) return false;
                }
                return true;
            },
        };
    }
};

pub const Row = StringHashMap(Value);
pub const Source = StringHashMap(ModelResult);

pub const ModelRows = union(enum) {
    One: Row,
    Many: []const Row,

    pub fn rows(self: *const ModelRows, allocator: std.mem.Allocator) ![]const Row {
        return switch (self.*) {
            .One => |*row| {
                const result = try allocator.alloc(Row, 1);
                result[0] = row.*;
                return result;
            },
            .Many => |r| r,
        };
    }
};

pub const ModelResult = struct {
    result: ModelRows,
    total: usize,

    pub fn one(row: Row) ModelResult {
        return .{ .result = .{ .One = row }, .total = 1 };
    }

    pub fn many(rows: []const Row) ModelResult {
        return .{ .result = .{ .Many = rows }, .total = rows.len };
    }

    pub fn r_rows(self: *const ModelResult, allocator: std.mem.Allocator) ![]const Row {
        return self.result.rows(allocator);
    }
};

pub const Outcome = struct {
    rows: ModelRows,
    total: usize,
    lookups: StringHashMap(StringHashMap(Row)),

    pub fn rows(self: *const Outcome, allocator: std.mem.Allocator) ![]const Row {
        return self.rows.rows(allocator);
    }

    pub fn empty(allocator: std.mem.Allocator) Outcome {
        return .{
            .rows = .{ .Many = &.{} },
            .total = 0,
            .lookups = StringHashMap(StringHashMap(Row)).init(allocator),
        };
    }
};

pub const RowItem = struct {
    id: []const u8,
    title: []const u8,
    extra: []const struct { []const u8, []const u8 },

    pub fn intoRow(self: RowItem, allocator: std.mem.Allocator) !Row {
        var row = Row.init(allocator);
        try row.put("id", Value{ .String = self.id });
        try row.put("title", Value{ .String = self.title });
        for (self.extra) |pair| {
            try row.put(pair[0], Value{ .String = pair[1] });
        }
        return row;
    }
};

pub fn rowFromForm(form: *const StringHashMap([]const u8), allocator: std.mem.Allocator) !Row {
    var row = Row.init(allocator);
    var it = form.iterator();
    while (it.next()) |entry| {
        try row.put(entry.key_ptr.*, Value{ .String = entry.value_ptr.* });
    }
    return row;
}

pub fn rowFromValue(value: *const Value, allocator: std.mem.Allocator) !Row {
    var row = Row.init(allocator);
    if (value.* == .Map) {
        var it = value.Map.iterator();
        while (it.next()) |entry| {
            try row.put(entry.key_ptr.*, entry.value_ptr.*);
        }
    }
    return row;
}

pub fn rowsFromOutcome(outcome: *const Outcome, allocator: std.mem.Allocator) ![]const Row {
    return outcome.rows.rows(allocator);
}

pub fn isSingle(manifest: *const anyopaque, outcome: *const Outcome) bool {
    _ = manifest;
    return outcome.rows == .One;
}

pub fn valueToLookupKey(value: *const Value, allocator: std.mem.Allocator) !?[]const u8 {
    return switch (value.*) {
        .String => |s| s,
        .Int => |n| try std.fmt.allocPrint(allocator, "{d}", .{n}),
        .Float => |f| try std.fmt.allocPrint(allocator, "{d}", .{f}),
        else => null,
    };
}

pub fn parseIndexItems(body: []const u8, extra_keys: []const []const u8, allocator: std.mem.Allocator) ![]RowItem {
    var items = std.ArrayList(RowItem).init(allocator);
    var lines = std.mem.splitScalar(u8, body, '\n');
    while (lines.next()) |raw_line| {
        const line = std.mem.trim(u8, raw_line, " \r");
        if (line.len == 0) continue;

        if (std.mem.indexOfScalar(u8, line, '\t') != null) {
            var parts = std.mem.splitSequence(u8, line, "\t");
            const id = parts.first();
            const title_s = parts.next() orelse "";
            const title = if (title_s.len == 0) id else title_s;
            var extra = std.ArrayList(struct { []const u8, []const u8 }).init(allocator);
            for (extra_keys) |_| {
                const val = parts.next() orelse "";
                try extra.append(.{ "", val });
            }
            try items.append(.{
                .id = try allocator.dupe(u8, id),
                .title = try allocator.dupe(u8, title),
                .extra = try extra.toOwnedSlice(),
            });
        } else {
            const space_idx = std.mem.indexOfScalar(u8, line, ' ') orelse continue;
            const id = line[0..space_idx];
            const title_s = line[space_idx + 1 ..];
            const title = if (title_s.len == 0) id else title_s;
            try items.append(.{
                .id = try allocator.dupe(u8, id),
                .title = try allocator.dupe(u8, title),
                .extra = try allocator.alloc(struct { []const u8, []const u8 }, 0),
            });
        }
    }
    return items.toOwnedSlice();
}

test "value display" {
    const a = std.testing.allocator;
    _ = a;
    try std.testing.expectFmt("", "{}", .{Value.Null});
    try std.testing.expectFmt("true", "{}", .{Value{ .Bool = true }});
    try std.testing.expectFmt("42", "{}", .{Value{ .Int = 42 }});
    try std.testing.expectFmt("hello", "{}", .{Value{ .String = "hello" }});
}

test "value equality" {
    try std.testing.expect((@as(Value, .Null)).eql(@as(Value, .Null)));
    try std.testing.expect((Value{ .Bool = true }).eql(Value{ .Bool = true }));
    try std.testing.expect((Value{ .Int = 42 }).eql(Value{ .Int = 42 }));
    try std.testing.expect((Value{ .String = "hi" }).eql(Value{ .String = "hi" }));
    try std.testing.expect(!(Value{ .Bool = true }).eql(Value{ .Bool = false }));
}

test "parseIndexItems tab-separated" {
    const allocator = std.testing.allocator;
    const body = "a\tAlice\tactive\nb\tBob\tinactive\n";
    const items = try parseIndexItems(body, &.{"status"}, allocator);
    try std.testing.expectEqual(@as(usize, 2), items.len);
    try std.testing.expectEqualStrings("a", items[0].id);
    try std.testing.expectEqualStrings("Alice", items[0].title);
}

test "parseIndexItems space-separated" {
    const allocator = std.testing.allocator;
    const body = "a Alice\nb Bob\n";
    const items = try parseIndexItems(body, &.{}, allocator);
    try std.testing.expectEqual(@as(usize, 2), items.len);
    try std.testing.expectEqualStrings("a", items[0].id);
    try std.testing.expectEqualStrings("Alice", items[0].title);
}

test "modelRows normalises" {
    const allocator = std.testing.allocator;
    var row = Row.init(allocator);
    try row.put("id", Value{ .Int = 1 });
    const mr = ModelResult.one(row);
    const r = try mr.r_rows(allocator);
    try std.testing.expectEqual(@as(usize, 1), r.len);
}

test "outcome empty" {
    const allocator = std.testing.allocator;
    const o = Outcome.empty(allocator);
    try std.testing.expectEqual(@as(usize, 0), o.total);
}

test "valueToLookupKey" {
    const allocator = std.testing.allocator;
    try std.testing.expectEqualStrings("abc", (try valueToLookupKey(&Value{ .String = "abc" }, allocator)).?);
    try std.testing.expect(try valueToLookupKey(&@as(Value, .Null), allocator) == null);
}
