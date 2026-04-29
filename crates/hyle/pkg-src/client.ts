import type {
  Blueprint,
  Column,
  Field,
  Forma,
  FormaContext,
  Manifest,
  PurifyError,
  Query,
  Result,
  Row,
  Source,
} from "./types";

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
  display_value?: (
    blueprintJson: string,
    outcomeJson: string,
    modelName: string,
    fieldName: string,
    valueJson: string,
  ) => string;
  filter_layout?: (blueprintJson: string, manifestJson: string) => string;
  forma_to_query?: (
    formaJson: string,
    tableName: string,
    contextJson: string,
    idJson: string,
  ) => string;
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
  displayValue: (
    blueprint: Blueprint,
    result: Result,
    modelName: string,
    fieldName: string,
    value: unknown,
  ) => string;
  filterLayout: (blueprint: Blueprint, manifest: Manifest) => Column[][];
  formaToQuery: (
    forma: Forma,
    tableName: string,
    context: FormaContext,
    id?: unknown,
  ) => Query;
  purifyRow: (
    blueprint: Blueprint,
    modelName: string,
    row: Row,
  ) => PurifyError[] | null;
  applyView: (rows: Row[], manifest: Manifest) => Row[];
  isSingle: (manifest: Manifest, result: Result) => boolean;
};

export type HyleResolvedData = {
  manifest: Manifest;
  result: Result;
  rows: Row[];
};

export function createHyleClient(loadWasm: HyleWasmLoader): HyleClient {
  let wasm: HyleWasmModule | null = null;
  let loadPromise: Promise<void> | null = null;

  const load = async () => {
    if (!loadPromise) {
      loadPromise = loadWasm().then(async (module) => {
        await module.default();
        wasm = module;
      });
    }

    return loadPromise;
  };

  const getWasm = () => {
    if (!wasm) {
      throw new Error("hyle WASM is not ready");
    }

    return wasm;
  };

  const client: HyleClient = {
    load,

    manifest(blueprint, query) {
      const module = getWasm();
      return JSON.parse(
        module.manifest(JSON.stringify(blueprint), JSON.stringify(query)),
      ) as Manifest;
    },

    resolve(blueprint, manifest, source) {
      const module = getWasm();
      return JSON.parse(
        module.resolve(
          JSON.stringify(blueprint),
          JSON.stringify(manifest),
          JSON.stringify(source),
        ),
      ) as Result;
    },

    resolveAndView(blueprint, manifest, source) {
      const module = getWasm();
      const fn_ = requireWasmExport(module.resolve_and_view, "resolve_and_view");
      return JSON.parse(
        fn_(JSON.stringify(blueprint), JSON.stringify(manifest), JSON.stringify(source)),
      ) as HyleResolvedView;
    },

    resolveQuery(blueprint, query, source) {
      const module = getWasm();
      const fn_ = requireWasmExport(module.resolve_query, "resolve_query");
      return JSON.parse(
        fn_(JSON.stringify(blueprint), JSON.stringify(query), JSON.stringify(source)),
      ) as HyleResolvedData;
    },

    rowsArray(result) {
      const module = getWasm();
      const fn_ = requireWasmExport(module.rows_array, "rows_array");
      return JSON.parse(fn_(JSON.stringify(result))) as Row[];
    },

    makeField(kind, label, entity) {
      const module = getWasm();
      const fn_ = requireWasmExport(module.make_field, "make_field");
      return JSON.parse(fn_(kind, label, JSON.stringify(entity ?? null))) as Field;
    },

    columns(blueprint, manifest) {
      const module = getWasm();
      const columns = requireWasmExport(module.columns, "columns");
      return JSON.parse(
        columns(JSON.stringify(blueprint), JSON.stringify(manifest)),
      ) as Column[];
    },

    filterRows(rows, filters) {
      if (!filters) return rows;

      const module = getWasm();
      const filterRows = requireWasmExport(module.filter_rows, "filter_rows");
      return JSON.parse(
        filterRows(JSON.stringify(rows), JSON.stringify(filters)),
      ) as Row[];
    },

    displayValue(blueprint, result, modelName, fieldName, value) {
      const module = getWasm();
      const displayValue = requireWasmExport(module.display_value, "display_value");
      return displayValue(
        JSON.stringify(blueprint),
        JSON.stringify(result),
        modelName,
        fieldName,
        JSON.stringify(value),
      );
    },

    filterLayout(blueprint, manifest) {
      const module = getWasm();
      const fn_ = requireWasmExport(module.filter_layout, "filter_layout");
      return JSON.parse(
        fn_(JSON.stringify(blueprint), JSON.stringify(manifest)),
      ) as Column[][];
    },

    formaToQuery(forma, tableName, context, id) {
      const module = getWasm();
      const fn_ = requireWasmExport(module.forma_to_query, "forma_to_query");
      return JSON.parse(
        fn_(
          JSON.stringify(forma),
          tableName,
          JSON.stringify(context),
          JSON.stringify(id ?? null),
        ),
      ) as Query;
    },

    purifyRow(blueprint, modelName, row) {
      const module = getWasm();
      const fn_ = requireWasmExport(module.purify_row, "purify_row");
      return JSON.parse(
        fn_(JSON.stringify(blueprint), modelName, JSON.stringify(row)),
      ) as ReturnType<HyleClient["purifyRow"]>;
    },

    applyView(rows, manifest) {
      const module = getWasm();
      const fn_ = requireWasmExport(module.apply_view, "apply_view");
      return JSON.parse(fn_(JSON.stringify(rows), JSON.stringify(manifest))) as Row[];
    },

    isSingle(manifest, result) {
      const module = getWasm();
      const fn_ = requireWasmExport(module.is_single, "is_single");
      return fn_(JSON.stringify(manifest), JSON.stringify(result));
    },
  };

  return client;
}

function requireWasmExport<T extends (...args: never[]) => unknown>(
  fn: T | undefined,
  name: string,
): T {
  if (!fn) {
    throw new Error(
      `hyle WASM export '${name}' is missing; rebuild the hyle-wasm package`,
    );
  }

  return fn;
}
