import { createContext, createElement, useCallback, useContext, useEffect, useMemo, useRef, useState, } from "react";
import { useQuery, useMutation as useTanstackMutation, useQueryClient, } from "@tanstack/react-query";
export { createHyleClient, field, } from "@tty-pt/hyle";
const HyleContext = createContext(null);
export function HyleProvider({ client, blueprint = null, children, }) {
    const [state, setState] = useState({ status: "loading", error: null });
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
            .catch((error) => {
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
    const value = useMemo(() => ({ client, blueprint, status: state.status, error: state.error }), [client, blueprint, state.error, state.status]);
    return createElement(HyleContext.Provider, { value }, children);
}
export function useHyle() {
    const context = useContext(HyleContext);
    if (!context) {
        throw new Error("useHyle must be used inside HyleProvider");
    }
    const manifest = useCallback((blueprint, query) => context.client.manifest(blueprint, query), [context.client]);
    const resolve = useCallback((blueprint, manifest, source) => context.client.resolve(blueprint, manifest, source), [context.client]);
    const resolveAndView = useCallback((blueprint, manifest, source) => context.client.resolveAndView(blueprint, manifest, source), [context.client]);
    const resolveQuery = useCallback((blueprint, query, source) => context.client.resolveQuery(blueprint, query, source), [context.client]);
    const columns = useCallback((blueprint, manifest) => context.client.columns(blueprint, manifest), [context.client]);
    const filterRows = useCallback((rows, filters) => context.client.filterRows(rows, filters), [context.client]);
    const applyView = useCallback((rows, manifest) => context.client.applyView(rows, manifest), [context.client]);
    const displayValue = useCallback((blueprint, result, modelName, fieldName, value) => context.client.displayValue(blueprint, result, modelName, fieldName, value), [context.client]);
    const filterLayout = useCallback((blueprint, manifest) => context.client.filterLayout(blueprint, manifest), [context.client]);
    const formaToQuery = useCallback((forma, tableName, context_, id) => context.client.formaToQuery(forma, tableName, context_, id), [context.client]);
    const purifyRow = useCallback((blueprint, modelName, row) => context.client.purifyRow(blueprint, modelName, row), [context.client]);
    const isSingle = useCallback((manifest, result) => context.client.isSingle(manifest, result), [context.client]);
    const rowsArray = useCallback((result) => context.client.rowsArray(result), [context.client]);
    const makeField = useCallback((kind, label, entity) => context.client.makeField(kind, label, entity), [context.client]);
    const modelFields = useCallback((blueprint, modelName) => {
        const model = blueprint.models[modelName];
        if (!model)
            return [];
        return Object.entries(model.fields).map(([key, field]) => ({
            key,
            label: field.label,
            field,
        }));
    }, []);
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
export function useHyleResolved({ blueprint, query, source, }) {
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
        }
        catch (error) {
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
// ─── makeHyleHooks ────────────────────────────────────────────────────────────
/** Model name for the forma configuration entity. */
const FORMA_MODEL = "forma";
export function makeHyleHooks({ blueprint, adapter, validate, validateRow, components = {}, }) {
    const { useSource, makeMutation } = adapter;
    // ── useManifest ───────────────────────────────────────────────────────────
    function useManifest(query) {
        const hyle = useHyle();
        const prevRef = useRef(null);
        const state = useMemo(() => {
            if (hyle.status === "loading") {
                return { status: "loading", error: null, manifest: null };
            }
            if (hyle.status === "error") {
                return {
                    status: "error",
                    error: hyle.error ?? new Error("hyle WASM failed to load"),
                    manifest: null,
                };
            }
            try {
                return {
                    status: "ready",
                    error: null,
                    manifest: hyle.manifest(blueprint, query),
                };
            }
            catch (error) {
                return {
                    status: "error",
                    error: toError(error),
                    manifest: null,
                };
            }
        }, [hyle.error, hyle.manifest, hyle.status, query]);
        // Stabilise the returned state by JSON identity so downstream memos
        // (especially the Filter component map) don't rebuild on every render
        // just because WASM returned a new object with the same content.
        const json = JSON.stringify(state);
        if (prevRef.current?.json === json)
            return prevRef.current.state;
        prevRef.current = { json, state };
        return state;
    }
    function useData(query) {
        const hyle = useHyle();
        const planned = useManifest(query);
        const sourceResult = useSource(planned.manifest, query);
        return useMemo(() => {
            const emptyFields = [];
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
                const { outcome: result, rows, isSingle, columns } = hyle.resolveAndView(blueprint, planned.manifest, sourceState.source);
                const row = isSingle ? (rows[0] ?? null) : null;
                // Build per-field accessor array for single-record usage
                const fields = columns.map((col) => ({
                    key: col.key,
                    label: col.label,
                    field: col.field,
                    raw: row?.[col.key] ?? null,
                    render: () => hyle.displayValue(blueprint, result, planned.manifest.base, col.key, row?.[col.key]),
                }));
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
            }
            catch (error) {
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
    function useList(query, options = {}) {
        const { perPageOptions = [5, 10, 20] } = options;
        const [page, setPage] = useState(query.page ?? 1);
        const [perPage, setPerPage] = useState(query.perPage ?? perPageOptions[0] ?? 20);
        const [sortField, setSortField] = useState(query.sort?.field);
        const [sortAscending, setSortAscending] = useState(query.sort?.ascending ?? true);
        const effectiveQuery = useMemo(() => ({
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
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    function useFilters(query, options = {}) {
        const hyle = useHyle();
        const { change } = options;
        const [committed, setCommitted] = useState({});
        const [formData, setFormData] = useState({});
        const [filterResetKey, setFilterResetKey] = useState(0);
        const [purifyErrors, setPurifyErrors] = useState(null);
        const effectiveQuery = useMemo(() => ({
            ...query,
            where: { ...(query.where ?? {}), ...committed },
        }), [committed, query]);
        // Fetch data internally: with id → seed form for edit mode; without id → get lookups for reference filters.
        const hasId = Boolean(query.where && "id" in query.where);
        const seedData = useData(hasId ? effectiveQuery : { model: effectiveQuery.model });
        useEffect(() => {
            if (seedData.status === "ready" && seedData.row !== null) {
                const seeded = {};
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
        const setField = useCallback((name, value) => {
            setFormData((prev) => ({ ...prev, [name]: value }));
        }, []);
        const filterApply = useCallback(() => {
            setCommitted((prev) => ({ ...prev, ...formData }));
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
        const validate = useCallback(() => {
            const activeFormData = Object.fromEntries(Object.entries(formDataRef.current).filter(([k]) => {
                if (!changeRef.current)
                    return true;
                if (!(k in changeRef.current))
                    return true;
                return changeRef.current[k](blueprint.models[effectiveQuery.model]?.fields[k] ?? { label: k, type: { kind: "primitive", primitive: "string" } }) !== null;
            }));
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
        const resultRef = useRef(null);
        resultRef.current = seedData.status === "ready" ? seedData.result : null;
        const Filter = useMemo(() => {
            const cols = manifest ? hyle.columns(blueprint, manifest) : [];
            const map = {};
            for (const col of cols) {
                const key = col.key;
                // Apply per-field change transform
                const changed = changeRef.current?.[key]?.(col.field);
                if (changed === null)
                    continue; // excluded
                const effectiveField = changed ?? col.field;
                const effectiveLabel = effectiveField.label ?? col.label;
                const fieldComponent = changed?.component;
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
                        onChange: (v) => setFieldRef.current(col.key, v),
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
    function useForma(tableName, id, options = {}) {
        const hyle = useHyle();
        const { context = "column" } = options;
        // Query the forma entity
        const formaQuery = useMemo(() => ({
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
            const forma = formaData.row;
            if (!forma.fields?.length)
                return [null, forma];
            try {
                // Delegate query derivation to Rust via WASM
                const derivedQuery = hyle.formaToQuery(forma, tableName, context, id);
                return [derivedQuery, forma];
            }
            catch {
                return [null, forma];
            }
        }, [context, hyle, id, formaData.row, formaData.status, tableName]);
    }
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    function useForm(query, options = {}) {
        const isEdit = Boolean(query.where && "id" in query.where);
        const model = query.model;
        // Always call both hooks unconditionally (Rules of Hooks).
        // Only one will be used based on isEdit, but hooks must not be conditional.
        let mutation = null;
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
        const onSubmit = useCallback((e) => {
            e?.preventDefault?.();
            const errors = filtersRef.current.validate();
            if (errors !== null)
                return;
            const id = filtersRef.current.formData.id;
            mutationRef.current?.mutate({ id, data: filtersRef.current.formData });
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
    function useMutation(model) {
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
/**
 * Standard optimistic update for a save (PUT) mutation.
 * Replaces the matching row in `source[model].result` with merged field values.
 *
 * ```ts
 * optimisticUpdate: saveOptimistic("user")
 * ```
 */
export function saveOptimistic(model) {
    return (current, { id, data }) => ({
        ...current,
        [model]: {
            ...current[model],
            result: current[model].result.map((r) => r.id === id ? { ...r, ...data } : r),
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
export function addOptimistic(model) {
    return (current, { data }) => {
        const rows = current[model].result;
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
export function deleteOptimistic(model) {
    return (current, { id }) => ({
        ...current,
        [model]: {
            total: current[model].total - 1,
            result: current[model].result.filter((r) => r.id !== id),
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
export function componentKeyForField(fieldType) {
    if (fieldType.kind === "primitive") {
        const map = {
            string: "String",
            number: "Number",
            boolean: "Boolean",
            file: "File",
        };
        return map[fieldType.primitive] ?? "String";
    }
    if (fieldType.kind === "reference")
        return "Reference";
    if (fieldType.kind === "array")
        return "Array";
    return "Shape";
}
/** Resolve the filter component key from a FieldType */
function filterComponentKeyForField(fieldType) {
    if (fieldType.kind === "primitive") {
        const map = {
            string: "FilterString",
            number: "FilterNumber",
            boolean: "FilterBoolean",
            file: "FilterFile",
        };
        return map[fieldType.primitive] ?? "FilterString";
    }
    if (fieldType.kind === "reference")
        return "FilterReference";
    if (fieldType.kind === "array")
        return "FilterArray";
    return "FilterShape";
}
/** Internal component that renders a filter input using the component map or a default <input> */
function FilterInputComponent({ fieldName, field, label, value, result, components, fieldComponent, onChange, }) {
    const filterKey = filterComponentKeyForField(field.type);
    const CustomFilter = fieldComponent ??
        components[filterKey];
    if (CustomFilter) {
        return createElement(CustomFilter, { fieldName, field, label, value, result, onChange });
    }
    // Default: plain text input
    return createElement("input", {
        type: "text",
        "aria-label": label,
        placeholder: label,
        value: value ?? "",
        onChange: (e) => onChange(e.target.value),
    });
}
function normalizeSourceResult(result) {
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
function isSource(result) {
    return Boolean(result && typeof result === "object" && !("source" in result));
}
function toError(error) {
    return error instanceof Error ? error : new Error(String(error));
}
export function hyleQueryKey(query) {
    return [
        "hyle",
        query.model,
        query.select ?? [],
        query.where ?? {},
        query.page ?? null,
        query.sort ?? null,
    ];
}
export function makeReactQuerySource(queryFn) {
    return function useReactQuerySource(manifest, query) {
        // eslint-disable-next-line react-hooks/rules-of-hooks
        const { data, isPending, isError, error } = useQuery({
            queryKey: hyleQueryKey(query),
            queryFn: () => queryFn(manifest, query),
        });
        if (isPending)
            return { loading: true };
        if (isError)
            return { error: error instanceof Error ? error : new Error(String(error)) };
        return data ?? { loading: true };
    };
}
export function makeReactQueryMutation(mutationFn, options = {}) {
    const { queryKey = ["hyle"], optimisticUpdate, onSuccess } = options;
    return function useReactQueryMutation() {
        // eslint-disable-next-line react-hooks/rules-of-hooks
        const queryClient = useQueryClient();
        // eslint-disable-next-line react-hooks/rules-of-hooks
        const mutation = useTanstackMutation({
            mutationFn,
            onMutate: async (input) => {
                if (!optimisticUpdate)
                    return { snapshot: undefined };
                await queryClient.cancelQueries({ queryKey });
                const snapshot = queryClient.getQueryData(queryKey);
                if (snapshot !== undefined) {
                    queryClient.setQueryData(queryKey, (current) => current ? optimisticUpdate(current, input) : current);
                }
                return { snapshot };
            },
            onError: (_err, _input, context) => {
                if (context?.snapshot !== undefined) {
                    queryClient.setQueryData(queryKey, context.snapshot);
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
export function makeRestAdapter(baseUrl, options = {}) {
    const { overrides = {} } = options;
    const defaultFetch = async (_manifest, _query) => {
        const res = await fetch(`${baseUrl}/source`);
        if (!res.ok)
            throw new Error(`Source fetch failed: ${res.status}`);
        return res.json();
    };
    const useSource = makeReactQuerySource((manifest, query) => {
        const modelOverride = overrides[query.model]?.fetch;
        return modelOverride ? modelOverride(manifest, query) : defaultFetch(manifest, query);
    });
    const makeMutation = (model) => {
        const mo = overrides[model] ?? {};
        const base = `${baseUrl}/${model}`;
        const queryKey = ["hyle", model];
        const useCreate = makeReactQueryMutation(mo.create ?? (async ({ data }) => {
            const res = await fetch(base, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(data),
            });
            if (!res.ok)
                throw new Error(`Create failed: ${res.status}`);
            return res.json();
        }), { queryKey, optimisticUpdate: addOptimistic(model) });
        const useUpdate = makeReactQueryMutation(mo.update ?? (async ({ id, data }) => {
            const res = await fetch(`${base}/${id}`, {
                method: "PUT",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify(data),
            });
            if (!res.ok)
                throw new Error(`Save failed: ${res.status}`);
            return res.json();
        }), { queryKey, optimisticUpdate: saveOptimistic(model) });
        const useDelete = makeReactQueryMutation(mo.delete ?? (async ({ id }) => {
            const res = await fetch(`${base}/${id}`, { method: "DELETE" });
            if (!res.ok)
                throw new Error(`Delete failed: ${res.status}`);
        }), { queryKey, optimisticUpdate: deleteOptimistic(model) });
        return { useCreate, useUpdate, useDelete };
    };
    return { useSource, makeMutation };
}
//# sourceMappingURL=index.js.map