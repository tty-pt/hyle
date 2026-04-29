import { describe, it, expect, vi } from "vitest";
import {
  act,
  makeMockClient,
  makeBlueprint,
  makeQuery,
  makeManifest,
  renderWithHyle,
} from "./utils";
import { makeHyleHooks } from "../src/index";

const blueprint = makeBlueprint();
const { useManifest } = makeHyleHooks({
  blueprint,
  adapter: { useSource: () => null },
});

describe("useManifest", () => {
  it("is loading before WASM resolves", () => {
    const client = makeMockClient();
    const { result } = renderWithHyle(() => useManifest(makeQuery()), client);
    expect(result.current.status).toBe("loading");
    expect(result.current.manifest).toBeNull();
  });

  it("is ready with the manifest after load resolves", async () => {
    const manifest = makeManifest({ fields: ["name", "role"] });
    const client = makeMockClient({ manifest: vi.fn(() => manifest) });
    const { result } = renderWithHyle(() => useManifest(makeQuery()), client);

    await act(async () => { client._resolveLoad(); });

    expect(result.current.status).toBe("ready");
    expect(result.current.manifest).toEqual(manifest);
  });

  it("is error when client.load() rejects", async () => {
    const client = makeMockClient();
    const { result } = renderWithHyle(() => useManifest(makeQuery()), client);

    await act(async () => { client._rejectLoad(new Error("WASM load failed")); });

    expect(result.current.status).toBe("error");
    expect((result.current as { error: Error }).error.message).toBe("WASM load failed");
    expect(result.current.manifest).toBeNull();
  });

  it("is error when manifest() throws after load", async () => {
    const client = makeMockClient({
      manifest: vi.fn(() => { throw new Error("invalid query"); }),
    });
    const { result } = renderWithHyle(() => useManifest(makeQuery()), client);

    await act(async () => { client._resolveLoad(); });

    expect(result.current.status).toBe("error");
    expect((result.current as { error: Error }).error.message).toBe("invalid query");
  });

  it("calls manifest() with the blueprint and query", async () => {
    const client = makeMockClient();
    const query = makeQuery({ select: ["name"] });
    renderWithHyle(() => useManifest(query), client);

    await act(async () => { client._resolveLoad(); });

    expect(client.manifest).toHaveBeenCalledWith(blueprint, query);
  });
});
