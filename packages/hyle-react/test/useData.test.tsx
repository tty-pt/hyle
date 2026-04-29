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
import type { HyleSourceAdapterResult, Manifest, Result, Row } from "../src/index";

const blueprint = makeBlueprint();

function makeHooks(
  useSource: (manifest: Manifest | null) => HyleSourceAdapterResult,
  overrides: Parameters<typeof makeMockClient>[0] = {},
) {
  const client = makeMockClient(overrides);
  const { useData } = makeHyleHooks({ blueprint, adapter: { useSource } });
  return { client, useData };
}

describe("useData", () => {
  it("is loading before WASM resolves", () => {
    const { client, useData } = makeHooks(() => null);
    const { result } = renderWithHyle(() => useData(makeQuery()), client);
    expect(result.current.status).toBe("loading");
    expect(result.current.rows).toEqual([]);
  });

  it("is loading while source is null after WASM ready", async () => {
    const { client, useData } = makeHooks(() => null);
    const { result } = renderWithHyle(() => useData(makeQuery()), client);

    await act(async () => { client._resolveLoad(); });

    expect(result.current.status).toBe("loading");
  });

  it("is ready with rows once source arrives", async () => {
    const row = makeRow();
    const source = makeSource([row]);
    const result_ = makeResult([row]);
    const { client, useData } = makeHooks(() => source, {
      resolveAndView: vi.fn(() => ({
        outcome: result_,
        rows: [row],
        isSingle: false,
        columns: [makeColumn("name", "Name")],
      })),
    });

    const { result } = renderWithHyle(() => useData(makeQuery()), client);
    await act(async () => { client._resolveLoad(); });

    expect(result.current.status).toBe("ready");
    expect(result.current.rows).toEqual([row]);
    expect(result.current.row).toBeNull();
  });

  it("sets row when isSingle is true", async () => {
    const row = makeRow();
    const source = makeSource([row]);
    const result_ = makeResult(row);
    const { client, useData } = makeHooks(() => source, {
      resolveAndView: vi.fn(() => ({
        outcome: result_,
        rows: [row],
        isSingle: true,
        columns: [makeColumn("name", "Name")],
      })),
    });

    const { result } = renderWithHyle(() => useData(makeQuery()), client);
    await act(async () => { client._resolveLoad(); });

    expect(result.current.status).toBe("ready");
    expect(result.current.row).toEqual(row);
  });

  it("is error when resolve() throws", async () => {
    const source = makeSource();
    const { client, useData } = makeHooks(() => source, {
      resolveAndView: vi.fn(() => { throw new Error("resolve failed"); }),
    });

    const { result } = renderWithHyle(() => useData(makeQuery()), client);
    await act(async () => { client._resolveLoad(); });

    expect(result.current.status).toBe("error");
    expect((result.current as { error: Error }).error.message).toBe("resolve failed");
  });

  it("is error when source adapter returns an error", async () => {
    const { client, useData } = makeHooks(() => ({
      source: null,
      loading: false,
      error: new Error("fetch failed"),
    }));

    const { result } = renderWithHyle(() => useData(makeQuery()), client);
    await act(async () => { client._resolveLoad(); });

    expect(result.current.status).toBe("error");
    expect((result.current as { error: Error }).error.message).toBe("fetch failed");
  });

  it("builds fields array for single-record usage", async () => {
    const row = makeRow({ name: "Alice" });
    const source = makeSource([row]);
    const result_: Result = makeResult(row);
    const { client, useData } = makeHooks(() => source, {
      resolveAndView: vi.fn(() => ({
        outcome: result_,
        rows: [row],
        isSingle: true,
        columns: [makeColumn("name", "Name")],
      })),
      displayValue: vi.fn(() => "Alice"),
    });

    const { result } = renderWithHyle(() => useData(makeQuery()), client);
    await act(async () => { client._resolveLoad(); });

    expect(result.current.fields).toHaveLength(1);
    expect(result.current.fields[0].key).toBe("name");
    expect(result.current.fields[0].render()).toBe("Alice");
  });

  it("calls the validate adapter and surfaces the result", async () => {
    const row = makeRow();
    const source = makeSource([row]);
    const result_: Result = makeResult([row]);
    const validate = vi.fn(() => ({ ok: true }));
    const client = makeMockClient({
      resolveAndView: vi.fn(() => ({
        outcome: result_,
        rows: [row],
        isSingle: false,
        columns: [],
      })),
    });
    const { useData } = makeHyleHooks({ blueprint, adapter: { useSource: () => source }, validate });

    const { result } = renderWithHyle(() => useData(makeQuery()), client);
    await act(async () => { client._resolveLoad(); });

    expect(validate).toHaveBeenCalled();
    expect((result.current as { validation: unknown }).validation).toEqual({ ok: true });
  });
});
