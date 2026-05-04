import type { HyleFieldComponents } from "@tty-pt/hyle-react";
import {
  FilterString,
  FilterNumber,
  FilterBoolean,
  FilterReference,
  FilterArray,
  FilterFile,
} from "./components/filters";
import { BooleanValue, ReferenceValue, ArrayValue } from "./components/values";
import "@tty-pt/hyle/hyle.css";

export { FilterString, FilterNumber, FilterBoolean, FilterReference, FilterArray, FilterFile, FilterAppearanceContext } from "./components/filters";
export type { FilterAppearance } from "./components/filters";
export { BooleanValue, ReferenceValue, ArrayValue } from "./components/values";
export { HyleTable, HyleTableBody, HyleTableFilterBar, HyleTableFilters, HyleTablePanel, HyleTablePagination } from "./components/table";
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
export const hyleDomComponents: HyleFieldComponents = {
  // Filter inputs
  FilterString,
  FilterNumber,
  FilterBoolean,
  FilterReference,
  FilterArray,
  FilterFile,
  // Value display
  Boolean:   BooleanValue,
  Reference: ReferenceValue,
  Array:     ArrayValue,
};
