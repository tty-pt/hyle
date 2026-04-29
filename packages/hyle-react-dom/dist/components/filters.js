import { jsx as _jsx, jsxs as _jsxs, Fragment as _Fragment } from "react/jsx-runtime";
import { useId } from "react";
// ── FilterString ──────────────────────────────────────────────────────────────
export function FilterString({ label, value, onChange }) {
    return (_jsx("input", { type: "text", "aria-label": label, placeholder: label, value: value, onChange: (e) => onChange(e.target.value) }));
}
// ── FilterNumber ──────────────────────────────────────────────────────────────
export function FilterNumber({ label, value, onChange }) {
    return (_jsx("input", { type: "number", step: "any", "aria-label": label, placeholder: label, value: value, onChange: (e) => onChange(e.target.value) }));
}
export function FilterBoolean({ label, value, onChange, appearance = "checkbox" }) {
    if (appearance === "select") {
        const strVal = value === true ? "true" : value === false ? "false" : "";
        return (_jsxs("select", { "aria-label": label, value: strVal, onChange: (e) => {
                const v = e.target.value;
                onChange(v === "true" ? true : v === "false" ? false : undefined);
            }, children: [_jsx("option", { value: "", children: "Any" }), _jsx("option", { value: "true", children: "Yes" }), _jsx("option", { value: "false", children: "No" })] }));
    }
    // Default: checkbox — unchecked = undefined (any), checked = true
    return (_jsx("input", { type: "checkbox", "aria-label": label, checked: value === true, onChange: (e) => onChange(e.target.checked ? true : undefined) }));
}
export function FilterReference({ label, value, field, result, onChange, appearance = "select", }) {
    const listId = useId();
    // Extract lookup options from result
    const options = [];
    if (result && field.type.kind === "reference") {
        const { entity, displayField } = field.type.reference;
        const lookupMap = result.lookups?.[entity];
        if (lookupMap) {
            for (const [id, row] of Object.entries(lookupMap)) {
                options.push({ id, label: String(row[displayField] ?? id) });
            }
        }
    }
    if (appearance === "autocomplete") {
        return (_jsxs(_Fragment, { children: [_jsx("input", { type: "text", "aria-label": label, placeholder: label, value: value, list: listId, onChange: (e) => onChange(e.target.value) }), _jsx("datalist", { id: listId, children: options.map((opt) => (_jsx("option", { value: opt.id, children: opt.label }, opt.id))) })] }));
    }
    // Default: select
    return (_jsxs("select", { "aria-label": label, value: value, onChange: (e) => onChange(e.target.value), children: [_jsx("option", { value: "", children: "Any" }), options.map((opt) => (_jsx("option", { value: opt.id, children: opt.label }, opt.id)))] }));
}
// ── FilterFile ────────────────────────────────────────────────────────────────
export function FilterFile({ label, value, onChange }) {
    return (_jsx("input", { type: "text", "aria-label": label, placeholder: label, value: value, onChange: (e) => onChange(e.target.value) }));
}
//# sourceMappingURL=filters.js.map