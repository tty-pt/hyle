const std = @import("std");
const raw = @import("raw.zig");
const Value = raw.Value;
const StringHashMap = raw.StringHashMap;

pub const SortType = enum {
    String,
    Numeric,
    Date,
    None,
};

pub const Primitive = enum {
    String,
    Number,
    Boolean,
    File,
};

pub const Reference = struct {
    entity: []const u8,
    display_field: []const u8,
};

pub const FieldType = union(enum) {
    Primitive: struct { primitive: Primitive },
    Reference: struct { reference: Reference },
    Array: struct { item: *const FieldType },
    Shape: struct { fields: StringHashMap(ShapeField) },
};

pub const ShapeField = struct {
    label: []const u8,
    field_type: FieldType,
    options: FieldOptions,
};

pub const InputHint = struct {
    kind: []const u8,
    props: StringHashMap(Value),

    pub fn new(kind: []const u8) InputHint {
        return .{ .kind = kind, .props = StringHashMap(Value).init(std.testing.allocator) };
    }

    pub fn withProp(self: *InputHint, key: []const u8, value: Value) void {
        self.props.put(key, value) catch {};
    }
};

pub const FieldOptions = struct {
    sort: SortType = .String,
    input: ?InputHint = null,
    fixed_value: ?Value = null,
    rule: ?[]const u8 = null,
    metadata: StringHashMap(Value) = undefined,
};

pub const Field = struct {
    label: []const u8,
    field_type: FieldType,
    options: FieldOptions,

    pub fn new(label: []const u8, field_type: FieldType) Field {
        return .{
            .label = label,
            .field_type = field_type,
            .options = FieldOptions{ .metadata = StringHashMap(Value).init(std.testing.allocator) },
        };
    }

    pub fn string(label: []const u8) Field {
        var f = Field.new(label, .{ .Primitive = .{ .primitive = .String } });
        f.options.input = InputHint.new("text");
        return f;
    }

    pub fn textarea(label: []const u8, rows: u32) Field {
        var f = Field.new(label, .{ .Primitive = .{ .primitive = .String } });
        var hint = InputHint.new("textarea");
        hint.props.put("rows", Value{ .Int = rows }) catch {};
        f.options.input = hint;
        return f;
    }

    pub fn number(label: []const u8) Field {
        var f = Field.new(label, .{ .Primitive = .{ .primitive = .Number } });
        f.options.sort = .Numeric;
        f.options.input = InputHint.new("number");
        return f;
    }

    pub fn boolean(label: []const u8) Field {
        var f = Field.new(label, .{ .Primitive = .{ .primitive = .Boolean } });
        f.options.sort = .None;
        f.options.input = InputHint.new("checkbox");
        return f;
    }

    pub fn file(label: []const u8) Field {
        var f = Field.new(label, .{ .Primitive = .{ .primitive = .File } });
        f.options.sort = .None;
        f.options.input = InputHint.new("file");
        return f;
    }

    pub fn reference(label: []const u8, entity: []const u8) Field {
        var f = Field.new(label, .{ .Reference = .{ .reference = .{ .entity = entity, .display_field = "name" } } });
        f.options.input = InputHint.new("select");
        return f;
    }

    pub fn array(label: []const u8, _item: FieldType) Field {
        _ = _item;
        var f = Field.new(label, .{ .Array = .{ .item = undefined } });
        f.options.sort = .None;
        return f;
    }

    pub fn shape(label: []const u8, fields: StringHashMap(ShapeField)) Field {
        var f = Field.new(label, .{ .Shape = .{ .fields = fields } });
        f.options.sort = .None;
        return f;
    }

    pub fn withSort(self: *Field, sort: SortType) void {
        self.options.sort = sort;
    }

    pub fn withInput(self: *Field, input: InputHint) void {
        self.options.input = input;
    }
};

pub fn makeField(kind: []const u8, label: []const u8, entity: ?[]const u8) Field {
    if (std.mem.eql(u8, kind, "number")) return Field.number(label);
    if (std.mem.eql(u8, kind, "boolean")) return Field.boolean(label);
    if (std.mem.eql(u8, kind, "file")) return Field.file(label);
    if (std.mem.eql(u8, kind, "ref")) return Field.reference(label, entity orelse "");
    return Field.string(label);
}
