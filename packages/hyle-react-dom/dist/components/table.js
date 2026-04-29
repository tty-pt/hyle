import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { useHyle, componentKeyForField } from "@tty-pt/hyle-react";
// ── HyleTableBody ─────────────────────────────────────────────────────────────
export function HyleTableBody({ blueprint: blueprintProp, list, filters, onRowClick, selectedId, components, }) {
    const hyle = useHyle();
    const blueprint = blueprintProp ?? hyle.blueprint;
    if (!blueprint) {
        return _jsx("div", { className: "hyle-table-error", children: "No blueprint provided. Pass a blueprint prop or set one on HyleProvider." });
    }
    if (list.status === "loading") {
        return _jsx("div", { className: "hyle-table-loading", children: "Loading\u2026" });
    }
    if (list.status === "error") {
        return _jsx("div", { className: "hyle-table-error", children: list.error.message });
    }
    const columns = hyle.columns(blueprint, list.manifest);
    return (_jsx("div", { className: "hyle-table-wrap", children: _jsxs("table", { className: "hyle-table", children: [_jsx("thead", { className: "hyle-thead", children: _jsx("tr", { children: columns.map((col) => {
                            const FilterInput = filters?.Filter[col.key];
                            const isActive = list.sortField === col.key;
                            return (_jsx("th", { className: "hyle-th", children: _jsxs("div", { className: "hyle-th-inner", children: [_jsxs("button", { type: "button", className: "hyle-sort-btn", onClick: () => {
                                                if (isActive) {
                                                    list.setSortAscending(!list.sortAscending);
                                                }
                                                else {
                                                    list.setSortField(col.key);
                                                    list.setSortAscending(true);
                                                }
                                            }, children: [col.label, isActive ? (list.sortAscending ? " ▲" : " ▼") : ""] }), FilterInput && (_jsx("div", { className: "hyle-col-filter", children: _jsx(FilterInput, {}) }))] }) }, col.key));
                        }) }) }), _jsx("tbody", { className: "hyle-tbody", children: list.rows.length === 0 ? (_jsx("tr", { children: _jsx("td", { colSpan: columns.length, className: "hyle-empty", children: "No results match the current filters." }) })) : (list.rows.map((row) => {
                        const selected = selectedId !== undefined && selectedId === row.id;
                        return (_jsx("tr", { className: selected ? "hyle-tr hyle-tr--selected"
                                : onRowClick ? "hyle-tr hyle-tr--clickable"
                                    : "hyle-tr", onClick: onRowClick ? () => onRowClick(row) : undefined, children: columns.map((col) => {
                                const value = row[col.key];
                                const compKey = componentKeyForField(col.field.type);
                                const ValueComp = components?.[compKey];
                                return (_jsx("td", { className: "hyle-td", children: ValueComp ? (_jsx(ValueComp, { value: value, row: row, field: col.field, column: col, result: list.status === "ready" ? list.result : null, modelName: list.manifest?.base ?? "", blueprint: blueprint })) : (hyle.displayValue(blueprint, list.result, list.manifest.base, col.key, value)) }, col.key));
                            }) }, String(row.id)));
                    })) })] }) }));
}
// ── HyleTableFilters ──────────────────────────────────────────────────────────
export function HyleTableFilters({ filters }) {
    return (_jsxs("div", { className: "hyle-table-filters", children: [_jsx("button", { type: "button", onClick: filters.filterApply, children: "Apply" }), _jsx("button", { type: "button", onClick: filters.filterClear, children: "Clear" })] }));
}
// ── HyleTablePagination ───────────────────────────────────────────────────────
export function HyleTablePagination({ list }) {
    if (list.status !== "ready")
        return null;
    return (_jsxs("div", { className: "hyle-pagination", children: [_jsxs("div", { className: "hyle-pagination-controls", children: [_jsx("button", { type: "button", onClick: () => list.setPage(list.page - 1), disabled: list.page <= 1, children: "\u2190 Prev" }), _jsxs("span", { children: ["Page ", list.page] }), _jsx("button", { type: "button", onClick: () => list.setPage(list.page + 1), disabled: list.rows.length < list.perPage, children: "Next \u2192" }), _jsx("select", { value: list.perPage, onChange: (e) => {
                            list.setPerPage(Number(e.target.value));
                            list.setPage(1);
                        }, children: list.perPageOptions.map((n) => (_jsxs("option", { value: n, children: [n, " / page"] }, n))) })] }), _jsxs("span", { className: "hyle-row-count", children: [list.rows.length, " of ", list.result.total, " rows"] })] }));
}
// ── HyleTable ─────────────────────────────────────────────────────────────────
export function HyleTable({ blueprint, list, filters, onRowClick, selectedId, components, }) {
    return (_jsxs("div", { className: "hyle-table-container", children: [_jsx(HyleTableBody, { blueprint: blueprint, list: list, filters: filters, onRowClick: onRowClick, selectedId: selectedId, components: components }), _jsx(HyleTablePagination, { list: list })] }));
}
//# sourceMappingURL=table.js.map