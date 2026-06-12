const std = @import("std");
const raw = @import("raw.zig");
const field_mod = @import("field.zig");
const blueprint_mod = @import("blueprint.zig");
const query_mod = @import("query.zig");

const Value = raw.Value;
const StringHashMap = raw.StringHashMap;
const Outcome = raw.Outcome;
const Blueprint = blueprint_mod.Blueprint;
const Manifest = query_mod.Manifest;
const Field = field_mod.Field;
const FieldType = field_mod.FieldType;
const Primitive = field_mod.Primitive;
const valueToLookupKey = raw.valueToLookupKey;

pub const Column = struct {
    key: []const u8,
    field: Field,
    label: []const u8,
};

pub fn deriveColumns(blueprint: *const Blueprint, manifest: *const Manifest, allocator: std.mem.Allocator) ![]Column {
    const model = blueprint.models.get(manifest.base) orelse return error.UnknownModel;
    var cols = std.ArrayList(Column).init(allocator);
    for (manifest.fields) |fname| {
        const f = model.fields.get(fname) orelse return error.UnknownField;
        try cols.append(.{ .key = fname, .field = f, .label = f.label });
    }
    return cols.toOwnedSlice();
}

pub fn displayValue(
    blueprint: *const Blueprint,
    outcome: *const Outcome,
    model_name: []const u8,
    field_name: []const u8,
    value: *const Value,
    allocator: std.mem.Allocator,
) ![]const u8 {
    if (value.* == .Null) return "";
    const model = blueprint.models.get(model_name) orelse return valueToDisplayText(value, allocator);
    const f = model.fields.get(field_name) orelse return valueToDisplayText(value, allocator);
    return displayValueForType(blueprint, outcome, model_name, &f.field_type, value, allocator);
}

fn displayValueForType(
    blueprint: *const Blueprint,
    outcome: *const Outcome,
    model_name: []const u8,
    field_type: *const FieldType,
    value: *const Value,
    allocator: std.mem.Allocator,
) ![]const u8 {
    switch (field_type.*) {
        .Reference => |ref| {
            const lookup_key = try valueToLookupKey(value, allocator);
            if (lookup_key) |key| {
                if (outcome.lookups.get(ref.reference.entity)) |lookup| {
                    if (lookup.get(key)) |related_row| {
                        if (related_row.get(ref.reference.display_field)) |display_val| {
                            return valueToDisplayText(&display_val, allocator);
                        }
                    }
                }
            }
            return valueToDisplayText(value, allocator);
        },
        .Primitive => |prim| {
            if (prim.primitive == .Boolean) {
                if (value.* == .Bool) {
                    return if (value.Bool) "Yes" else "No";
                }
            }
            return valueToDisplayText(value, allocator);
        },
        .Array => |arr| {
            if (value.* == .Array) {
                var parts = std.ArrayList(u8).init(allocator);
                for (value.Array, 0..) |v, i| {
                    if (i > 0) try parts.appendSlice(", ");
                    const text = try displayValueForType(blueprint, outcome, model_name, arr.item, &v, allocator);
                    try parts.appendSlice(text);
                }
                return parts.items;
            }
            return valueToDisplayText(value, allocator);
        },
        .Shape => |shape| {
            if (value.* == .Map) {
                var parts = std.ArrayList(u8).init(allocator);
                var it = shape.fields.iterator();
                var first = true;
                while (it.next()) |entry| {
                    const key = entry.key_ptr.*;
                    const sf = entry.value_ptr.*;
                    if (value.Map.get(key)) |sub_val| {
                        if (sub_val == .Null) continue;
                        if (!first) try parts.appendSlice("; ");
                        first = false;
                        const displayed = try displayValueForType(blueprint, outcome, model_name, &sf.field_type, &sub_val, allocator);
                        try std.fmt.format(parts.writer(), "{s}: {s}", .{ sf.label, displayed });
                    }
                }
                return parts.items;
            }
            return valueToDisplayText(value, allocator);
        },
    }
}

