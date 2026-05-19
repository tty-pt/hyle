const std = @import("std");
const raw = @import("raw.zig");

const Source = raw.Source;
const Value = raw.Value;
const StringHashMap = raw.StringHashMap;
const ModelResult = raw.ModelResult;
const ModelRows = raw.ModelRows;

pub fn sourceFromJson(json: []const u8, allocator: std.mem.Allocator) !?Source {
    const parsed = try std.json.parseFromSlice(std.json.Value, allocator, json, .{ .allocate = .alloc_always });
    defer parsed.deinit();

    const obj = parsed.value;
    if (obj != .object) return null;

    var source = Source.init(allocator);
    var it = obj.object.iterator();
    while (it.next()) |entry| {
        const model_name = entry.key_ptr.*;
        const model_val = entry.value_ptr.*;

        const rows = try parseModelResult(model_val, allocator);
        try source.put(model_name, rows);
    }
    return source;
}

fn parseModelResult(val: *const std.json.Value, allocator: std.mem.Allocator) !ModelResult {
    if (val.* != .object) return ModelResult.one(Row.init(allocator));

    const result_field = val.object.get("result") orelse return ModelResult.one(Row.init(allocator));
    const total_field = val.object.get("total");

    const total: usize = if (total_field) |t|
        if (t.* == .integer) @intCast(t.integer) else 0
    else
        0;

    if (result_field.* == .object) {
        const row = try jsonToRow(result_field, allocator);
        return ModelResult.one(row);
    }

    if (result_field.* == .array) {
        var rows = std.ArrayList(Row).init(allocator);
        for (result_field.array.items) |item| {
            const row = try jsonToRow(&item, allocator);
            try rows.append(row);
        }
        return ModelResult.many(rows.items);
    }

    return ModelResult.one(Row.init(allocator));
}

fn jsonToRow(val: *const std.json.Value, allocator: std.mem.Allocator) !Row {
    var row = Row.init(allocator);
    if (val.* != .object) return row;

    var it = val.object.iterator();
    while (it.next()) |entry| {
        const v = try jsonValueToHyleValue(entry.value_ptr.*, allocator);
        try row.put(entry.key_ptr.*, v);
    }
    return row;
}

fn jsonValueToHyleValue(val: std.json.Value, allocator: std.mem.Allocator) !Value {
    return switch (val) {
        .null => Value.Null,
        .bool => |b| Value{ .Bool = b },
        .integer => |n| Value{ .Int = @intCast(n) },
        .float => |f| Value{ .Float = f },
        .string => |s| Value{ .String = try allocator.dupe(u8, s) },
        .array => |arr| {
            var items = std.ArrayList(Value).init(allocator);
            for (arr.items) |item| {
                try items.append(try jsonValueToHyleValue(item, allocator));
            }
            return Value{ .Array = try items.toOwnedSlice() };
        },
        .object => |obj| {
            var map = StringHashMap(Value).init(allocator);
            var it = obj.iterator();
            while (it.next()) |entry| {
                const v = try jsonValueToHyleValue(entry.value_ptr.*, allocator);
                try map.put(entry.key_ptr.*, v);
            }
            return Value{ .Map = map };
        },
    };
}
