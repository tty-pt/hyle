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
import type { HyleMutationFactory } from "../src/index";

const blueprint = makeBlueprint();

function makeMutationFactory(createMutate = vi.fn(), updateMutate = vi.fn()): HyleMutationFactory {
  return (_model: string) => ({
    useCreate: () => ({
      mutate: createMutate,
      reset: vi.fn(),
      isPending: false,
      isSuccess: false,
      error: null,
      data: undefined,
    }),
    useUpdate: () => ({
      mutate: updateMutate,
      reset: vi.fn(),
      isPending: false,
      isSuccess: false,
      error: null,
      data: undefined,
    }),
    useDelete: () => ({
      mutate: vi.fn(),
      reset: vi.fn(),
      isPending: false,
      isSuccess: false,
      error: null,
      data: undefined,
    }),
  });
}

function makeHooks(
  overrides: Parameters<typeof makeMockClient>[0] = {},
  createMutate = vi.fn(),
  updateMutate = vi.fn(),
) {
  const row = makeRow();
  const source = makeSource([row]);
  const client = makeMockClient({
    resolveAndView: vi.fn(() => ({
      outcome: makeResult([row]),
      rows: [row],
      isSingle: false,
      columns: [makeColumn("name", "Name"), makeColumn("role", "Role")],
    })),
    purifyRow: vi.fn(() => null),
    ...overrides,
  });
  const { useForm } = makeHyleHooks({
    blueprint,
    adapter: {
      useSource: () => source,
      makeMutation: makeMutationFactory(createMutate, updateMutate),
    },
  });
  return { client, useForm, createMutate, updateMutate };
}

describe("useForm", () => {
  it("isEdit is false when query has no id in where", async () => {
    const { client, useForm } = makeHooks();
    const { result } = renderWithHyle(() => useForm(makeQuery()), client);
    await act(async () => { client._resolveLoad(); });

    expect(result.current.isEdit).toBe(false);
  });

  it("isEdit is true when query has id in where", async () => {
    const { client, useForm } = makeHooks();
    const { result } = renderWithHyle(
      () => useForm(makeQuery({ where: { id: "42" } })),
      client,
    );
    await act(async () => { client._resolveLoad(); });

    expect(result.current.isEdit).toBe(true);
  });

  it("isValid is true initially (no purifyErrors)", async () => {
    const { client, useForm } = makeHooks();
    const { result } = renderWithHyle(() => useForm(makeQuery()), client);
    await act(async () => { client._resolveLoad(); });

    expect(result.current.isValid).toBe(true);
  });

  it("onSubmit calls create mutation when isEdit is false", async () => {
    const createMutate = vi.fn();
    const { client, useForm } = makeHooks({}, createMutate);
    const { result } = renderWithHyle(() => useForm(makeQuery()), client);
    await act(async () => { client._resolveLoad(); });

    await act(async () => { result.current.onSubmit(); });

    expect(createMutate).toHaveBeenCalled();
  });

  it("onSubmit calls update mutation when isEdit is true", async () => {
    const updateMutate = vi.fn();
    const { client, useForm } = makeHooks({}, vi.fn(), updateMutate);
    const { result } = renderWithHyle(
      () => useForm(makeQuery({ where: { id: "42" } })),
      client,
    );
    await act(async () => { client._resolveLoad(); });

    await act(async () => { result.current.onSubmit(); });

    expect(updateMutate).toHaveBeenCalled();
  });

  it("onSubmit blocks mutation and sets purifyErrors when validation fails", async () => {
    const errors = [{ field: "name", rule: "required", message: "required" }];
    const createMutate = vi.fn();
    const client = makeMockClient({
      resolveAndView: vi.fn(() => ({
        outcome: makeResult([makeRow()]),
        rows: [makeRow()],
        isSingle: false,
        columns: [makeColumn("name", "Name")],
      })),
      purifyRow: vi.fn(() => errors),
    });
    const { useForm } = makeHyleHooks({
      blueprint,
      adapter: {
        useSource: () => makeSource(),
        makeMutation: makeMutationFactory(createMutate),
      },
    });
    const { result } = renderWithHyle(() => useForm(makeQuery()), client);
    await act(async () => { client._resolveLoad(); });

    await act(async () => { result.current.onSubmit(); });

    expect(result.current.purifyErrors).toEqual(errors);
    expect(createMutate).not.toHaveBeenCalled();
  });

  it("isValid is false after validation returns errors", async () => {
    const errors = [{ field: "name", rule: "required", message: "required" }];
    const client = makeMockClient({
      resolveAndView: vi.fn(() => ({
        outcome: makeResult([makeRow()]),
        rows: [makeRow()],
        isSingle: false,
        columns: [makeColumn("name", "Name")],
      })),
      purifyRow: vi.fn(() => errors),
    });
    const { useForm } = makeHyleHooks({
      blueprint,
      adapter: { useSource: () => makeSource() },
    });
    const { result } = renderWithHyle(() => useForm(makeQuery()), client);
    await act(async () => { client._resolveLoad(); });

    await act(async () => { result.current.onSubmit(); });

    expect(result.current.isValid).toBe(false);
  });

  it("mutation is null when adapter has no makeMutation", async () => {
    const client = makeMockClient();
    const { useForm } = makeHyleHooks({
      blueprint,
      adapter: { useSource: () => makeSource() },
    });
    const { result } = renderWithHyle(() => useForm(makeQuery()), client);
    await act(async () => { client._resolveLoad(); });

    expect(result.current.mutation).toBeNull();
  });

  it("onSubmit prevents default on event objects", async () => {
    const { client, useForm } = makeHooks();
    const { result } = renderWithHyle(() => useForm(makeQuery()), client);
    await act(async () => { client._resolveLoad(); });

    const preventDefault = vi.fn();
    await act(async () => { result.current.onSubmit({ preventDefault }); });

    expect(preventDefault).toHaveBeenCalled();
  });
});
