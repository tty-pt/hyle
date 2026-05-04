import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { createElement, type ReactNode } from "react";
import { HyleProvider } from "@tty-pt/hyle-react";
import type { HyleClient, Column, Field, Result, Row, Manifest, Source } from "@tty-pt/hyle";
import { HyleTableBody, HyleTableFilterBar, HyleTableFilters, HyleTablePagination, HyleTable, HyleTablePanel } from "../src/components/table";
import type { HyleListState, HyleFiltersState } from "@tty-pt/hyle-react";

// ── Mock client ───────────────────────────────────────────────────────────────

const defaultManifest: Manifest = { base: "user", fields: ["name", "active"] };
const defaultColumns: Column[] = [
  { key: "name",   label: "Name",   field: { label: "Name",   type: { kind: "primitive", primitive: "string"  } } },
  { key: "active", label: "Active", field: { label: "Active", type: { kind: "primitive", primitive: "boolean" } } },
];
const defaultRows: Row[] = [
  { id: "1", name: "Alice", active: true  },
  { id: "2", name: "Bob",   active: false },
];
const defaultResult: Result = { rows: defaultRows, total: 2 };
const defaultSource: Source = { user: { result: defaultRows, total: 2 } };

function makeClient(overrides: Partial<HyleClient> = {}): HyleClient {
  return {
    load:           vi.fn(() => Promise.resolve()),
    manifest:       vi.fn(() => defaultManifest),
    resolve:        vi.fn(() => defaultResult),
    resolveAndView: vi.fn(() => ({ outcome: defaultResult, rows: defaultRows, isSingle: false, columns: defaultColumns })),
    resolveQuery:   vi.fn(() => ({ manifest: defaultManifest, result: defaultResult, rows: defaultRows })),
    rowsArray:      vi.fn((r: Result) => (Array.isArray(r.rows) ? r.rows : [r.rows])),
    makeField:      vi.fn(() => ({ label: "X", type: { kind: "primitive", primitive: "string" } } as Field)),
    columns:        vi.fn(() => defaultColumns),
    filterRows:     vi.fn((rows: Row[]) => rows),
    applyView:      vi.fn((rows: Row[]) => rows),
    displayValue:   vi.fn((_bp, _res, _base, field, value) => `${field}:${value}`),
    filterLayout:   vi.fn(() => [defaultColumns]),
    formaToQuery:   vi.fn(() => ({ model: "user" })),
    purifyRow:      vi.fn(() => null),
    isSingle:       vi.fn(() => false),
    ...overrides,
  };
}

function wrap(client: HyleClient) {
  return ({ children }: { children: ReactNode }) =>
    createElement(HyleProvider, { client, children });
}

// ── Fixtures ──────────────────────────────────────────────────────────────────

const blueprint = {
  models: {
    user: {
      fields: {
        name:   { label: "Name",   type: { kind: "primitive" as const, primitive: "string"  as const } },
        active: { label: "Active", type: { kind: "primitive" as const, primitive: "boolean" as const } },
      },
    },
  },
};

function makeReadyList(overrides: Partial<HyleListState> = {}): HyleListState {
  return ({
    status: "ready" as const,
    error: null,
    manifest: defaultManifest,
    result: defaultResult,
    rows: defaultRows,
    row: null,
    fields: [],
    validation: null,
    query: { model: "user" },
    page: 1,
    perPage: 5,
    perPageOptions: [5, 10, 20],
    setPage: vi.fn(),
    setPerPage: vi.fn(),
    sortField: undefined,
    sortAscending: true,
    setSortField: vi.fn(),
    setSortAscending: vi.fn(),
    ...overrides,
  }) as HyleListState;
}

function makeLoadingList(): HyleListState {
  return {
    status: "loading" as const,
    error: null,
    manifest: null,
    result: null,
    rows: [],
    row: null,
    fields: [],
    validation: null,
    query: { model: "user" },
    page: 1,
    perPage: 5,
    perPageOptions: [5, 10, 20],
    setPage: vi.fn(),
    setPerPage: vi.fn(),
    sortField: undefined,
    sortAscending: true,
    setSortField: vi.fn(),
    setSortAscending: vi.fn(),
  };
}

