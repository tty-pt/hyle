import {
  createContext,
  createElement,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ComponentType,
  type ReactNode,
} from "react";

import type {
  Blueprint,
  Column,
  Field,
  FieldType,
  Forma,
  FormaContext,
  HyleClient,
  Manifest,
  PurifyError,
  Query,
  Result,
  HyleResolvedView,
  Row,
  FormaField,
  FormaFieldType,
  Source,
  TypedBlueprint,
  TypedModel,
  TypedField,
  RowOf,
} from "@tty-pt/hyle";

import {
  useQuery,
  useMutation as useTanstackMutation,
  useQueryClient,
  type QueryKey,
} from "@tanstack/react-query";

export type {
  Blueprint,
  Column,
  Field,
  FieldOptions,
  FieldType,
  Forma,
  FormaContext,
  FormaField,
  FormaFieldType,
  Manifest,
  Model,
  ModelResult,
  PurifyError,
  Query,
  Result,
  HyleResolvedView,
  Row,
  RowOf,
  ShapeField,
  Sort,
  Source,
  TypedBlueprint,
  TypedField,
  TypedModel,
} from "@tty-pt/hyle";

export {
  createHyleClient,
  field,
} from "@tty-pt/hyle";

// ─── Component map types ────────────────────────────────────────────────────

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
  // Value display components
  String?: ComponentType<ValueProps>;
  Number?: ComponentType<ValueProps>;
  Boolean?: ComponentType<ValueProps>;
  File?: ComponentType<ValueProps>;
  Reference?: ComponentType<ValueProps>;
  Array?: ComponentType<ValueProps>;
  Shape?: ComponentType<ValueProps>;
  // Filter input components
  FilterString?: ComponentType<FilterProps<string>>;
  FilterNumber?: ComponentType<FilterProps<string>>;
  FilterBoolean?: ComponentType<FilterProps<boolean | undefined>>;
  FilterFile?: ComponentType<FilterProps<string>>;
  FilterReference?: ComponentType<FilterProps<string>>;
  FilterArray?: ComponentType<FilterProps<unknown>>;
  FilterShape?: ComponentType<FilterProps<unknown>>;
};

// ─── Hyle context / provider ────────────────────────────────────────────────

export type HyleStatus = "loading" | "ready" | "error";

export type HyleResolvedState =
  | { status: "loading"; error: null; manifest: null; result: null; rows: Row[] }
  | { status: "error"; error: Error; manifest: null; result: null; rows: Row[] }
  | { status: "ready"; error: null; manifest: Manifest; result: Result; rows: Row[] };

export type HyleSourceAdapterResult =
  | Source
  | {
      source?: Source | null;
      loading?: boolean;
      error?: unknown;
    }
  | null
  | undefined;

export type HyleSourceAdapter = (manifest: Manifest | null, query: Query) => HyleSourceAdapterResult;

export type HyleValidationAdapter = (input: {
  blueprint: Blueprint;
  query: Query;
  rows: Row[];
  result: Result;
}) => unknown;

export type HyleRowValidationAdapter = (
  blueprint: Blueprint,
  modelName: string,
  row: Row,
) => PurifyError[] | null;

