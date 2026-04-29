import type { Blueprint, Column, Field, Forma, FormaContext, Manifest, PurifyError, Query, Result, Row, Source } from "./types";
export type HyleResolvedView = {
    outcome: Result;
    rows: Row[];
    isSingle: boolean;
    columns: Column[];
};
export type HyleWasmModule = {
    default: () => Promise<unknown> | unknown;
    manifest: (blueprintJson: string, queryJson: string) => string;
    resolve: (blueprintJson: string, manifestJson: string, sourceJson: string) => string;
    resolve_and_view?: (blueprintJson: string, manifestJson: string, sourceJson: string) => string;
    resolve_query?: (blueprintJson: string, queryJson: string, sourceJson: string) => string;
    rows_array?: (resultJson: string) => string;
    make_field?: (kind: string, label: string, entityJson: string) => string;
    columns?: (blueprintJson: string, manifestJson: string) => string;
    filter_rows?: (rowsJson: string, filtersJson: string) => string;
    display_value?: (blueprintJson: string, outcomeJson: string, modelName: string, fieldName: string, valueJson: string) => string;
    filter_layout?: (blueprintJson: string, manifestJson: string) => string;
    forma_to_query?: (formaJson: string, tableName: string, contextJson: string, idJson: string) => string;
    purify_row?: (blueprintJson: string, modelName: string, rowJson: string) => string;
    apply_view?: (rowsJson: string, manifestJson: string) => string;
    is_single?: (manifestJson: string, outcomeJson: string) => boolean;
};
export type HyleWasmLoader = () => Promise<HyleWasmModule>;
export type HyleClient = {
    load: () => Promise<void>;
    manifest: (blueprint: Blueprint, query: Query) => Manifest;
    resolve: (blueprint: Blueprint, manifest: Manifest, source: Source) => Result;
    resolveAndView: (blueprint: Blueprint, manifest: Manifest, source: Source) => HyleResolvedView;
    resolveQuery: (blueprint: Blueprint, query: Query, source: Source) => HyleResolvedData;
    rowsArray: (result: Result) => Row[];
    makeField: (kind: string, label: string, entity?: string) => Field;
    columns: (blueprint: Blueprint, manifest: Manifest) => Column[];
    filterRows: (rows: Row[], filters: Record<string, unknown> | undefined) => Row[];
    displayValue: (blueprint: Blueprint, result: Result, modelName: string, fieldName: string, value: unknown) => string;
    filterLayout: (blueprint: Blueprint, manifest: Manifest) => Column[][];
    formaToQuery: (forma: Forma, tableName: string, context: FormaContext, id?: unknown) => Query;
    purifyRow: (blueprint: Blueprint, modelName: string, row: Row) => PurifyError[] | null;
    applyView: (rows: Row[], manifest: Manifest) => Row[];
    isSingle: (manifest: Manifest, result: Result) => boolean;
};
export type HyleResolvedData = {
    manifest: Manifest;
    result: Result;
    rows: Row[];
};
export declare function createHyleClient(loadWasm: HyleWasmLoader): HyleClient;
//# sourceMappingURL=client.d.ts.map