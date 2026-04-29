import { createElement, type ReactNode } from "react";
import { vi } from "vitest";
import { HyleProvider } from "@tty-pt/hyle-react";
import type { HyleClient } from "@tty-pt/hyle";
import type {
  Blueprint,
  Column,
  Field,
  Manifest,
  Result,
  Row,
  Source,
} from "@tty-pt/hyle-react";

export function makeBlueprint(): Blueprint {
  return {
    models: {
      user: {
        label: "Users",
        fields: {
          name:   { label: "Name",   type: { kind: "primitive", primitive: "string"  } },
          active: { label: "Active", type: { kind: "primitive", primitive: "boolean" } },
        },
      },
    },
  };
}

export function makeManifest(): Manifest {
  return { base: "user", fields: ["name", "active"] };
}

export function makeColumn(key: string, label: string): Column {
  const field: Field = { label, type: { kind: "primitive", primitive: "string" } };
  return { key, field, label };
}

export function makeResult(rows: Row[] = []): Result {
  return { rows, total: rows.length };
}

export function makeSource(rows: Row[] = []): Source {
  return { user: { result: rows, total: rows.length } };
}

export function makeMockClient(overrides: Partial<HyleClient> = {}): HyleClient & { _resolve: () => void } {
  let resolve!: () => void;
  const loadPromise = new Promise<void>((res) => { resolve = res; });

  const defaultManifest = makeManifest();
  const defaultResult   = makeResult([{ id: "1", name: "Alice", active: true }]);
  const defaultColumns: Column[] = [makeColumn("name", "Name"), makeColumn("active", "Active")];

  return {
    load: vi.fn(() => loadPromise),
    manifest: vi.fn(() => defaultManifest),
    resolve: vi.fn(() => defaultResult),
    resolveAndView: vi.fn(() => ({
      outcome: defaultResult,
      rows: Array.isArray(defaultResult.rows) ? defaultResult.rows as Row[] : [defaultResult.rows as Row],
      isSingle: false,
      columns: defaultColumns,
    })),
    resolveQuery: vi.fn(() => ({ manifest: defaultManifest, result: defaultResult, rows: defaultResult.rows as Row[] })),
    rowsArray: vi.fn((r: Result) => (Array.isArray(r.rows) ? r.rows : [r.rows as Row])),
    makeField: vi.fn(() => ({ label: "X", type: { kind: "primitive", primitive: "string" } } as Field)),
    columns: vi.fn(() => defaultColumns),
    filterRows: vi.fn((rows: Row[]) => rows),
    applyView: vi.fn((rows: Row[]) => rows),
    displayValue: vi.fn((_bp, _res, _model, _field, value) => String(value ?? "")),
    filterLayout: vi.fn(() => [defaultColumns]),
    formaToQuery: vi.fn(() => ({ model: "user" })),
    purifyRow: vi.fn(() => null),
    isSingle: vi.fn(() => false),
    _resolve: resolve,
    ...overrides,
  } as HyleClient & { _resolve: () => void };
}

export function makeWrapper(client: HyleClient) {
  return function Wrapper({ children }: { children: ReactNode }) {
    return createElement(HyleProvider, { client, children });
  };
}