/** Factory that produces create/update/delete mutation hooks for a given model name. */
export type HyleMutationFactory = (model: string) => {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  useCreate: () => HyleMutation<any, any>;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  useUpdate: () => HyleMutation<any, any>;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
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

export type HyleHooksConfig<
  TModels extends Record<string, TypedModel<Record<string, TypedField<unknown>>>> = Record<string, TypedModel<Record<string, TypedField<unknown>>>>
> = {
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

const HyleContext = createContext<HyleContextValue | null>(null);

export type UseHyleReturn = {
  status: HyleStatus;
  error: Error | null;
  blueprint: Blueprint | null;
  manifest: (blueprint: Blueprint, query: Query) => Manifest;
  resolve: (blueprint: Blueprint, manifest: Manifest, source: Source) => Result;
  resolveAndView: (blueprint: Blueprint, manifest: Manifest, source: Source) => HyleResolvedView;
  resolveQuery: (blueprint: Blueprint, query: Query, source: Source) => { manifest: Manifest; result: Result; rows: Row[] };
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
  modelFields: (blueprint: Blueprint, modelName: string) => { key: string; label: string; field: Field }[];
};

export function HyleProvider({
  client,
  blueprint = null,
  children,
}: {
  client: HyleClient;
  blueprint?: Blueprint | null;
  children: ReactNode;
}) {
  const [state, setState] = useState<{
    status: HyleStatus;
    error: Error | null;
  }>({ status: "loading", error: null });

  useEffect(() => {
    let cancelled = false;

    setState({ status: "loading", error: null });
    client
      .load()
      .then(() => {
        if (!cancelled) {
          setState({ status: "ready", error: null });
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setState({
            status: "error",
            error: toError(error),
          });
        }
      });

    return () => {
      cancelled = true;
    };
  }, [client]);

  const value = useMemo(
    () => ({ client, blueprint, status: state.status, error: state.error }),
    [client, blueprint, state.error, state.status],
  );

  return createElement(HyleContext.Provider, { value }, children);
}

export function useHyle(): UseHyleReturn {
  const context = useContext(HyleContext);

  if (!context) {
    throw new Error("useHyle must be used inside HyleProvider");
  }

  const manifest = useCallback(
    (blueprint: Blueprint, query: Query) => context.client.manifest(blueprint, query),
    [context.client],
  );

  const resolve = useCallback(
    (blueprint: Blueprint, manifest: Manifest, source: Source) =>
      context.client.resolve(blueprint, manifest, source),
    [context.client],
  );

  const resolveAndView = useCallback(
    (blueprint: Blueprint, manifest: Manifest, source: Source) =>
      context.client.resolveAndView(blueprint, manifest, source),
    [context.client],
  );

  const resolveQuery = useCallback(
    (blueprint: Blueprint, query: Query, source: Source) =>
      context.client.resolveQuery(blueprint, query, source),
    [context.client],
  );

  const columns = useCallback(
    (blueprint: Blueprint, manifest: Manifest) =>
      context.client.columns(blueprint, manifest),
    [context.client],
  );

  const filterRows = useCallback(
    (rows: Row[], filters: Record<string, unknown> | undefined) =>
      context.client.filterRows(rows, filters),
    [context.client],
  );

  const applyView = useCallback(
    (rows: Row[], manifest: Manifest) =>
      context.client.applyView(rows, manifest),
    [context.client],
  );

  const displayValue = useCallback(
    (
      blueprint: Blueprint,
      result: Result,
      modelName: string,
      fieldName: string,
      value: unknown,
    ) => context.client.displayValue(blueprint, result, modelName, fieldName, value),
    [context.client],
  );

  const filterLayout = useCallback(
    (blueprint: Blueprint, manifest: Manifest) =>
      context.client.filterLayout(blueprint, manifest),
    [context.client],
  );

  const formaToQuery = useCallback(
    (forma: Forma, tableName: string, context_: FormaContext, id?: unknown) =>
      context.client.formaToQuery(forma, tableName, context_, id),
    [context.client],
  );

  const purifyRow = useCallback(
    (blueprint: Blueprint, modelName: string, row: Row) =>
      context.client.purifyRow(blueprint, modelName, row),
    [context.client],
  );

  const isSingle = useCallback(
    (manifest: Manifest, result: Result) =>
      context.client.isSingle(manifest, result),
    [context.client],
  );

  const rowsArray = useCallback(
    (result: Result) => context.client.rowsArray(result),
    [context.client],
  );

  const makeField = useCallback(
    (kind: string, label: string, entity?: string) =>
      context.client.makeField(kind, label, entity),
    [context.client],
  );

  const modelFields = useCallback(
    (blueprint: Blueprint, modelName: string): { key: string; label: string; field: Field }[] => {
      const model = blueprint.models[modelName];
      if (!model) return [];
      return Object.entries(model.fields).map(([key, field]) => ({
        key,
        label: field.label,
        field,
      }));
    },
    [],
  );

  return useMemo(() => ({
    status: context.status,
    error: context.error,
    blueprint: context.blueprint,
    manifest,
    resolve,
    resolveAndView,
    resolveQuery,
    rowsArray,
    makeField,
    columns,
    filterRows,
    applyView,
    displayValue,
    filterLayout,
    formaToQuery,
    purifyRow,
    isSingle,
    modelFields,
  }), [
    applyView,
    columns,
    context.blueprint,
    context.error,
    context.status,
    displayValue,
    filterLayout,
    filterRows,
    formaToQuery,
    isSingle,
    makeField,
    manifest,
    modelFields,
    purifyRow,
    resolve,
    resolveAndView,
    resolveQuery,
    rowsArray,
  ]);
}

export function useHyleResolved({
  blueprint,
  query,
  source,
}: {
  blueprint: Blueprint;
  query: Query;
  source: Source | null | undefined;
}): HyleResolvedState {
  const hyle = useHyle();

  return useMemo(() => {
    if (hyle.status === "loading" || !source) {
      return { status: "loading", error: null, manifest: null, result: null, rows: [] };
    }

    if (hyle.status === "error") {
      return {
        status: "error",
        error: hyle.error ?? new Error("hyle WASM failed to load"),
        manifest: null,
        result: null,
        rows: [],
      };
    }

    try {
      return {
        status: "ready",
        error: null,
        ...hyle.resolveQuery(blueprint, query, source),
      };
    } catch (error) {
      return {
        status: "error",
        error: toError(error),
        manifest: null,
        result: null,
        rows: [],
      };
    }
  }, [blueprint, hyle.error, hyle.resolveQuery, hyle.status, query, source]);
}

// ─── State types ─────────────────────────────────────────────────────────────

export type HyleManifestState =
  | { status: "loading"; error: null; manifest: null }
  | { status: "error"; error: Error; manifest: null }
  | { status: "ready"; error: null; manifest: Manifest };

export type HyleDataField = {
  key: string;
  label: string;
  field: Field;
  raw: unknown;
  render: () => string;
};

export type HyleDataState<TRow extends Row = Row> =
  | {
      status: "loading";
      error: null;
      manifest: Manifest | null;
      result: null;
      rows: TRow[];
      row: TRow | null;
      fields: HyleDataField[];
      validation: null;
    }
  | {
      status: "error";
      error: Error;
      manifest: Manifest | null;
      result: null;
      rows: TRow[];
      row: TRow | null;
      fields: HyleDataField[];
      validation: null;
    }
  | {
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

export type ReadyListState<TRow extends Row = Row> =
  Extract<HyleListState<TRow>, { status: "ready" }>;

export type TypedFiltersState<TRow extends Row, TFrom extends string> =
  HyleFiltersState<TRow> & { query: Query & { model: TFrom } };

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
  onSubmit: (e?: { preventDefault?: () => void }) => void;
  mutation: HyleMutation | null;
};

// ─── Forma types ──────────────────────────────────────────────────────────────

export type UseFormaOptions = {
  /** Which field subset context to use when deriving select */
  context?: FormaContext;
  /** How many items to show per page */
  selectAmount?: number;
  /** Filter amount for filter layout derivation */
  filterAmount?: number;
};

// ─── makeHyleHooks ────────────────────────────────────────────────────────────

/** Model name for the forma configuration entity. */
const FORMA_MODEL = "forma";

export function makeHyleHooks<
  TModels extends Record<string, TypedModel<Record<string, TypedField<unknown>>>> = Record<string, TypedModel<Record<string, TypedField<unknown>>>>
>({
  blueprint,
  adapter,
  validate,
  validateRow,
  components = {},
}: HyleHooksConfig<TModels>) {
  const { useSource, makeMutation } = adapter;

  // Helper: resolve the row type for a given `from` string against TModels
  type RowForFrom<TFrom extends string> =
    TFrom extends keyof TModels ? RowOf<TModels[TFrom]> : Row;

  // ── useManifest ───────────────────────────────────────────────────────────

  function useManifest(query: Query): HyleManifestState {
    const hyle = useHyle();
    const prevRef = useRef<{ json: string; state: HyleManifestState } | null>(null);

    const state = useMemo(() => {
      if (hyle.status === "loading") {
        return { status: "loading", error: null, manifest: null } as HyleManifestState;
      }

      if (hyle.status === "error") {
        return {
          status: "error",
          error: hyle.error ?? new Error("hyle WASM failed to load"),
          manifest: null,
        } as HyleManifestState;
      }

      try {
        return {
          status: "ready",
          error: null,
          manifest: hyle.manifest(blueprint, query),
        } as HyleManifestState;
      } catch (error) {
        return {
          status: "error",
          error: toError(error),
          manifest: null,
        } as HyleManifestState;
      }
    }, [hyle.error, hyle.manifest, hyle.status, query]);

    // Stabilise the returned state by JSON identity so downstream memos
    // (especially the Filter component map) don't rebuild on every render
    // just because WASM returned a new object with the same content.
    const json = JSON.stringify(state);
    if (prevRef.current?.json === json) return prevRef.current.state;
    prevRef.current = { json, state };
    return state;
  }

  // ── useData ───────────────────────────────────────────────────────────────

  function useData<TFrom extends string>(
    query: Query & { model: TFrom }
  ): HyleDataState<RowForFrom<TFrom>>;
  function useData(query: Query): HyleDataState {
    const hyle = useHyle();
    const planned = useManifest(query);
    const sourceResult = useSource(planned.manifest, query);

    return useMemo(() => {
      const emptyFields: HyleDataField[] = [];

      if (planned.status === "error") {
        return {
          status: "error",
          error: planned.error,
          manifest: planned.manifest,
          result: null,
          rows: [],
          row: null,
          fields: emptyFields,
          validation: null,
        };
      }

      if (planned.status === "loading") {
        return {
          status: "loading",
          error: null,
          manifest: planned.manifest,
          result: null,
          rows: [],
          row: null,
          fields: emptyFields,
          validation: null,
        };
      }

      const sourceState = normalizeSourceResult(sourceResult);

      if (sourceState.error) {
        return {
          status: "error",
          error: sourceState.error,
          manifest: planned.manifest,
          result: null,
          rows: [],
          row: null,
          fields: emptyFields,
          validation: null,
        };
      }

      if (sourceState.loading || !sourceState.source || hyle.status !== "ready") {
        return {
          status: "loading",
          error: null,
          manifest: planned.manifest,
          result: null,
          rows: [],
          row: null,
          fields: emptyFields,
          validation: null,
        };
      }

      try {
        const { outcome: result, rows, isSingle, columns } =
          hyle.resolveAndView(blueprint, planned.manifest, sourceState.source);

        const row = isSingle ? (rows[0] ?? null) : null;

        // Build per-field accessor array for single-record usage
        const fields: HyleDataField[] = columns.map(
          (col) => ({
            key: col.key,
            label: col.label,
            field: col.field,
            raw: row?.[col.key] ?? null,
            render: () =>
              hyle.displayValue(blueprint, result, planned.manifest.base, col.key, row?.[col.key]),
          }),
        );

        return {
          status: "ready",
          error: null,
          manifest: planned.manifest,
          result,
          rows,
          row,
          fields,
          validation: validate?.({ blueprint, query, rows, result }) ?? null,
        };
      } catch (error) {
        return {
          status: "error",
          error: toError(error),
          manifest: planned.manifest,
          result: null,
          rows: [],
          row: null,
          fields: emptyFields,
          validation: null,
        };
      }
    }, [hyle.resolveAndView, hyle.status, planned, query, sourceResult]);
  }

  // ── useList ───────────────────────────────────────────────────────────────

  function useList<TFrom extends string>(
    query: Query & { model: TFrom },
    options?: UseListOptions,
  ): HyleListState<RowForFrom<TFrom>>;
  function useList(query: Query, options: UseListOptions = {}): HyleListState {
    const { perPageOptions = [5, 10, 20] } = options;
    const [page, setPage] = useState(query.page ?? 1);
    const [perPage, setPerPage] = useState(query.perPage ?? perPageOptions[0] ?? 20);
    const [sortField, setSortField] = useState<string | undefined>(query.sort?.field);
    const [sortAscending, setSortAscending] = useState(query.sort?.ascending ?? true);

    const effectiveQuery = useMemo<Query>(() => ({
      ...query,
      page,
      perPage,
      sort: sortField ? { field: sortField, ascending: sortAscending } : query.sort,
    }), [page, perPage, query, sortAscending, sortField]);

    const data = useData(effectiveQuery);

    return {
      ...data,
      query: effectiveQuery,
      page,
      perPage,
      perPageOptions,
      setPage,
      setPerPage,
      sortField,
      sortAscending,
      setSortField,
      setSortAscending,
    };
  }

  // ── useFilters ────────────────────────────────────────────────────────────

  function useFilters<TFrom extends string>(
    query: Query & { model: TFrom },
    options?: UseFiltersOptions,
  ): TypedFiltersState<RowForFrom<TFrom>, TFrom>;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  function useFilters(query: Query, options: UseFiltersOptions = {}): any {
    const hyle = useHyle();
    const { change } = options;

    const [committed, setCommitted] = useState<Partial<Row>>({});
    const [formData, setFormData] = useState<Partial<Row>>({});
    const [filterResetKey, setFilterResetKey] = useState(0);
    const [purifyErrors, setPurifyErrors] = useState<PurifyError[] | null>(null);

    const effectiveQuery = useMemo<Query>(() => ({
      ...query,
      where: { ...(query.where ?? {}), ...(committed as Record<string, unknown>) },
    }), [committed, query]);

    // Fetch data internally: with id → seed form for edit mode; without id → get lookups for reference filters.
    const hasId = Boolean(query.where && "id" in query.where);
    const seedData = useData(hasId ? effectiveQuery : { model: effectiveQuery.model });

    useEffect(() => {
      if (seedData.status === "ready" && seedData.row !== null) {
        const seeded: Partial<Row> = {};
        for (const [k, v] of Object.entries(seedData.row)) {
          seeded[k] = v !== null && v !== undefined ? v : "";
        }
        setFormData(seeded);
      }
    // Re-seed only when the record identity changes (new id opened), not on
    // every render — seedData.row is a new object reference on each useMemo
    // recompute even when the data hasn't changed.
    // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [seedData.row?.id]);

    const setField = useCallback((name: string, value: unknown) => {
      setFormData((prev) => ({ ...prev, [name]: value }));
    }, []) as HyleFiltersState["setField"];

    const filterApply = useCallback(() => {
      setCommitted((prev) => ({ ...prev, ...(formData as Record<string, unknown>) }));
    }, [formData]);

    const filterClear = useCallback(() => {
      setFormData({});
      setCommitted({});
      setFilterResetKey((k) => k + 1);
    }, []);

    // Refs used inside validate/Filter closures to avoid stale captures
    const changeRef = useRef(change);
    changeRef.current = change;
    const formDataRef = useRef(formData);
    formDataRef.current = formData;

    const validate = useCallback((): PurifyError[] | null => {
      const activeFormData: Row = Object.fromEntries(
        Object.entries(formDataRef.current as Record<string, unknown>).filter(([k]) => {
          if (!changeRef.current) return true;
          if (!(k in changeRef.current)) return true;
          return changeRef.current[k](blueprint.models[effectiveQuery.model]?.fields[k] ?? { label: k, type: { kind: "primitive", primitive: "string" } }) !== null;
        })
      );
      const errors = validateRow
        ? validateRow(blueprint, effectiveQuery.model, activeFormData)
        : hyle.purifyRow(blueprint, effectiveQuery.model, activeFormData);
      setPurifyErrors(errors);
      return errors;
    // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [effectiveQuery.model, hyle.purifyRow, validateRow]);

    // Derive Filter component map from plan — no fetch needed, just field metadata.
    const planned = useManifest(effectiveQuery);
    const manifest = planned.status === "ready" ? planned.manifest : null;

    const componentsRef = useRef(components);
    componentsRef.current = components;
    const setFieldRef = useRef(setField);
    setFieldRef.current = setField;
    const resultRef = useRef<Result | null>(null);
    resultRef.current = seedData.status === "ready" ? seedData.result : null;

    const Filter = useMemo(() => {
      const cols = manifest ? hyle.columns(blueprint, manifest) : [];
      const map: Record<string, ComponentType> = {};
      for (const col of cols) {
        const key = col.key;
        // Apply per-field change transform
        const changed = changeRef.current?.[key]?.(col.field);
        if (changed === null) continue; // excluded
        const effectiveField = changed ?? col.field;
        const effectiveLabel = effectiveField.label ?? col.label;
        const fieldComponent = (changed as FieldChangeResult | undefined)?.component;
        map[key] = function HyleFilterField() {
          const value = formDataRef.current[col.key] ?? "";
          return createElement(FilterInputComponent, {
            fieldName: col.key,
            field: effectiveField,
            label: effectiveLabel,
            value,
            result: resultRef.current,
            components: componentsRef.current,
            fieldComponent,
            onChange: (v: unknown) => setFieldRef.current(col.key, v),
          });
        };
        Object.defineProperty(map[key], "name", { value: `Filter.${col.key}` });
      }
      return map;
    // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [manifest, filterResetKey]);

    return {
      query: effectiveQuery,
      formData,
      setField,
      filterApply,
      filterClear,
      filterResetKey,
      Filter,
      validate,
      purifyErrors,
    };
  }

  // ── useForma ──────────────────────────────────────────────────────────────

  /**
   * Fetch a forma definition from the "forma" model in the blueprint,
   * then derive a Query for the target table.
   *
   * Returns [derivedQuery, forma] — derivedQuery is null while loading.
   */
  function useForma(
    tableName: string,
    id?: unknown,
    options: UseFormaOptions = {},
  ): [Query | null, Forma | null] {
    const hyle = useHyle();
    const { context = "column" } = options;

    // Query the forma entity
    const formaQuery: Query = useMemo(() => ({
      model: FORMA_MODEL,
      where: { id: tableName },
      method: "one",
      select: ["fields", "detail", "form", "column", "filters"],
    }), [tableName]);

    const formaData = useData(formaQuery);

    return useMemo(() => {
      if (formaData.status !== "ready" || !formaData.row) {
        return [null, null];
      }

      // The row from the "forma" entity IS the Forma object
      const forma = formaData.row as unknown as Forma;

      if (!forma.fields?.length) return [null, forma];

      try {
        // Delegate query derivation to Rust via WASM
        const derivedQuery = hyle.formaToQuery(forma, tableName, context, id);
        return [derivedQuery, forma];
      } catch {
        return [null, forma];
      }
    }, [context, hyle, id, formaData.row, formaData.status, tableName]);
  }

  // ── useForm ───────────────────────────────────────────────────────────────

  /**
   * Auto-wired form hook. Derives model name and edit/create mode from the query,
   * then wires the appropriate mutation (create vs update) from the `makeMutation`
   * factory configured in `makeHyleHooks`.
   *
   * Returns everything `useFilters` returns plus:
   * - `isEdit` — whether the query targets an existing record (has `where.id`).
   * - `isValid` — `true` when `purifyErrors` is `null`.
    * - `onSubmit` — validates then fires the appropriate mutation.
    * - `mutation` — the active mutation handle (null if no `makeMutation` configured).
    */
   function useForm<TFrom extends string>(
    query: Query & { model: TFrom },
    options?: UseFormOptions,
  ): HyleFormState<RowForFrom<TFrom>>;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  function useForm(query: Query, options: UseFormOptions = {}): any {
    const isEdit = Boolean(query.where && "id" in query.where);
    const model = query.model;

    // Always call both hooks unconditionally (Rules of Hooks).
    // Only one will be used based on isEdit, but hooks must not be conditional.
    let mutation: HyleMutation | null = null;
    if (makeMutation) {
      const factory = makeMutation(model);
      const createMut = factory.useCreate();
      const updateMut = factory.useUpdate();
      mutation = isEdit ? updateMut : createMut;
    }

    const filters = useFilters(query, { change: options.change });

    const mutationRef = useRef(mutation);
    mutationRef.current = mutation;
    const filtersRef = useRef(filters);
    filtersRef.current = filters;

    const onSubmit = useCallback((e?: { preventDefault?: () => void }) => {
      e?.preventDefault?.();
      const errors = filtersRef.current.validate();
      if (errors !== null) return;
      const id = filtersRef.current.formData.id;
      mutationRef.current?.mutate({ id, data: filtersRef.current.formData } as Record<string, unknown>);
    // eslint-disable-next-line react-hooks/exhaustive-deps
    }, []);

    return {
      ...filters,
      isEdit,
      isValid: filters.purifyErrors === null,
      onSubmit,
      mutation,
    };
  }

  return {
    useManifest,
    useData,
    useList,
    useFilters,
    useForma,
    useForm,
    useMutation,
  };

  /**
   * Returns create/update/delete mutation handles for the given model.
   * Use this for standalone mutations (e.g. delete) outside of a form.
   */
  function useMutation(model: string): {
    create: HyleMutation;
    update: HyleMutation;
    delete: HyleMutation;
  } | null {
    if (!makeMutation) {
      // Rules of Hooks: still call hooks unconditionally — but with a stable dummy.
      // We can't conditionally call makeMutation(model), so we handle null adapter.
      return null;
    }
    const factory = makeMutation(model);
    return {
      create: factory.useCreate(),
      update: factory.useUpdate(),
      delete: factory.useDelete(),
    };
  }
}

// ─── Optimistic update helpers ────────────────────────────────────────────────

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
export function saveOptimistic(model: string) {
  return (current: Source, { id, data }: MutationInput): Source => ({
    ...current,
    [model]: {
      ...current[model],
      result: (current[model].result as Row[]).map((r) =>
        r.id === id ? { ...r, ...data } : r,
      ),
    },
  });
}

/**
 * Standard optimistic update for an add (POST) mutation.
 * Appends a new row built from `data` with a temporary id, increments total.
 *
 * ```ts
 * optimisticUpdate: addOptimistic("user")
 * ```
 */
export function addOptimistic(model: string) {
  return (current: Source, { data }: MutationInput): Source => {
    const rows = current[model].result as Row[];
    const tempId = Math.max(0, ...rows.map((r) => Number(r.id) || 0)) + 1;
    return {
      ...current,
      [model]: {
        total: current[model].total + 1,
        result: [...rows, { id: tempId, ...data }],
      },
    };
  };
}

/**
 * Standard optimistic update for a delete (DELETE) mutation.
 * Removes the matching row from `source[model].result`, decrements total.
 *
 * ```ts
 * optimisticUpdate: deleteOptimistic("user")
 * ```
 */
export function deleteOptimistic(model: string) {
  return (current: Source, { id }: MutationInput): Source => ({
    ...current,
    [model]: {
      total: current[model].total - 1,
      result: (current[model].result as Row[]).filter((r) => r.id !== id),
    },
  });
}

// ─── Component author utilities ───────────────────────────────────────────────

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
export function componentKeyForField(fieldType: FieldType): keyof HyleFieldComponents {
  if (fieldType.kind === "primitive") {
    const map: Record<string, keyof HyleFieldComponents> = {
      string: "String",
      number: "Number",
      boolean: "Boolean",
      file: "File",
    };
    return map[fieldType.primitive] ?? "String";
  }
  if (fieldType.kind === "reference") return "Reference";
  if (fieldType.kind === "array") return "Array";
  return "Shape";
}

/** Resolve the filter component key from a FieldType */
function filterComponentKeyForField(fieldType: FieldType): keyof HyleFieldComponents {
  if (fieldType.kind === "primitive") {
    const map: Record<string, keyof HyleFieldComponents> = {
      string: "FilterString",
      number: "FilterNumber",
      boolean: "FilterBoolean",
      file: "FilterFile",
    };
    return map[fieldType.primitive] ?? "FilterString";
  }
  if (fieldType.kind === "reference") return "FilterReference";
  if (fieldType.kind === "array") return "FilterArray";
  return "FilterShape";
}

type FilterInputProps = {
  fieldName: string;
  field: Field;
  label: string;
  value: unknown;
  result: Result | null;
  components: HyleFieldComponents;
  /** Per-field component override (from `change` map); takes priority over `components`. */
  fieldComponent?: ComponentType<FilterProps<unknown>>;
  onChange: (value: unknown) => void;
};

/** Internal component that renders a filter input using the component map or a default <input> */
function FilterInputComponent({
  fieldName,
  field,
  label,
  value,
  result,
  components,
  fieldComponent,
  onChange,
}: FilterInputProps) {
  const filterKey = filterComponentKeyForField(field.type);
  const CustomFilter =
    (fieldComponent as ComponentType<FilterProps<unknown>> | undefined) ??
    (components[filterKey] as ComponentType<FilterProps<unknown>> | undefined);

  if (CustomFilter) {
    return createElement(CustomFilter, { fieldName, field, label, value, result, onChange });
  }

  // Default: plain text input
  return createElement("input", {
    type: "text",
    "aria-label": label,
    placeholder: label,
    value: value ?? "",
    onChange: (e: { target: { value: string } }) => onChange(e.target.value),
  });
}

function normalizeSourceResult(result: HyleSourceAdapterResult): {
  source: Source | null;
  loading: boolean;
  error: Error | null;
} {
  if (!result) {
    return { source: null, loading: true, error: null };
  }

  if (isSource(result)) {
    return { source: result, loading: false, error: null };
  }

  return {
    source: result.source ?? null,
    loading: result.loading ?? !result.source,
    error: result.error ? toError(result.error) : null,
  };
}

function isSource(result: HyleSourceAdapterResult): result is Source {
  return Boolean(result && typeof result === "object" && !("source" in result));
}

function toError(error: unknown): Error {
  return error instanceof Error ? error : new Error(String(error));
}

// ─── TanStack Query adapters ────────────────────────────────────────────────────

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

export function hyleQueryKey(query: Pick<Query, "model" | "select" | "where" | "page" | "sort">): QueryKey {
  return [
    "hyle",
    query.model,
    query.select ?? [],
    query.where ?? {},
    query.page ?? null,
    query.sort ?? null,
  ];
}

export function makeReactQuerySource(queryFn: ReactQuerySourceFn): HyleSourceAdapter {
  return function useReactQuerySource(
    manifest: Manifest | null,
    query: Query,
  ) {
    // eslint-disable-next-line react-hooks/rules-of-hooks
    const { data, isPending, isError, error } = useQuery({
      queryKey: hyleQueryKey(query),
      queryFn: () => queryFn(manifest, query),
    });

    if (isPending) return { loading: true };
    if (isError)   return { error: error instanceof Error ? error : new Error(String(error)) };
    return data ?? { loading: true };
  };
}

export function makeReactQueryMutation<TOutput = unknown>(
  mutationFn: ReactQueryMutationFn<TOutput>,
  options: ReactQueryMutationOptions<TOutput> = {},
): () => HyleMutation<MutationInput, TOutput> {
  const { queryKey = ["hyle"], optimisticUpdate, onSuccess } = options;

  return function useReactQueryMutation(): HyleMutation<MutationInput, TOutput> {
    // eslint-disable-next-line react-hooks/rules-of-hooks
    const queryClient = useQueryClient();

    // eslint-disable-next-line react-hooks/rules-of-hooks
    const mutation = useTanstackMutation<TOutput, Error, MutationInput, { snapshot: Source | undefined }>({
      mutationFn,

      onMutate: async (input) => {
        if (!optimisticUpdate) return { snapshot: undefined };

        await queryClient.cancelQueries({ queryKey });

        const snapshot = queryClient.getQueryData<Source>(queryKey);

        if (snapshot !== undefined) {
          queryClient.setQueryData<Source>(queryKey, (current) =>
            current ? optimisticUpdate(current, input) : current
          );
        }

        return { snapshot };
      },

      onError: (_err, _input, context) => {
        if (context?.snapshot !== undefined) {
          queryClient.setQueryData<Source>(queryKey, context.snapshot);
        }
      },

      onSuccess: (data, input) => {
        onSuccess?.(data, input);
      },

      onSettled: () => {
        queryClient.invalidateQueries({ queryKey });
      },
    });

    return {
      mutate: mutation.mutate,
      reset: mutation.reset,
      isPending: mutation.isPending,
      isSuccess: mutation.isSuccess,
      error: mutation.error,
      data: mutation.data,
    };
  };
}

export function makeRestAdapter(
  baseUrl: string,
  options: { overrides?: RestAdapterOverrides } = {},
): HyleAdapter {
  const { overrides = {} } = options;

  const defaultFetch: ReactQuerySourceFn = async (_manifest, _query) => {
    const res = await fetch(`${baseUrl}/source`);
    if (!res.ok) throw new Error(`Source fetch failed: ${res.status}`);
    return res.json();
  };

  const useSource = makeReactQuerySource((manifest, query) => {
    const modelOverride = overrides[query.model]?.fetch;
    return modelOverride ? modelOverride(manifest, query) : defaultFetch(manifest, query);
  });

  const makeMutation = (model: string) => {
    const mo = overrides[model] ?? {};
    const base = `${baseUrl}/${model}`;
    const queryKey: QueryKey = ["hyle", model];

    const useCreate = makeReactQueryMutation<unknown>(
      mo.create ?? (async ({ data }) => {
        const res = await fetch(base, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(data),
        });
        if (!res.ok) throw new Error(`Create failed: ${res.status}`);
        return res.json();
      }),
      { queryKey, optimisticUpdate: addOptimistic(model) },
    );

    const useUpdate = makeReactQueryMutation<unknown>(
      mo.update ?? (async ({ id, data }) => {
        const res = await fetch(`${base}/${id}`, {
          method: "PUT",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(data),
        });
        if (!res.ok) throw new Error(`Save failed: ${res.status}`);
        return res.json();
      }),
      { queryKey, optimisticUpdate: saveOptimistic(model) },
    );

    const useDelete = makeReactQueryMutation<unknown>(
      mo.delete ?? (async ({ id }) => {
        const res = await fetch(`${base}/${id}`, { method: "DELETE" });
        if (!res.ok) throw new Error(`Delete failed: ${res.status}`);
      }),
      { queryKey, optimisticUpdate: deleteOptimistic(model) },
    );

    return { useCreate, useUpdate, useDelete };
  };

  return { useSource, makeMutation };
}
