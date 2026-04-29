import { describe, it, expect, vi } from "vitest";
import {
  act,
  makeMockClient,
  makeBlueprint,
  makeQuery,
  makeManifest,
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
      columns: [makeColumn("name", "Name")],
    })),
    ...overrides,
  });
  const { useList } = makeHyleHooks({ blueprint, adapter: { useSource: () => source } });
  return { client, useList };
}

describe("useList", () => {
  it("initialises page and perPage from the query", async () => {
    const { client, useList } = makeHooks();
    const query = makeQuery({ page: 3, perPage: 50 });
    const { result } = renderWithHyle(() => useList(query), client);
    await act(async () => { client._resolveLoad(); });

    expect(result.current.page).toBe(3);
    expect(result.current.perPage).toBe(50);
  });

  it("defaults page=1 perPage=5 when not in query", async () => {
    const { client, useList } = makeHooks();
    const { result } = renderWithHyle(() => useList(makeQuery()), client);
    await act(async () => { client._resolveLoad(); });

    expect(result.current.page).toBe(1);
    expect(result.current.perPage).toBe(5);
  });

  it("setPage updates effectiveQuery.page", async () => {
    const { client, useList } = makeHooks();
    const { result } = renderWithHyle(() => useList(makeQuery()), client);
    await act(async () => { client._resolveLoad(); });

    act(() => { result.current.setPage(5); });

    expect(result.current.page).toBe(5);
    expect(result.current.query.page).toBe(5);
  });

  it("setPerPage updates effectiveQuery.perPage", async () => {
    const { client, useList } = makeHooks();
    const { result } = renderWithHyle(() => useList(makeQuery()), client);
    await act(async () => { client._resolveLoad(); });

    act(() => { result.current.setPerPage(100); });

    expect(result.current.perPage).toBe(100);
    expect(result.current.query.perPage).toBe(100);
  });

  it("setSortField updates effectiveQuery.sort", async () => {
    const { client, useList } = makeHooks();
    const { result } = renderWithHyle(() => useList(makeQuery()), client);
    await act(async () => { client._resolveLoad(); });

    act(() => { result.current.setSortField("name"); });

    expect(result.current.sortField).toBe("name");
    expect(result.current.query.sort?.field).toBe("name");
  });

  it("setSortAscending updates sort direction", async () => {
    const { client, useList } = makeHooks();
    const { result } = renderWithHyle(() => useList(makeQuery()), client);
    await act(async () => { client._resolveLoad(); });

    act(() => {
      result.current.setSortField("name");
      result.current.setSortAscending(false);
    });

    expect(result.current.sortAscending).toBe(false);
    expect(result.current.query.sort?.ascending).toBe(false);
  });

  it("exposes data status from useData", async () => {
    const { client, useList } = makeHooks();
    const { result } = renderWithHyle(() => useList(makeQuery()), client);

    expect(result.current.status).toBe("loading");
    await act(async () => { client._resolveLoad(); });
    expect(result.current.status).toBe("ready");
  });
});
