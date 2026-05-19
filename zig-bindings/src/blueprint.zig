const std = @import("std");
const raw = @import("raw.zig");
const field_mod = @import("field.zig");
const query_mod = @import("query.zig");

const Value = raw.Value;
const StringHashMap = raw.StringHashMap;
const Row = raw.Row;
const Source = raw.Source;
const ModelResult = raw.ModelResult;
const Outcome = raw.Outcome;
const ModelRows = raw.ModelRows;
const Field = field_mod.Field;
const FieldType = field_mod.FieldType;
const SortType = field_mod.SortType;
const Primitive = field_mod.Primitive;
const Reference = field_mod.Reference;
const Query = query_mod.Query;
const Manifest = query_mod.Manifest;
const Sort = query_mod.Sort;
const valueToLookupKey = raw.valueToLookupKey;

pub const Model = struct {
    label: ?[]const u8,
    fields: StringHashMap(Field),

    pub fn init(allocator: std.mem.Allocator) Model {
        return .{ .label = null, .fields = StringHashMap(Field).init(allocator) };
    }

    pub fn withLabel(label: []const u8) Model {
        return .{ .label = label, .fields = StringHashMap(Field).init(std.testing.allocator) };
    }

    pub fn field(self: *Model, name: []const u8, f: Field) void {
        self.fields.put(name, f) catch {};
    }
};

pub const Blueprint = struct {
    models: StringHashMap(Model),

    pub fn init(allocator: std.mem.Allocator) Blueprint {
        return .{ .models = StringHashMap(Model).init(allocator) };
    }

    pub fn model(self: *Blueprint, name: []const u8, m: Model) void {
        self.models.put(name, m) catch {};
    }

    pub fn manifest(self: *const Blueprint, query: *const Query, allocator: std.mem.Allocator) !Manifest {
        const mdl = self.models.get(query.model) orelse return error.UnknownModel;

        const fields: [][]const u8 = if (query.select.len > 0)
            try allocator.dupe([]const u8, query.select)
        else
            try hashMapKeys(mdl.fields, allocator);

        if (fields.len == 0) return error.EmptySelection;

        for (fields) |fname| {
            if (!mdl.fields.contains(fname)) {
                return error.UnknownField;
            }
        }

        const id = query.where_.get("id");

        var filter_map = StringHashMap(Value).init(allocator);
        {
            var it = query.where_.iterator();
            while (it.next()) |entry| {
                if (!std.mem.eql(u8, entry.key_ptr.*, "id")) {
                    try filter_map.put(entry.key_ptr.*, entry.value_ptr.*);
                }
            }
        }

        var explicit_filter_set = StringHashMap(void).init(allocator);
        for (query.filters) |row| {
            for (row) |fname| {
                if (!mdl.fields.contains(fname)) {
                    return error.UnknownField;
                }
                try explicit_filter_set.put(fname, {});
            }
        }

        var lookups_set = StringHashMap(void).init(allocator);
        var inlines_set = StringHashMap(void).init(allocator);

        for (fields) |fname| {
            const f = mdl.fields.get(fname).?;
            try collectReferences(
                self, query.model, fname, &f.field_type,
                explicit_filter_set.contains(fname),
                &lookups_set, &inlines_set, allocator,
            );
        }

        {
            var it = explicit_filter_set.iterator();
            while (it.next()) |entry| {
                const fname = entry.key_ptr.*;
                const already_in_fields = for (fields) |ff| {
                    if (std.mem.eql(u8, ff, fname)) break true;
                } else false;
                if (already_in_fields) continue;

                const f = mdl.fields.get(fname).?;
                try collectReferences(
                    self, query.model, fname, &f.field_type,
                    true, &lookups_set, &inlines_set, allocator,
                );
            }
        }

        const lookups = try stringHashMapKeys(&lookups_set, allocator);
        const inlines = try stringHashMapKeys(&inlines_set, allocator);

        return Manifest{
            .base = query.model,
            .id = id,
            .fields = fields,
            .filter = filter_map,
            .lookups = lookups,
            .inlines = inlines,
            .page = query.page,
            .per_page = query.per_page,
            .sort = query.sort,
            .method = query.method,
            .filter_fields = try allocator.dupe([][]const u8, query.filters),
        };
    }

    pub fn resolve(self: *const Blueprint, man: *const Manifest, source: *const Source, allocator: std.mem.Allocator) !Outcome {
        _ = self;
        const base = source.get(man.base) orelse return error.MissingBaseModel;
        var lookups = StringHashMap(StringHashMap(Row)).init(allocator);

        var all_refs = std.ArrayList([]const u8).init(allocator);
        try all_refs.appendSlice(man.lookups);
        try all_refs.appendSlice(man.inlines);

        for (all_refs.items) |model_name| {
            if (source.get(model_name)) |result| {
                const rows = try result.r_rows(allocator);
                const by_id = try rowsById(rows, allocator);
                try lookups.put(model_name, by_id);
            }
        }

        return Outcome{
            .rows = base.result,
            .total = base.total,
            .lookups = lookups,
        };
    }

    pub fn resolveQuery(self: *const Blueprint, query: *const Query, source: *const Source, allocator: std.mem.Allocator) !struct { Manifest, Outcome, []const Row } {
        const man = try self.manifest(query, allocator);
        const outcome = try self.resolve(&man, source, allocator);
        const rows = try outcome.rows.rows(allocator);
        return .{ man, outcome, rows };
    }
};

