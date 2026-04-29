export function createHyleClient(loadWasm) {
    let wasm = null;
    let loadPromise = null;
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
    const client = {
        load,
        manifest(blueprint, query) {
            const module = getWasm();
            return JSON.parse(module.manifest(JSON.stringify(blueprint), JSON.stringify(query)));
        },
        resolve(blueprint, manifest, source) {
            const module = getWasm();
            return JSON.parse(module.resolve(JSON.stringify(blueprint), JSON.stringify(manifest), JSON.stringify(source)));
        },
        resolveAndView(blueprint, manifest, source) {
            const module = getWasm();
            const fn_ = requireWasmExport(module.resolve_and_view, "resolve_and_view");
            return JSON.parse(fn_(JSON.stringify(blueprint), JSON.stringify(manifest), JSON.stringify(source)));
        },
        resolveQuery(blueprint, query, source) {
            const module = getWasm();
            const fn_ = requireWasmExport(module.resolve_query, "resolve_query");
            return JSON.parse(fn_(JSON.stringify(blueprint), JSON.stringify(query), JSON.stringify(source)));
        },
        rowsArray(result) {
            const module = getWasm();
            const fn_ = requireWasmExport(module.rows_array, "rows_array");
            return JSON.parse(fn_(JSON.stringify(result)));
        },
        makeField(kind, label, entity) {
            const module = getWasm();
            const fn_ = requireWasmExport(module.make_field, "make_field");
            return JSON.parse(fn_(kind, label, JSON.stringify(entity ?? null)));
        },
        columns(blueprint, manifest) {
            const module = getWasm();
            const columns = requireWasmExport(module.columns, "columns");
            return JSON.parse(columns(JSON.stringify(blueprint), JSON.stringify(manifest)));
        },
        filterRows(rows, filters) {
            if (!filters)
                return rows;
            const module = getWasm();
            const filterRows = requireWasmExport(module.filter_rows, "filter_rows");
            return JSON.parse(filterRows(JSON.stringify(rows), JSON.stringify(filters)));
        },
        displayValue(blueprint, result, modelName, fieldName, value) {
            const module = getWasm();
            const displayValue = requireWasmExport(module.display_value, "display_value");
            return displayValue(JSON.stringify(blueprint), JSON.stringify(result), modelName, fieldName, JSON.stringify(value));
        },
        filterLayout(blueprint, manifest) {
            const module = getWasm();
            const fn_ = requireWasmExport(module.filter_layout, "filter_layout");
            return JSON.parse(fn_(JSON.stringify(blueprint), JSON.stringify(manifest)));
        },
        formaToQuery(forma, tableName, context, id) {
            const module = getWasm();
            const fn_ = requireWasmExport(module.forma_to_query, "forma_to_query");
            return JSON.parse(fn_(JSON.stringify(forma), tableName, JSON.stringify(context), JSON.stringify(id ?? null)));
        },
        purifyRow(blueprint, modelName, row) {
            const module = getWasm();
            const fn_ = requireWasmExport(module.purify_row, "purify_row");
            return JSON.parse(fn_(JSON.stringify(blueprint), modelName, JSON.stringify(row)));
        },
        applyView(rows, manifest) {
            const module = getWasm();
            const fn_ = requireWasmExport(module.apply_view, "apply_view");
            return JSON.parse(fn_(JSON.stringify(rows), JSON.stringify(manifest)));
        },
        isSingle(manifest, result) {
            const module = getWasm();
            const fn_ = requireWasmExport(module.is_single, "is_single");
            return fn_(JSON.stringify(manifest), JSON.stringify(result));
        },
    };
    return client;
}
function requireWasmExport(fn, name) {
    if (!fn) {
        throw new Error(`hyle WASM export '${name}' is missing; rebuild the hyle-wasm package`);
    }
    return fn;
}
//# sourceMappingURL=client.js.map