import { type ComponentType, type ReactNode } from "react";
import type { Blueprint, Column, Field, FieldType, Forma, FormaContext, HyleClient, Manifest, PurifyError, Query, Result, HyleResolvedView, Row, Source, TypedBlueprint, TypedModel, TypedField, RowOf } from "@tty-pt/hyle";
import { type QueryKey } from "@tanstack/react-query";
export type { Blueprint, Column, Field, FieldOptions, FieldType, Forma, FormaContext, FormaField, FormaFieldType, Manifest, Model, ModelResult, PurifyError, Query, Result, HyleResolvedView, Row, RowOf, ShapeField, Sort, Source, TypedBlueprint, TypedField, TypedModel, } from "@tty-pt/hyle";
export { createHyleClient, field, } from "@tty-pt/hyle";
export type ValueProps = {
    value: unknown;
    row: Row;
    field: Field;
    column: Column;
    result: Result | null;
    modelName: string;
    blueprint: Blueprint;
};
export type FilterProps<T = unknown> = {
    label: string;
    value: T;
    field: Field;
    fieldName: string;
    /** Resolved lookup data for the current query — used by reference filter inputs. */
    result: Result | null;
    onChange: (value: T) => void;
};
/**
 * A map of React components keyed by FieldType kind (and primitive variant)
 * used to customize rendering of values and filter inputs.
 * Falls back to built-in text rendering via display_value() when not provided.
 */
export type HyleFieldComponents = {
    String?: ComponentType<ValueProps>;
    Number?: ComponentType<ValueProps>;
    Boolean?: ComponentType<ValueProps>;
    File?: ComponentType<ValueProps>;
    Reference?: ComponentType<ValueProps>;
    Array?: ComponentType<ValueProps>;
    Shape?: ComponentType<ValueProps>;
    FilterString?: ComponentType<FilterProps<string>>;
    FilterNumber?: ComponentType<FilterProps<string>>;
    FilterBoolean?: ComponentType<FilterProps<boolean | undefined>>;
    FilterFile?: ComponentType<FilterProps<string>>;
    FilterReference?: ComponentType<FilterProps<string>>;
    FilterArray?: ComponentType<FilterProps<unknown>>;
    FilterShape?: ComponentType<FilterProps<unknown>>;
};
export type HyleStatus = "loading" | "ready" | "error";
export type HyleResolvedState = {
    status: "loading";
    error: null;
    manifest: null;
    result: null;
    rows: Row[];
} | {
    status: "error";
    error: Error;
    manifest: null;
    result: null;
    rows: Row[];
} | {
    status: "ready";
    error: null;
    manifest: Manifest;
    result: Result;
    rows: Row[];
};
export type HyleSourceAdapterResult = Source | {
    source?: Source | null;
    loading?: boolean;
    error?: unknown;
} | null | undefined;
export type HyleSourceAdapter = (manifest: Manifest | null, query: Query) => HyleSourceAdapterResult;
export type HyleValidationAdapter = (input: {
    blueprint: Blueprint;
    query: Query;
    rows: Row[];
    result: Result;
}) => unknown;
export type HyleRowValidationAdapter = (blueprint: Blueprint, modelName: string, row: Row) => PurifyError[] | null;
/** Factory that produces create/update/delete mutation hooks for a given model name. */
export type HyleMutationFactory = (model: string) => {
    useCreate: () => HyleMutation<any, any>;
    useUpdate: () => HyleMutation<any, any>;
    useDelete: () => HyleMutation<any, any>;
};
/**
 * Unified transport adapter: bundles the source (GET) adapter and mutation
 * factory (POST/PUT/DELETE) into one coherent configuration object.
 *
 * Use `makeRestAdapter` from `@tty-pt/hyle-react` to build this from a base URL,
 * or construct it manually for custom transports.
 */
