import { createElement, type ReactNode } from "react";
import { renderHook, act } from "@testing-library/react";
import { vi } from "vitest";
import type {
  Blueprint,
  Column,
  Field,
  Forma,
  HyleClient,
  Manifest,
  Query,
  Result,
  Row,
  Source,
} from "@tty-pt/hyle";
import { HyleProvider } from "../src/index";

// ─── Fixture factories ────────────────────────────────────────────────────────

export function makeBlueprint(extra: Partial<Blueprint> = {}): Blueprint {
  return {
    models: {
      user: {
        fields: {
          name: { label: "Name", type: { kind: "primitive", primitive: "string" } },
          role: { label: "Role", type: { kind: "primitive", primitive: "string" } },
          active: { label: "Active", type: { kind: "primitive", primitive: "boolean" } },
        },
      },
    },
    ...extra,
  };
}

export function makeQuery(extra: Partial<Query> = {}): Query {
  return { model: "user", select: ["name", "role"], ...extra };
}

export function makeManifest(extra: Partial<Manifest> = {}): Manifest {
  return { base: "user", fields: ["name", "role"], ...extra };
}

export function makeRow(extra: Row = {}): Row {
  return { id: "1", name: "Alice", role: "admin", ...extra };
}

export function makeResult(rows: Row | Row[] = [], total = 1): Result {
  return { rows, total };
}

export function makeSource(rows: Row[] = [makeRow()]): Source {
  return { user: { result: rows, total: rows.length } };
}

export function makeColumn(key: string, label: string): Column {
  const field: Field = { label, type: { kind: "primitive", primitive: "string" } };
  return { key, field, label };
}

// ─── Mock client factory ──────────────────────────────────────────────────────

export type MockClient = HyleClient & {
  _resolveLoad: () => void;
  _rejectLoad: (err: unknown) => void;
};

export function makeMockClient(overrides: Partial<HyleClient> = {}): MockClient {
  let resolveLoad!: () => void;
  let rejectLoad!: (err: unknown) => void;
  const loadPromise = new Promise<void>((res, rej) => {
    resolveLoad = res;
    rejectLoad = rej;
  });

  const defaultManifest = makeManifest();
  const defaultResult = makeResult([makeRow()]);
  const defaultColumns: Column[] = [makeColumn("name", "Name"), makeColumn("role", "Role")];

  const client: MockClient = {
    load: vi.fn(() => loadPromise),
    manifest: vi.fn(() => defaultManifest),
    resolve: vi.fn(() => defaultResult),
    resolveAndView: vi.fn(() => ({
      outcome: defaultResult,
      rows: Array.isArray(defaultResult.rows) ? defaultResult.rows : [defaultResult.rows as Row],
      isSingle: false,
      columns: defaultColumns,
    })),
    resolveQuery: vi.fn(() => ({
      manifest: defaultManifest,
      result: defaultResult,
      rows: [makeRow()],
    })),
    rowsArray: vi.fn((result: Result) => (Array.isArray(result.rows) ? result.rows : [result.rows])),
    makeField: vi.fn(() => ({ label: "X", type: { kind: "primitive", primitive: "string" } } as Field)),
    columns: vi.fn(() => defaultColumns),
    filterRows: vi.fn((rows: Row[]) => rows),
    applyView: vi.fn((rows: Row[]) => rows),
    displayValue: vi.fn(() => "value"),
    filterLayout: vi.fn(() => [defaultColumns]),
    formaToQuery: vi.fn(() => makeQuery()),
    purifyRow: vi.fn(() => null),
    isSingle: vi.fn(() => false),
    _resolveLoad: resolveLoad,
    _rejectLoad: rejectLoad,
    ...overrides,
  };

  return client;
}

// ─── renderWithHyle helper ────────────────────────────────────────────────────

export function renderWithHyle<T>(
  hook: () => T,
  client: HyleClient,
) {
  const wrapper = ({ children }: { children: ReactNode }) =>
    createElement(HyleProvider, { client, children });

  return renderHook(hook, { wrapper });
}

export { act };