function makeErrorList(): HyleListState {
  return {
    status: "error" as const,
    error: new Error("fetch failed"),
    manifest: null,
    result: null,
    rows: [],
    row: null,
    fields: [],
    validation: null,
    query: { model: "user" },
    page: 1,
    perPage: 5,
    perPageOptions: [5, 10, 20],
    setPage: vi.fn(),
    setPerPage: vi.fn(),
    sortField: undefined,
    sortAscending: true,
    setSortField: vi.fn(),
    setSortAscending: vi.fn(),
  };
}

function makeFilters(overrides: Partial<HyleFiltersState> = {}): HyleFiltersState {
  return {
    query: { model: "user" },
    formData: {},
    setField: vi.fn(),
    filterApply: vi.fn(),
    filterClear: vi.fn(),
    filterResetKey: 0,
    fields: [
      { key: "name",   label: "Name",   field: { label: "Name",   type: { kind: "primitive", primitive: "string"  } }, raw: null, render: () => "" },
      { key: "active", label: "Active", field: { label: "Active", type: { kind: "primitive", primitive: "boolean" } }, raw: null, render: () => "" },
    ],
    Filter: {
      name:   () => createElement("input", { "data-testid": "filter-name",   placeholder: "Name"   }),
      active: () => createElement("input", { "data-testid": "filter-active", placeholder: "Active" }),
    },
    onSubmit: vi.fn(),
    mutation: null,
    purifyErrors: null,
    ...overrides,
  };
}

// ── HyleTableBody ─────────────────────────────────────────────────────────────

describe("HyleTableBody", () => {
  let client: HyleClient;
  beforeEach(() => { client = makeClient(); });

  it("renders loading state", () => {
    render(
      createElement(HyleTableBody, { blueprint, list: makeLoadingList() }),
      { wrapper: wrap(client) },
    );
    expect(screen.getByText("Loading…")).toBeTruthy();
  });

  it("renders error state", () => {
    render(
      createElement(HyleTableBody, { blueprint, list: makeErrorList() }),
      { wrapper: wrap(client) },
    );
    expect(screen.getByText("fetch failed")).toBeTruthy();
  });

  it("renders column headers from blueprint", () => {
    render(
      createElement(HyleTableBody, { blueprint, list: makeReadyList() }),
      { wrapper: wrap(client) },
    );
    expect(screen.getByText("Name")).toBeTruthy();
    expect(screen.getByText("Active")).toBeTruthy();
  });

  it("renders row cell values via displayValue", () => {
    render(
      createElement(HyleTableBody, { blueprint, list: makeReadyList() }),
      { wrapper: wrap(client) },
    );
    expect(screen.getByText("name:Alice")).toBeTruthy();
    expect(screen.getByText("name:Bob")).toBeTruthy();
  });

  it("renders empty state when rows is empty", () => {
    render(
      createElement(HyleTableBody, { blueprint, list: makeReadyList({ rows: [] }) }),
      { wrapper: wrap(client) },
    );
    expect(screen.getByText(/No results/)).toBeTruthy();
  });

  it("calls setSortField when sort button clicked on inactive column", () => {
    const setSortField = vi.fn();
    render(
      createElement(HyleTableBody, { blueprint, list: makeReadyList({ setSortField }) }),
      { wrapper: wrap(client) },
    );
    fireEvent.click(screen.getByText("Name"));
    expect(setSortField).toHaveBeenCalledWith("name");
  });

  it("calls setSortAscending when sort button clicked on active column", () => {
    const setSortAscending = vi.fn();
    render(
      createElement(HyleTableBody, {
        blueprint,
        list: makeReadyList({ sortField: "name", sortAscending: true, setSortAscending }),
      }),
      { wrapper: wrap(client) },
    );
    fireEvent.click(screen.getByText("Name ▲"));
    expect(setSortAscending).toHaveBeenCalledWith(false);
  });

  it("renders filter inputs per column when filters provided", () => {
    render(
      createElement(HyleTableFilterBar, { filters: makeFilters() }),
      { wrapper: wrap(client) },
    );
    expect(screen.getByTestId("filter-name")).toBeTruthy();
    expect(screen.getByTestId("filter-active")).toBeTruthy();
  });

  it("calls onRowClick with the correct row", () => {
    const onRowClick = vi.fn();
    render(
      createElement(HyleTableBody, { blueprint, list: makeReadyList(), onRowClick }),
      { wrapper: wrap(client) },
    );
    fireEvent.click(screen.getAllByRole("row")[1]);
    expect(onRowClick).toHaveBeenCalledWith(defaultRows[0]);
  });
});

