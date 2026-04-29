export const field = {
    string(label, options = {}) {
        return {
            label,
            type: { kind: "primitive", primitive: "string" },
            options: { input: { kind: "text" }, ...options },
        };
    },
    number(label, options = {}) {
        return {
            label,
            type: { kind: "primitive", primitive: "number" },
            options: { sort: "numeric", input: { kind: "number" }, ...options },
        };
    },
    boolean(label, options = {}) {
        return {
            label,
            type: { kind: "primitive", primitive: "boolean" },
            options: { sort: "none", input: { kind: "checkbox" }, ...options },
        };
    },
    file(label, options = {}) {
        return {
            label,
            type: { kind: "primitive", primitive: "file" },
            options: { input: { kind: "file" }, ...options },
        };
    },
    ref(label, entity, options = {}) {
        return {
            label,
            type: { kind: "reference", reference: { entity, displayField: "name" } },
            options: { input: { kind: "select" }, ...options },
        };
    },
    array(label, item, options = {}) {
        return {
            label,
            type: { kind: "array", item: item.type },
            options: { ...options },
        };
    },
    shape(label, fields, options = {}) {
        return {
            label,
            type: { kind: "shape", fields },
            options: { ...options },
        };
    },
};
//# sourceMappingURL=helpers.js.map