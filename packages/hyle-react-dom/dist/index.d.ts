import type { HyleFieldComponents } from "@tty-pt/hyle-react";
import "./table.css";
export { FilterString, FilterNumber, FilterBoolean, FilterReference, FilterFile } from "./components/filters";
export { BooleanValue, ReferenceValue } from "./components/values";
export { HyleTable, HyleTableBody, HyleTableFilters, HyleTablePagination } from "./components/table";
export { HyleFormFields } from "./components/form";
/**
 * A `HyleFieldComponents` map providing native HTML DOM inputs for each field
 * type. Pass as `components` to `makeHyleHooks`:
 *
 * ```ts
 * const { useList, useFilters } = makeHyleHooks({
 *   blueprint,
 *   useSource,
 *   components: hyleDomComponents,
 * });
 * ```
 */
export declare const hyleDomComponents: HyleFieldComponents;
//# sourceMappingURL=index.d.ts.map