export type HyleAdapter = {
    useSource: HyleSourceAdapter;
    makeMutation?: HyleMutationFactory;
};
export type HyleHooksConfig<TModels extends Record<string, TypedModel<Record<string, TypedField<unknown>>>> = Record<string, TypedModel<Record<string, TypedField<unknown>>>>> = {
    blueprint: TypedBlueprint<TModels> | Blueprint;
    adapter: HyleAdapter;
    validate?: HyleValidationAdapter;
    validateRow?: HyleRowValidationAdapter;
    components?: HyleFieldComponents;
};
type HyleContextValue = {
    client: HyleClient;
    status: HyleStatus;
    error: Error | null;
    blueprint: Blueprint | null;
};
export type UseHyleReturn = {
    status: HyleStatus;
    error: Error | null;
    blueprint: Blueprint | null;
    manifest: (blueprint: Blueprint, query: Query) => Manifest;
    resolve: (blueprint: Blueprint, manifest: Manifest, source: Source) => Result;
    resolveAndView: (blueprint: Blueprint, manifest: Manifest, source: Source) => HyleResolvedView;
    resolveQuery: (blueprint: Blueprint, query: Query, source: Source) => {
        manifest: Manifest;
        result: Result;
        rows: Row[];
    };
    rowsArray: (result: Result) => Row[];
    makeField: (kind: string, label: string, entity?: string) => Field;
    columns: (blueprint: Blueprint, manifest: Manifest) => Column[];
    filterRows: (rows: Row[], filters: Record<string, unknown> | undefined) => Row[];
    applyView: (rows: Row[], manifest: Manifest) => Row[];
    displayValue: (blueprint: Blueprint, result: Result, modelName: string, fieldName: string, value: unknown) => string;
    filterLayout: (blueprint: Blueprint, manifest: Manifest) => Column[][];
    formaToQuery: (forma: Forma, tableName: string, context: FormaContext, id?: unknown) => Query;
    purifyRow: (blueprint: Blueprint, modelName: string, row: Row) => PurifyError[] | null;
    isSingle: (manifest: Manifest, result: Result) => boolean;
    modelFields: (blueprint: Blueprint, modelName: string) => {
        key: string;
        label: string;
        field: Field;
    }[];
};
export declare function HyleProvider({ client, blueprint, children, }: {
    client: HyleClient;
    blueprint?: Blueprint | null;
    children: ReactNode;
}): import("react").FunctionComponentElement<import("react").ProviderProps<HyleContextValue | null>>;
export declare function useHyle(): UseHyleReturn;
export declare function useHyleResolved({ blueprint, query, source, }: {
    blueprint: Blueprint;
    query: Query;
    source: Source | null | undefined;
}): HyleResolvedState;
export type HyleManifestState = {
    status: "loading";
    error: null;
    manifest: null;
} | {
    status: "error";
    error: Error;
    manifest: null;
} | {
    status: "ready";
    error: null;
    manifest: Manifest;
};
export type HyleDataField = {
    key: string;
    label: string;
    field: Field;
    raw: unknown;
    render: () => string;
};
export type HyleDataState<TRow extends Row = Row> = {
    status: "loading";
    error: null;
    manifest: Manifest | null;
    result: null;
    rows: TRow[];
    row: TRow | null;
    fields: HyleDataField[];
    validation: null;
} | {
    status: "error";
    error: Error;
    manifest: Manifest | null;
    result: null;
    rows: TRow[];
    row: TRow | null;
    fields: HyleDataField[];
    validation: null;
} | {
    status: "ready";
    error: null;
    manifest: Manifest;
    result: Result;
    rows: TRow[];
    row: TRow | null;
    fields: HyleDataField[];
    validation: unknown;
};
export type HyleMutation<TInput = Record<string, unknown>, TOutput = unknown> = {
    mutate: (input: TInput) => void;
    reset: () => void;
    isPending: boolean;
    isSuccess: boolean;
    error: Error | null;
    data: TOutput | undefined;
};
export type HyleListState<TRow extends Row = Row> = HyleDataState<TRow> & {
    /** Effective query with page/sort merged in */
    query: Query;
    page: number;
    perPage: number;
    perPageOptions: number[];
    setPage: (page: number) => void;
    setPerPage: (perPage: number) => void;
    sortField: string | undefined;
    sortAscending: boolean;
    setSortField: (field: string | undefined) => void;
    setSortAscending: (ascending: boolean) => void;
};
export type UseListOptions = {
    perPageOptions?: number[];
};
/**
 * The result of a per-field `change` transform.
 * Extends `Field` with an optional per-field filter component override that
 * takes priority over the global `components` map.
 */