fn collectReferences(
    blueprint: *const Blueprint,
    _source_model: []const u8,
    _source_field: []const u8,
    field_type: *const FieldType,
    explicit_need: bool,
    lookups: *StringHashMap(void),
    inlines: *StringHashMap(void),
    allocator: std.mem.Allocator,
) !void {
    switch (field_type.*) {
        .Primitive => {},
        .Reference => |ref| {
            if (!blueprint.models.contains(ref.reference.entity)) {
                return error.UnknownReference;
            }
            if (explicit_need) {
                try lookups.put(ref.reference.entity, {});
            } else {
                try inlines.put(ref.reference.entity, {});
            }
        },
        .Array => |arr| {
            try collectReferences(blueprint, _source_model, _source_field, arr.item, true, lookups, inlines, allocator);
        },
        .Shape => |shape| {
            var it = shape.fields.iterator();
            while (it.next()) |entry| {
                try collectReferences(blueprint, _source_model, entry.key_ptr.*, &entry.value_ptr.field_type, explicit_need, lookups, inlines, allocator);
            }
        },
    }
}

fn rowsById(rows: []const Row, allocator: std.mem.Allocator) !StringHashMap(Row) {
    var map = StringHashMap(Row).init(allocator);
    for (rows) |row| {
        const id_val = row.get("id") orelse continue;
        const key = try valueToLookupKey(id_val, allocator) orelse continue;
        try map.put(key, row);
    }
    return map;
}

fn stringHashMapKeys(set: *const StringHashMap(void), allocator: std.mem.Allocator) ![][]const u8 {
    var keys = std.ArrayList([]const u8).init(allocator);
    var it = set.iterator();
    while (it.next()) |entry| {
        try keys.append(entry.key_ptr.*);
    }
    return keys.toOwnedSlice();
}

fn hashMapKeys(map: anytype, allocator: std.mem.Allocator) ![][]const u8 {
    var keys = std.ArrayList([]const u8).init(allocator);
    var it = map.iterator();
    while (it.next()) |entry| {
        try keys.append(entry.key_ptr.*);
    }
    return keys.toOwnedSlice();
}

test "blueprint manifest empty select" {
    const a = std.testing.allocator;
    var bp = Blueprint.init(a);
    var mdl = Model.init(a);
    mdl.field("name", Field.string("Name"));
    mdl.field("email", Field.string("Email"));
    bp.model("user", mdl);

    var q = Query.init("user", a);
    const m = try bp.manifest(&q, a);
    try std.testing.expectEqualStrings("user", m.base);
    try std.testing.expect(m.fields.len >= 2);
}

test "blueprint manifest unknown model" {
    const a = std.testing.allocator;
    const bp = Blueprint.init(a);
    var q = Query.init("ghost", a);
    try std.testing.expectError(error.UnknownModel, bp.manifest(&q, a));
}

test "blueprint manifest reference classification" {
    const a = std.testing.allocator;
    var bp = Blueprint.init(a);
    var user_mdl = Model.init(a);
    user_mdl.field("name", Field.string("Name"));
    user_mdl.field("role", Field.reference("Role", "role"));
    bp.model("user", user_mdl);

    var role_mdl = Model.init(a);
    role_mdl.field("name", Field.string("Name"));
    bp.model("role", role_mdl);

    var q = Query.init("user", a);
    q.withSelect(&.{ "name", "role" }, a);
    const m = try bp.manifest(&q, a);
    try std.testing.expect(m.inlines.len == 1);
}
