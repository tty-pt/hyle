import type { FilterProps } from "@tty-pt/hyle-react";
export declare function FilterString({ label, value, onChange }: FilterProps<string>): import("react/jsx-runtime").JSX.Element;
export declare function FilterNumber({ label, value, onChange }: FilterProps<string>): import("react/jsx-runtime").JSX.Element;
export type FilterBooleanProps = FilterProps<boolean | undefined> & {
    /**
     * - `"checkbox"` (default) — unchecked = any/unset (`undefined`), checked = yes (`true`)
     * - `"select"` — three-state `<select>`: Any / Yes / No
     */
    appearance?: "checkbox" | "select";
};
export declare function FilterBoolean({ label, value, onChange, appearance }: FilterBooleanProps): import("react/jsx-runtime").JSX.Element;
export type FilterReferenceProps = FilterProps<string> & {
    /**
     * - `"select"` (default) — `<select>` populated from `result.lookups`
     * - `"autocomplete"` — `<input>` paired with a `<datalist>` for browser-native autocomplete
     */
    appearance?: "select" | "autocomplete";
};
export declare function FilterReference({ label, value, field, result, onChange, appearance, }: FilterReferenceProps): import("react/jsx-runtime").JSX.Element;
export declare function FilterFile({ label, value, onChange }: FilterProps<string>): import("react/jsx-runtime").JSX.Element;
//# sourceMappingURL=filters.d.ts.map