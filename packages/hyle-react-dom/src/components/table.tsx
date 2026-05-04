import { createContext, useContext, useRef, useEffect, type ReactNode } from "react";
import { useHyle, componentKeyForField } from "@tty-pt/hyle-react";
import type { Blueprint, Row, HyleListState, HyleFiltersState, HyleFieldComponents } from "@tty-pt/hyle-react";
import type { ComponentType } from "react";

// ── HyleFiltersContext ────────────────────────────────────────────────────────

const HyleFiltersContext = createContext<HyleFiltersState | null>(null);

function useFiltersContext(): HyleFiltersState | null {
  return useContext(HyleFiltersContext);
}

// ── HyleTableBody ─────────────────────────────────────────────────────────────

export function HyleTableBody({
  blueprint: blueprintProp,
  list,
  filters,
  onRowClick,
  selectedId,
  rowHref,
  components,
}: {
  blueprint?: Blueprint;
  list: HyleListState;
  filters?: HyleFiltersState;
  onRowClick?: (row: Row) => void;
  selectedId?: unknown;
  rowHref?: (row: Row) => string;
  components?: HyleFieldComponents;
}) {
  const hyle = useHyle();
  const blueprint = blueprintProp ?? hyle.blueprint;

  if (!blueprint) {
    return <div className="hyle-error">No blueprint provided. Pass a blueprint prop or set one on HyleProvider.</div>;
  }

  if (list.status === "loading") {
    return <div className="hyle-error">Loading…</div>;
  }

  if (list.status === "error") {
    return <div className="hyle-error">{list.error.message}</div>;
  }

  const columns = hyle.columns(blueprint, list.manifest);

  return (
    <div className="hyle-table-wrap">
      <table>
        <thead>
          <tr>
            {columns.map((col) => {
              const isActive = list.sortField === col.key;
              return (
                <th key={col.key}>
                  <button
                    type="button"
                    className="hyle-sort-button"
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
                </th>
              );
            })}
          </tr>
        </thead>
        <tbody>
          {list.rows.length === 0 ? (
            <tr>
              <td colSpan={columns.length} className="hyle-empty-state">
                No results match the current filters.
              </td>
            </tr>
          ) : (
            list.rows.map((row) => {
              const selected = selectedId !== undefined && selectedId === row.id;
              const href = rowHref ? rowHref(row) : undefined;
              return (
                <tr
                  key={String(row.id)}
                  className={
                    selected ? "hyle-row-selected"
                    : (onRowClick || href) ? "hyle-row-clickable"
                    : ""
                  }
                  onClick={onRowClick ? () => onRowClick(row) : undefined}
                >
                  {columns.map((col, i) => {
                    const value = (row as Record<string, unknown>)[col.key];
                    const compKey = componentKeyForField(col.field.type);
                    const ValueComp = components?.[compKey] as ComponentType<Record<string, unknown>> | undefined;
                    const cellContent = ValueComp ? (
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
                    );
                    return (
                      <td key={col.key}>
                        {i === 0 && href
                          ? <a href={href}>{cellContent}</a>
                          : cellContent}
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

export function HyleTableFilters() {
  const filters = useFiltersContext();
  return (
    <div className="hyle-filter-actions">
      <button
        type="reset"
        onClick={(e) => { e.preventDefault(); filters?.filterClear(); }}
      >
        Clear
      </button>
      <button type="submit">Apply</button>
    </div>
  );
}

// ── HyleTableFilterBar ────────────────────────────────────────────────────────

export function HyleTableFilterBar({
  filters,
  only,
  children,
}: {
  filters: HyleFiltersState;
  only?: string[];
  children?: ReactNode;
}) {
  const visible = only
    ? filters.fields.filter((f) => only.includes(f.key))
    : filters.fields;

  return (
    <div className="hyle-filter-bar">
      {visible.map((f) => {
        const FilterInput = filters.Filter[f.key];
        if (!FilterInput) return null;
        const isFieldset = f.field.type.kind === "array";
        if (isFieldset) return <FilterInput key={f.key} />;
        return (
          <label key={f.key}>
            {f.label}
            <FilterInput />
          </label>
        );
      })}
      {children}
    </div>
  );
}

// ── HyleTablePagination ───────────────────────────────────────────────────────

export function HyleTablePagination({ list }: { list: HyleListState }) {
  if (list.status !== "ready") return null;

  return (
    <div className="hyle-table-footer">
      <div className="hyle-pagination">
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
    <>
      <HyleTableBody
        blueprint={blueprint}
        list={list}
        filters={filters}
        onRowClick={onRowClick}
        selectedId={selectedId}
        components={components}
      />
      <HyleTablePagination list={list} />
    </>
  );
}

export function HyleTablePanel({
  list,
  filters,
  onRowClick,
  selectedId,
  rowHref,
  components,
  children,
}: {
  list: HyleListState;
  filters?: HyleFiltersState;
  onRowClick?: (row: Row) => void;
  selectedId?: unknown;
  rowHref?: (row: Row) => string;
  components?: HyleFieldComponents;
  children?: ReactNode;
}) {
  const formRef = useRef<HTMLFormElement>(null);

  useEffect(() => {
    const form = formRef.current;
    if (!form) return;
    const handler = (e: Event) => e.preventDefault();
    form.addEventListener("submit", handler, { capture: true });
    return () => form.removeEventListener("submit", handler, { capture: true });
  }, []);

  return (
    <HyleFiltersContext.Provider value={filters ?? null}>
      <form
        ref={formRef}
        method="get"
        data-hyle-panel="true"
        onSubmit={(e) => {
          e.preventDefault();
          filters?.filterApply();
          list.setPage(1);
        }}
      >
        {children}
        <HyleTableBody
          list={list}
          onRowClick={onRowClick}
          selectedId={selectedId}
          rowHref={rowHref}
          components={components}
        />
        <HyleTablePagination list={list} />
      </form>
    </HyleFiltersContext.Provider>
  );
}