pub fn displayValueFromOutcome(outcome: *const Outcome, key: []const u8, val: *const Value, allocator: std.mem.Allocator) ![]const u8 {
    _ = key;
    if (val.* == .String) {
        const s = val.String;
        var it = outcome.lookups.iterator();
        while (it.next()) |lookup_entry| {
            const lookup = lookup_entry.value_ptr.*;
            if (lookup.get(s)) |ref_row| {
                var row_it = ref_row.iterator();
                while (row_it.next()) |row_entry| {
                    if (!std.mem.eql(u8, row_entry.key_ptr.*, "id")) {
                        if (row_entry.value_ptr.* == .String) {
                            return row_entry.value_ptr.String;
                        }
                    }
                }
            }
        }
        return try allocator.dupe(u8, s);
    }
    return valueToDisplayText(val, allocator);
}

pub fn valueToDisplayText(value: *const Value, allocator: std.mem.Allocator) ![]const u8 {
    return switch (value.*) {
        .Null => "",
        .Bool => |b| if (b) "true" else "false",
        .Int => |n| try std.fmt.allocPrint(allocator, "{d}", .{n}),
        .Float => |f| try std.fmt.allocPrint(allocator, "{d}", .{f}),
        .String => |s| try allocator.dupe(u8, s),
        .Bytes => |b| {
            var hex = std.ArrayList(u8).init(allocator);
            for (b) |byte| try std.fmt.format(hex.writer(), "{x:02}", .{byte});
            return hex.items;
        },
        .Array => |arr| {
            var parts = std.ArrayList(u8).init(allocator);
            for (arr, 0..) |v, i| {
                if (i > 0) try parts.appendSlice(", ");
                const text = try valueToDisplayText(&v, allocator);
                try parts.appendSlice(text);
            }
            return parts.items;
        },
        .Map => |m| {
            var parts = std.ArrayList(u8).init(allocator);
            try parts.append('{');
            var it = m.iterator();
            var first = true;
            while (it.next()) |entry| {
                if (!first) try parts.appendSlice(", ");
                first = false;
                const text = try valueToDisplayText(entry.value_ptr, allocator);
                try std.fmt.format(parts.writer(), "{s}={s}", .{ entry.key_ptr.*, text });
            }
            try parts.append('}');
            return parts.items;
        },
    };
}

test "deriveColumns" {
    const a = std.testing.allocator;
    var bp = blueprint_mod.Blueprint.init(a);
    var mdl = blueprint_mod.Model.init(a);
    mdl.field("name", Field.string("Name"));
    bp.model("user", mdl);

    var q = query_mod.Query.init("user", a);
    q.withSelect(&.{"name"}, a);
    const m = try bp.manifest(&q, a);
    const cols = try deriveColumns(&bp, &m, a);
    try std.testing.expectEqual(@as(usize, 1), cols.len);
    try std.testing.expectEqualStrings("name", cols[0].key);
}

test "displayValue simple" {
    const a = std.testing.allocator;
    var bp = blueprint_mod.Blueprint.init(a);
    var mdl = blueprint_mod.Model.init(a);
    mdl.field("name", Field.string("Name"));
    bp.model("user", mdl);

    var outcome = Outcome.empty(a);
    const result = try displayValue(&bp, &outcome, "user", "name", &Value{ .String = "Alice" }, a);
    try std.testing.expectEqualStrings("Alice", result);
}

test "valueToDisplayText" {
    const a = std.testing.allocator;
    try std.testing.expectEqualStrings("", try valueToDisplayText(&@as(Value, .Null), a));
    try std.testing.expectEqualStrings("true", try valueToDisplayText(&Value{ .Bool = true }, a));
    try std.testing.expectEqualStrings("42", try valueToDisplayText(&Value{ .Int = 42 }, a));
    try std.testing.expectEqualStrings("hello", try valueToDisplayText(&Value{ .String = "hello" }, a));
}