// ── HyleTablePagination ───────────────────────────────────────────────────────

describe("HyleTablePagination", () => {
  it("disables prev button on page 1", () => {
    render(createElement(HyleTablePagination, { list: makeReadyList({ page: 1 }) }));
    expect((screen.getByText("← Prev") as HTMLButtonElement).disabled).toBe(true);
  });

  it("calls setPage with page-1 when prev clicked", () => {
    const setPage = vi.fn();
    render(createElement(HyleTablePagination, { list: makeReadyList({ page: 3, setPage }) }));
    fireEvent.click(screen.getByText("← Prev"));
    expect(setPage).toHaveBeenCalledWith(2);
  });

  it("disables next button when rows.length < perPage", () => {
    render(createElement(HyleTablePagination, { list: makeReadyList({ rows: defaultRows, perPage: 20 }) }));
    expect((screen.getByText("Next →") as HTMLButtonElement).disabled).toBe(true);
  });

  it("calls setPage with page+1 when next clicked", () => {
    const setPage = vi.fn();
    render(createElement(HyleTablePagination, { list: makeReadyList({ page: 1, rows: defaultRows, perPage: 2, setPage }) }));
    fireEvent.click(screen.getByText("Next →"));
    expect(setPage).toHaveBeenCalledWith(2);
  });

  it("renders perPageOptions in select", () => {
    render(createElement(HyleTablePagination, { list: makeReadyList({ perPageOptions: [5, 10, 20] }) }));
    const opts = screen.getAllByRole("option") as HTMLOptionElement[];
    expect(opts.map((o) => o.value)).toEqual(["5", "10", "20"]);
  });

  it("calls setPerPage and resets page on select change", () => {
    const setPerPage = vi.fn();
    const setPage = vi.fn();
    render(createElement(HyleTablePagination, { list: makeReadyList({ setPerPage, setPage }) }));
    fireEvent.change(screen.getByRole("combobox"), { target: { value: "10" } });
    expect(setPerPage).toHaveBeenCalledWith(10);
    expect(setPage).toHaveBeenCalledWith(1);
  });

  it("shows row count", () => {
    render(createElement(HyleTablePagination, { list: makeReadyList() }));
    expect(screen.getByText(/2 of 2 rows/)).toBeTruthy();
  });

  it("renders nothing when list is loading", () => {
    const { container } = render(createElement(HyleTablePagination, { list: makeLoadingList() }));
    expect(container.firstChild).toBeNull();
  });
});

// ── HyleTableFilters ──────────────────────────────────────────────────────────

describe("HyleTableFilters", () => {
  let client: HyleClient;
  beforeEach(() => { client = makeClient(); });

  it("calls filterApply when Apply clicked", () => {
    const filterApply = vi.fn();
    render(
      createElement(HyleTablePanel, {
        list: makeReadyList(),
        filters: makeFilters({ filterApply }),
        children: createElement(HyleTableFilters),
      }),
      { wrapper: wrap(client) },
    );
    fireEvent.click(screen.getByText("Apply"));
    expect(filterApply).toHaveBeenCalled();
  });

  it("calls filterClear when Clear clicked", () => {
    const filterClear = vi.fn();
    render(
      createElement(HyleTablePanel, {
        list: makeReadyList(),
        filters: makeFilters({ filterClear }),
        children: createElement(HyleTableFilters),
      }),
      { wrapper: wrap(client) },
    );
    fireEvent.click(screen.getByText("Clear"));
    expect(filterClear).toHaveBeenCalled();
  });
});
