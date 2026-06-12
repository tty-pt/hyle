const std = @import("std");

pub const Error = error{
    UnknownModel,
    UnknownField,
    UnknownReference,
    EmptySelection,
    MissingBaseModel,
    OutOfMemory,
};

pub fn HyleResult(comptime T: type) type {
    return Error!T;
}
