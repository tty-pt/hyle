import { describe, it, expect, vi } from "vitest";
import {
  act,
  makeMockClient,
  makeBlueprint,
  makeQuery,
  makeResult,
  makeRow,
  makeSource,
  renderWithHyle,
} from "./utils";
import { makeHyleHooks } from "../src/index";
import type { Forma, Query } from "../src/index";

const blueprint = makeBlueprint();

const sampleForma: Forma = {
  fields: [
    { name: "name", label: "Name", fieldType: "string" },
    { name: "role", label: "Role", fieldType: "string" },
  ],
  column: ["name", "role"],
};

const formaRow = { ...sampleForma } as Record<string, unknown>;

function makeHooks(overrides: Parameters<typeof makeMockClient>[0] = {}) {
  // The "forma" entity is always queried with method "one"
  const client = makeMockClient({
    resolveAndView: vi.fn(() => ({
      outcome: makeResult(formaRow),
      rows: [formaRow],
      isSingle: true,
      columns: [],
    })),
    formaToQuery: vi.fn(() => makeQuery({ model: "user" })),
    ...overrides,
  });
  const { useForma } = makeHyleHooks({ blueprint, adapter: { useSource: () => makeSource([formaRow]) } });
  return { client, useForma };
}

describe("useForma", () => {
  it("returns [null, null] while loading", () => {
    const { client, useForma } = makeHooks();
    const { result } = renderWithHyle(() => useForma("user"), client);

    expect(result.current[0]).toBeNull();
    expect(result.current[1]).toBeNull();
  });

  it("returns [derivedQuery, forma] when ready", async () => {
    const derivedQuery: Query = makeQuery({ model: "user", select: ["name"] });
    const { client, useForma } = makeHooks({
      formaToQuery: vi.fn(() => derivedQuery),
    });
    const { result } = renderWithHyle(() => useForma("user"), client);

    await act(async () => { client._resolveLoad(); });

    expect(result.current[1]).toMatchObject({ fields: sampleForma.fields });
    expect(result.current[0]).toEqual(derivedQuery);
  });

  it("returns [null, forma] when forma has no fields", async () => {
    const emptyFormaRow = { fields: [] } as Record<string, unknown>;
    const client = makeMockClient({
      resolveAndView: vi.fn(() => ({
        outcome: makeResult(emptyFormaRow),
        rows: [emptyFormaRow],
        isSingle: true,
        columns: [],
      })),
      formaToQuery: vi.fn(() => makeQuery()),
    });
    const { useForma } = makeHyleHooks({
      blueprint,
      adapter: { useSource: () => makeSource([emptyFormaRow]) },
    });
    const { result } = renderWithHyle(() => useForma("user"), client);

    await act(async () => { client._resolveLoad(); });

    expect(result.current[0]).toBeNull();
    expect(result.current[1]).toMatchObject({ fields: [] });
  });

  it("returns [null, forma] when formaToQuery throws", async () => {
    const { client, useForma } = makeHooks({
      formaToQuery: vi.fn(() => { throw new Error("bad forma"); }),
    });
    const { result } = renderWithHyle(() => useForma("user"), client);

    await act(async () => { client._resolveLoad(); });

    expect(result.current[0]).toBeNull();
    expect(result.current[1]).toMatchObject({ fields: sampleForma.fields });
  });

  it("passes tableName, context, and id to formaToQuery", async () => {
    const { client, useForma } = makeHooks();
    renderWithHyle(() => useForma("user", "42", { context: "form" }), client);

    await act(async () => { client._resolveLoad(); });

    expect(client.formaToQuery).toHaveBeenCalledWith(
      expect.objectContaining({ fields: sampleForma.fields }),
      "user",
      "form",
      "42",
    );
  });
});