export type FieldChangeResult = Field & {
    component?: ComponentType<FilterProps<unknown>>;
};
/**
 * A map of per-field transform functions.
 * Return a `FieldChangeResult` to override label/type/options/component for
 * that field.  Return `null` to exclude the field from the Filter map and
 * from `purifyRow` validation entirely.
 */
export type FieldChangeMap = Record<string, (field: Field) => FieldChangeResult | null>;
export type UseFiltersOptions = {
    /**
     * Per-field transform map.  Each function receives the blueprint `Field` and
     * returns a (possibly modified) `FieldChangeResult`, or `null` to exclude
     * the field entirely.
     */
    change?: FieldChangeMap;
};
export type HyleFiltersState<TRow extends Row = Row> = {
    /** Effective query with committed values merged into where — pass to useData or useList */
    query: Query;
    /** Uncommitted form field values */
    formData: Partial<TRow>;
    /** Update a single field value without committing */
    setField: <K extends keyof TRow & string>(name: K, value: TRow[K]) => void;
    /** Commit formData into where clause → triggers query re-run when passed to useData/useList */
    filterApply: () => void;
    /** Reset formData and committed state */
    filterClear: () => void;
    /** Key that increments on filterClear (force-remounts Filter components) */
    filterResetKey: number;
    /** Pre-built Filter input components keyed by field name */
    Filter: Record<string, ComponentType>;
    /**
     * Run purifyRow validation against current formData and update purifyErrors.
     * Call this before submitting or when you want inline validation in a filter form.
     */
    validate: () => PurifyError[] | null;
    /** Sync validation errors from purify_row (null = valid or not yet validated) */
    purifyErrors: PurifyError[] | null;
};
export type ReadyListState<TRow extends Row = Row> = Extract<HyleListState<TRow>, {
    status: "ready";
}>;
export type TypedFiltersState<TRow extends Row, TFrom extends string> = HyleFiltersState<TRow> & {
    query: Query & {
        model: TFrom;
    };
};
/**
 * Options for `useForm`.
 */
export type UseFormOptions = {
    /**
     * Per-field transform map — same semantics as `UseFiltersOptions.change`.
     */
    change?: FieldChangeMap;
};
/**
 * State returned by `useForm`.
 *
 * Extends `HyleFiltersState` with:
 * - `isEdit` — `true` when the query contains an `id` (edit mode), `false` for create.
 * - `isValid` — `true` when there are no purify errors (either not yet submitted, or last
 *   submission passed validation).
 * - `onSubmit` — validates then fires the appropriate mutation (create or update).
 * - `mutation` — the active mutation handle (null if no `makeMutation` configured).
 */
export type HyleFormState<TRow extends Row = Row> = HyleFiltersState<TRow> & {
    isEdit: boolean;
    isValid: boolean;
    onSubmit: (e?: {
        preventDefault?: () => void;
    }) => void;
    mutation: HyleMutation | null;
};
export type UseFormaOptions = {
    /** Which field subset context to use when deriving select */
    context?: FormaContext;
    /** How many items to show per page */
    selectAmount?: number;
    /** Filter amount for filter layout derivation */
    filterAmount?: number;
};
export declare function makeHyleHooks<TModels extends Record<string, TypedModel<Record<string, TypedField<unknown>>>> = Record<string, TypedModel<Record<string, TypedField<unknown>>>>>({ blueprint, adapter, validate, validateRow, components, }: HyleHooksConfig<TModels>): {
    useManifest: (query: Query) => HyleManifestState;
    useData: <TFrom extends string>(query: Query & {
        model: TFrom;
    }) => HyleDataState<TFrom extends keyof TModels ? RowOf<TModels[TFrom]> : Row>;
    useList: <TFrom extends string>(query: Query & {
        model: TFrom;
    }, options?: UseListOptions) => HyleListState<TFrom extends keyof TModels ? RowOf<TModels[TFrom]> : Row>;
    useFilters: <TFrom extends string>(query: Query & {
        model: TFrom;
    }, options?: UseFiltersOptions) => TypedFiltersState<TFrom extends keyof TModels ? RowOf<TModels[TFrom]> : Row, TFrom>;
    useForma: (tableName: string, id?: unknown, options?: UseFormaOptions) => [Query | null, Forma | null];
    useForm: <TFrom extends string>(query: Query & {
        model: TFrom;
    }, options?: UseFormOptions) => HyleFormState<TFrom extends keyof TModels ? RowOf<TModels[TFrom]> : Row>;
    useMutation: (model: string) => {
        create: HyleMutation;
        update: HyleMutation;
        delete: HyleMutation;
    } | null;
};
/**
 * A minimal mutation input shape used by the optimistic update helpers
 * and the `makeReactQueryMutation` / `makeRestAdapter` factories.
 */
