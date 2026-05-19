const std = @import("std");
const raw = @import("raw.zig");
const query_mod = @import("query.zig");

const Value = raw.Value;
const StringHashMap = raw.StringHashMap;
const Query = query_mod.Query;

pub const FormaFieldType = union(enum) {
    Named: []const u8,
    Array: *const FormaFieldType,
    Shape: []const FormaField,
};

pub const FormaField = struct {
    name: []const u8,
    label: []const u8,
    field_type: FormaFieldType,
    detail_type: ?FormaFieldType,
    form_type: ?FormaFieldType,
    column_type: ?FormaFieldType,
    fixed_value: ?Value,
    rule: ?[]const u8,

    pub fn init(name: []const u8, label: []const u8) FormaField {
        return .{
            .name = name,
            .label = label,
            .field_type = FormaFieldType{ .Named = "string" },
            .detail_type = null,
            .form_type = null,
            .column_type = null,
            .fixed_value = null,
            .rule = null,
        };
    }
};

pub const Forma = struct {
    fields: []const FormaField,
    detail: ?[]const []const u8,
    form: ?[]const []const u8,
    column: ?[]const []const u8,
    filters: ?[]const []const []const u8,
};

pub const FormaContext = enum {
    Detail,
    Form,
    Column,
};

pub fn formaToQuery(
    forma: *const Forma,
    table_name: []const u8,
    context: FormaContext,
    id: ?*const Value,
    allocator: std.mem.Allocator,
) Query {
    const context_names: ?[]const []const u8 = switch (context) {
        .Detail => forma.detail,
        .Form => forma.form,
        .Column => forma.column,
    };

    const active_fields: []const *const FormaField = if (context_names) |names|
        findFields(forma.fields, names, allocator)
    else
        allFieldsRef(forma.fields, allocator);

    var select = std.ArrayList([]const u8).init(allocator);
    for (active_fields) |f| {
        select.append(f.name) catch {};
    }

    var where_ = StringHashMap(Value).init(allocator);
    if (id) |id_val| {
        where_.put("id", id_val.*) catch {};
    }

    var q = Query.init(table_name, allocator);
    q.select = select.items;
    q.where_ = where_;
    if (forma.filters) |filters| {
        q.filters = tryAllocFilterLayout(filters, allocator);
    }
    if (id != null) {
        q.method = "one";
    }
    return q;
}

fn findFields(all_fields: []const FormaField, names: []const []const u8, allocator: std.mem.Allocator) []const *const FormaField {
    var result = std.ArrayList(*const FormaField).init(allocator);
    for (names) |name| {
        for (all_fields) |*f| {
            if (std.mem.eql(u8, f.name, name)) {
                result.addOne() catch continue;
                result.items[result.items.len - 1] = f;
            }
        }
    }
    return result.items;
}

fn allFieldsRef(all_fields: []const FormaField, allocator: std.mem.Allocator) []const *const FormaField {
    var result = std.ArrayList(*const FormaField).init(allocator);
    for (all_fields) |*f| {
        result.addOne() catch continue;
        result.items[result.items.len - 1] = f;
    }
    return result.items;
}

fn tryAllocFilterLayout(layout: []const []const []const u8, allocator: std.mem.Allocator) [][]const u8 {
    var result = std.ArrayList([]const u8).init(allocator);
    for (layout) |row| {
        for (row) |item| {
            result.append(item) catch {};
        }
    }
    return result.items;
}
