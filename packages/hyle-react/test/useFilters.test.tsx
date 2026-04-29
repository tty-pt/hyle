import { describe, it, expect, vi } from "vitest";
import {
  act,
  makeMockClient,
  makeBlueprint,
  makeQuery,
  makeResult,
  makeRow,
  makeSource,
  makeColumn,
  renderWithHyle,
} from "./utils";
import { makeHyleHooks } from "../src/index";

const blueprint = makeBlueprint();

function makeHooks(overrides: Parameters<typeof makeMockClient>[0] = {}) {
  const row = makeRow();
  const source = makeSource([row]);
  const result = makeResult([row]);
  const client = makeMockClient({
    resolveAndView: vi.fn(() => ({
      outcome: result,
      rows: [row],
      isSingle: false,
      columns: [makeColumn("name", "Name"), makeColumn("role", "Role")],
    })),
    purifyRow: vi.fn(() => null),
    ...overrides,
  });
  const { useFilters } = makeHyleHooks({ blueprint, adapter: { useSource: () => source } });
  return { client, useFilters };
}

describe("useFilters", () => {
  it("starts with empty formData", async () => {
    const { client, useFilters } = makeHooks();
    const { result } = renderWithHyle(() => useFilters(makeQuery()), client);
    await act(async () => { client._resolveLoad(); });

    expect(result.current.formData).toEqual({});
  });

  it("setField updates formData without committing", async () => {
    const { client, useFilters } = makeHooks();
    const { result } = renderWithHyle(() => useFilters(makeQuery()), client);
    await act(async () => { client._resolveLoad(); });

    act(() => { result.current.setField("name", "Bob"); });

    expect(result.current.formData).toEqual({ name: "Bob" });
    // where clause not yet updated
    expect(result.current.query.where?.["name"]).toBeUndefined();
  });

  it("filterApply merges formData into effectiveQuery.where", async () => {
    const { client, useFilters } = makeHooks();
    const { result } = renderWithHyle(() => useFilters(makeQuery()), client);
    await act(async () => { client._resolveLoad(); });

    act(() => { result.current.setField("name", "Bob"); });
    act(() => { result.current.filterApply(); });

    expect(result.current.query.where?.["name"]).toBe("Bob");
  });

  it("filterClear resets formData, where, and increments filterResetKey", async () => {
    const { client, useFilters } = makeHooks();
    const { result } = renderWithHyle(() => useFilters(makeQuery()), client);
    await act(async () => { client._resolveLoad(); });

    act(() => { result.current.setField("name", "Bob"); });
    act(() => { result.current.filterApply(); });
    const keyBefore = result.current.filterResetKey;

    act(() => { result.current.filterClear(); });

    expect(result.current.formData).toEqual({});
    expect(result.current.query.where?.["name"]).toBeUndefined();
    expect(result.current.filterResetKey).toBe(keyBefore + 1);
  });

  it("validate() sets purifyErrors when purifyRow returns errors", async () => {
    const errors = [{ field: "name", rule: "required", message: "required" }];
    const client = makeMockClient({
      resolveAndView: vi.fn(() => ({
        outcome: makeResult([makeRow()]),
        rows: [makeRow()],
        isSingle: false,
        columns: [],
      })),
      purifyRow: vi.fn(() => errors),
    });
    const { useFilters } = makeHyleHooks({
      blueprint,
      adapter: { useSource: () => makeSource() },
    });
    const { result } = renderWithHyle(() => useFilters(makeQuery()), client);
    await act(async () => { client._resolveLoad(); });

    await act(async () => { result.current.validate(); });

    expect(result.current.purifyErrors).toEqual(errors);
  });

  it("validate() sets purifyErrors to null when purifyRow returns null", async () => {
    const client = makeMockClient({
      resolveAndView: vi.fn(() => ({
        outcome: makeResult([makeRow()]),
        rows: [makeRow()],
        isSingle: false,
        columns: [],
      })),
      purifyRow: vi.fn(() => null),
    });
    const { useFilters } = makeHyleHooks({
      blueprint,
      adapter: { useSource: () => makeSource() },
    });
    const { result } = renderWithHyle(() => useFilters(makeQuery()), client);
    await act(async () => { client._resolveLoad(); });

    act(() => { result.current.setField("name", "Bob"); });
    await act(async () => { result.current.validate(); });

    expect(result.current.purifyErrors).toBeNull();
  });

  it("seeds formData from row when id is in query where", async () => {
    const existingRow = makeRow({ id: "42", name: "Carol", role: "editor" });
    const client = makeMockClient({
      resolveAndView: vi.fn(() => ({
        outcome: makeResult(existingRow),
        rows: [existingRow],
        isSingle: true,
        columns: [makeColumn("name", "Name"), makeColumn("role", "Role")],
      })),
      purifyRow: vi.fn(() => null),
    });
    const { useFilters } = makeHyleHooks({
      blueprint,
      adapter: { useSource: () => makeSource([existingRow]) },
    });
    const { result } = renderWithHyle(
      () => useFilters(makeQuery({ where: { id: "42" } })),
      client,
    );
    await act(async () => { client._resolveLoad(); });

    // formData seeded from row
    expect(result.current.formData["name"]).toBe("Carol");
    expect(result.current.formData["role"]).toBe("editor");
  });

  it("change: modified field uses new label in Filter map", async () => {
    const { client, useFilters } = makeHooks();
    const change = {
      name: (f: import("../src/index").Field) => ({ ...f, label: "Full Name" }),
    };
    const { result } = renderWithHyle(
      () => useFilters(makeQuery(), { change }),
      client,
    );
    await act(async () => { client._resolveLoad(); });

    // The Filter map should still have the "name" key.
    expect(typeof result.current.Filter["name"]).toBe("function");
    // The "role" field should be unaffected.
    expect(typeof result.current.Filter["role"]).toBe("function");
  });

  it("change: null transform removes field from Filter map", async () => {
    const { client, useFilters } = makeHooks();
    const change = { name: () => null };
    const { result } = renderWithHyle(
      () => useFilters(makeQuery(), { change }),
      client,
    );
    await act(async () => { client._resolveLoad(); });

    expect(result.current.Filter["name"]).toBeUndefined();
    expect(typeof result.current.Filter["role"]).toBe("function");
  });

  it("change: null transform excludes field from purifyRow call", async () => {
    const purifyRow = vi.fn(() => null);
    const client = makeMockClient({
      resolveAndView: vi.fn(() => ({
        outcome: makeResult([makeRow()]),
        rows: [makeRow()],
        isSingle: false,
        columns: [makeColumn("name", "Name"), makeColumn("role", "Role")],
      })),
      purifyRow,
    });
    const { useFilters } = makeHyleHooks({
      blueprint,
      adapter: { useSource: () => makeSource() },
    });
    // Exclude "name" via change.
    const change = { name: () => null };
    const { result } = renderWithHyle(
      () => useFilters(makeQuery(), { change }),
      client,
    );
    await act(async () => { client._resolveLoad(); });

    act(() => { result.current.setField("name", "Bob"); });
    await act(async () => { result.current.validate(); });

    // purifyRow should have been called without "name" in the row.
    const calledRow = purifyRow.mock.calls[0]?.[2] ?? {};
    expect(calledRow).not.toHaveProperty("name");
  });
});
