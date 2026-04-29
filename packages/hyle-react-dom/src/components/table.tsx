import { useHyle, componentKeyForField } from "@tty-pt/hyle-react";
import type { Blueprint, Row, HyleListState, HyleFiltersState, HyleFieldComponents } from "@tty-pt/hyle-react";
import type { ComponentType } from "react";

// ── HyleTableBody ─────────────────────────────────────────────────────────────

export function HyleTableBody({
  blueprint: blueprintProp,
  list,
  filters,
  onRowClick,
  selectedId,
  components,
}: {
  blueprint?: Blueprint;
  list: HyleListState;
  filters?: HyleFiltersState;
  onRowClick?: (row: Row) => void;
  selectedId?: unknown;
  components?: HyleFieldComponents;
}) {
  const hyle = useHyle();
  const blueprint = blueprintProp ?? hyle.blueprint;

  if (!blueprint) {
    return <div className="hyle-table-error">No blueprint provided. Pass a blueprint prop or set one on HyleProvider.</div>;
  }

  if (list.status === "loading") {
    return <div className="hyle-table-loading">Loading…</div>;
  }

  if (list.status === "error") {
    return <div className="hyle-table-error">{list.error.message}</div>;
  }

  const columns = hyle.columns(blueprint, list.manifest);

  return (
    <div className="hyle-table-wrap">
      <table className="hyle-table">
        <thead className="hyle-thead">
          <tr>
            {columns.map((col) => {
              const FilterInput: ComponentType | undefined = filters?.Filter[col.key];
              const isActive = list.sortField === col.key;
              return (
                <th key={col.key} className="hyle-th">
                  <div className="hyle-th-inner">
                    <button
                      type="button"
                      className="hyle-sort-btn"
                      onClick={() => {
                        if (isActive) {
                          list.setSortAscending(!list.sortAscending);
                        } else {
                          list.setSortField(col.key);
                          list.setSortAscending(true);
                        }
                      }}
                    >
                      {col.label}
                      {isActive ? (list.sortAscending ? " ▲" : " ▼") : ""}
                    </button>
                    {FilterInput && (
                      <div className="hyle-col-filter">
                        <FilterInput />
                      </div>
                    )}
                  </div>
                </th>
              );
            })}
          </tr>
        </thead>
        <tbody className="hyle-tbody">
          {list.rows.length === 0 ? (
            <tr>
              <td colSpan={columns.length} className="hyle-empty">
                No results match the current filters.
              </td>
            </tr>
          ) : (
            list.rows.map((row) => {
              const selected = selectedId !== undefined && selectedId === row.id;
              return (
                <tr
                  key={String(row.id)}
                  className={
                    selected ? "hyle-tr hyle-tr--selected"
                    : onRowClick ? "hyle-tr hyle-tr--clickable"
                    : "hyle-tr"
                  }
                  onClick={onRowClick ? () => onRowClick(row) : undefined}
                >
                  {columns.map((col) => {
                    const value = (row as Record<string, unknown>)[col.key];
                    const compKey = componentKeyForField(col.field.type);
                    const ValueComp = components?.[compKey] as ComponentType<Record<string, unknown>> | undefined;
                    return (
                      <td key={col.key} className="hyle-td">
                        {ValueComp ? (
                          <ValueComp
                            value={value}
                            row={row}
                            field={col.field}
                            column={col}
                            result={list.status === "ready" ? list.result : null}
                            modelName={list.manifest?.base ?? ""}
                            blueprint={blueprint}
                          />
                        ) : (
                          hyle.displayValue(
                            blueprint,
                            list.result,
                            list.manifest.base,
                            col.key,
                            value,
                          )
                        )}
                      </td>
                    );
                  })}
                </tr>
              );
            })
          )}
        </tbody>
      </table>
    </div>
  );
}

// ── HyleTableFilters ──────────────────────────────────────────────────────────

export function HyleTableFilters({ filters }: { filters: HyleFiltersState }) {
  return (
    <div className="hyle-table-filters">
      <button type="button" onClick={filters.filterApply}>Apply</button>
      <button type="button" onClick={filters.filterClear}>Clear</button>
    </div>
  );
}

// ── HyleTablePagination ───────────────────────────────────────────────────────

export function HyleTablePagination({ list }: { list: HyleListState }) {
  if (list.status !== "ready") return null;

  return (
    <div className="hyle-pagination">
      <div className="hyle-pagination-controls">
        <button
          type="button"
          onClick={() => list.setPage(list.page - 1)}
          disabled={list.page <= 1}
        >
          ← Prev
        </button>
        <span>Page {list.page}</span>
        <button
          type="button"
          onClick={() => list.setPage(list.page + 1)}
          disabled={list.rows.length < list.perPage}
        >
          Next →
        </button>
        <select
          value={list.perPage}
          onChange={(e) => {
            list.setPerPage(Number(e.target.value));
            list.setPage(1);
          }}
        >
          {list.perPageOptions.map((n) => (
            <option key={n} value={n}>{n} / page</option>
          ))}
        </select>
      </div>
      <span className="hyle-row-count">
        {list.rows.length} of {list.result.total} rows
      </span>
    </div>
  );
}

// ── HyleTable ─────────────────────────────────────────────────────────────────

export function HyleTable({
  blueprint,
  list,
  filters,
  onRowClick,
  selectedId,
  components,
}: {
  blueprint?: Blueprint;
  list: HyleListState;
  filters?: HyleFiltersState;
  onRowClick?: (row: Row) => void;
  selectedId?: unknown;
  components?: HyleFieldComponents;
}) {
  return (
    <div className="hyle-table-container">
      <HyleTableBody
        blueprint={blueprint}
        list={list}
        filters={filters}
        onRowClick={onRowClick}
        selectedId={selectedId}
        components={components}
      />
      <HyleTablePagination list={list} />
    </div>
  );
}
