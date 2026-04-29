import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { useHyle } from "@tty-pt/hyle-react";
// ── HyleFormFields ────────────────────────────────────────────────────────────
//
// Renders a labeled input row for every field in a model, using the Filter
// components from a HyleFiltersState.  Skips fields with no Filter component.
export function HyleFormFields({ blueprint: blueprintProp, model, Filter, }) {
    const hyle = useHyle();
    const blueprint = blueprintProp ?? hyle.blueprint;
    if (!blueprint) {
        return _jsx("div", { className: "hyle-form-error", children: "No blueprint provided. Pass a blueprint prop or set one on HyleProvider." });
    }
    const fields = hyle.modelFields(blueprint, model);
    return (_jsx("div", { className: "hyle-form-fields", children: fields.map(({ key, label }) => {
            const Input = Filter[key];
            if (!Input)
                return null;
            return (_jsxs("div", { className: "hyle-field-row", children: [_jsx("label", { className: "hyle-field-label", children: label }), _jsx(Input, {})] }, key));
        }) }));
}
//# sourceMappingURL=form.js.map