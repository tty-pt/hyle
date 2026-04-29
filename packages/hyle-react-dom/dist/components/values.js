import { jsx as _jsx } from "react/jsx-runtime";
import { useHyle } from "@tty-pt/hyle-react";
export function BooleanValue({ value }) {
    return (_jsx("input", { type: "checkbox", readOnly: true, checked: Boolean(value), "aria-label": String(Boolean(value)) }));
}
/** Reference values are resolved to a display string via hyle.displayValue. */
export function ReferenceValue({ value, column, result, modelName, blueprint }) {
    const hyle = useHyle();
    if (!result)
        return _jsx("span", { children: String(value ?? "") });
    return _jsx("span", { children: hyle.displayValue(blueprint, result, modelName, column.key, value) });
}
//# sourceMappingURL=values.js.map