export type MutationInput = {
    id?: unknown;
    data: Record<string, unknown>;
};
/**
 * Standard optimistic update for a save (PUT) mutation.
 * Replaces the matching row in `source[model].result` with merged field values.
 *
 * ```ts
 * optimisticUpdate: saveOptimistic("user")
 * ```
 */
export declare function saveOptimistic(model: string): (current: Source, { id, data }: MutationInput) => Source;
/**
 * Standard optimistic update for an add (POST) mutation.
 * Appends a new row built from `data` with a temporary id, increments total.
 *
 * ```ts
 * optimisticUpdate: addOptimistic("user")
 * ```
 */
export declare function addOptimistic(model: string): (current: Source, { data }: MutationInput) => Source;
/**
 * Standard optimistic update for a delete (DELETE) mutation.
 * Removes the matching row from `source[model].result`, decrements total.
 *
 * ```ts
 * optimisticUpdate: deleteOptimistic("user")
 * ```
 */
export declare function deleteOptimistic(model: string): (current: Source, { id }: MutationInput) => Source;
/**
 * Map a `FieldType` to the corresponding key in `HyleFieldComponents`.
 *
 * Used by component libraries (e.g. `@tty-pt/hyle-react-dom`) to select the correct
 * display or input component from a `HyleFieldComponents` map for a given
 * field type. Call this when building custom table cells, form inputs, or
 * filter controls that need to delegate rendering to the configured component
 * registry.
 *
 * @example
 * const key = componentKeyForField(field.fieldType); // e.g. "Reference"
 * const Component = components[key] ?? components.String;
 */
export declare function componentKeyForField(fieldType: FieldType): keyof HyleFieldComponents;
export type ReactQuerySourceFn = (manifest: Manifest | null, query: Query) => Promise<Source>;
export type ReactQueryMutationFn<TOutput = unknown> = (input: MutationInput) => Promise<TOutput>;
export type ReactQueryMutationOptions<TOutput = unknown> = {
    queryKey?: QueryKey;
    optimisticUpdate?: (current: Source, input: MutationInput) => Source;
    onSuccess?: (data: TOutput, input: MutationInput) => void;
};
export type RestAdapterOverrides = {
    [model: string]: Partial<{
        fetch: ReactQuerySourceFn;
        create: (input: MutationInput) => Promise<unknown>;
        update: (input: MutationInput) => Promise<unknown>;
        delete: (input: MutationInput) => Promise<unknown>;
    }>;
};
export declare function hyleQueryKey(query: Pick<Query, "model" | "select" | "where" | "page" | "sort">): QueryKey;
export declare function makeReactQuerySource(queryFn: ReactQuerySourceFn): HyleSourceAdapter;
export declare function makeReactQueryMutation<TOutput = unknown>(mutationFn: ReactQueryMutationFn<TOutput>, options?: ReactQueryMutationOptions<TOutput>): () => HyleMutation<MutationInput, TOutput>;
export declare function makeRestAdapter(baseUrl: string, options?: {
    overrides?: RestAdapterOverrides;
}): HyleAdapter;
//# sourceMappingURL=index.d.ts.map