import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { createElement, type ReactNode } from "react";
import { HyleProvider } from "@tty-pt/hyle-react";
import type { HyleClient } from "@tty-pt/hyle";
import type { Blueprint, Field, Column, Result, Row, Manifest } from "@tty-pt/hyle-react";
import { HyleFormFields } from "../src/components/form";
import type { HyleFiltersState } from "@tty-pt/hyle-react";

// ── Mock client ───────────────────────────────────────────────────────────────

const bp: Blueprint = {
  models: {
    user: {
      fields: {
        name:  { label: "Name",  type: { kind: "primitive", primitive: "string"  } },
        email: { label: "Email", type: { kind: "primitive", primitive: "string"  } },
        active:{ label: "Active",type: { kind: "primitive", primitive: "boolean" } },
      },
    },
  },
};

function makeClient(overrides: Partial<HyleClient> = {}): HyleClient {
  const manifest: Manifest = { base: "user", fields: ["name", "email", "active"] };
  const result: Result = { rows: [], total: 0 };
  const cols: Column[] = [
    { key: "name",   label: "Name",   field: { label: "Name",   type: { kind: "primitive", primitive: "string"  } } },
    { key: "email",  label: "Email",  field: { label: "Email",  type: { kind: "primitive", primitive: "string"  } } },
    { key: "active", label: "Active", field: { label: "Active", type: { kind: "primitive", primitive: "boolean" } } },
  ];
  return {
    load:         vi.fn(() => Promise.resolve()),
    manifest:     vi.fn(() => manifest),
    resolve:      vi.fn(() => result),
    resolveQuery: vi.fn(() => ({ manifest, result, rows: [] })),
    resolveAndView: vi.fn(() => ({ outcome: result, rows: [], isSingle: false, columns: cols })),
    rowsArray:    vi.fn((r: Result) => (Array.isArray(r.rows) ? r.rows : [r.rows as Row])),
    makeField:    vi.fn(() => ({ label: "X", type: { kind: "primitive", primitive: "string" } } as Field)),
    columns:      vi.fn(() => cols),
    filterRows:   vi.fn((rows: Row[]) => rows),
    applyView:    vi.fn((rows: Row[]) => rows),
    displayValue: vi.fn(() => ""),
    filterLayout: vi.fn(() => [cols]),
    formaToQuery: vi.fn(() => ({ model: "user" })),
    purifyRow:    vi.fn(() => null),
    isSingle:     vi.fn(() => false),
    // modelFields returns the fields array from the blueprint model
    ...overrides,
  };
}

function wrap(client: HyleClient) {
  return ({ children }: { children: ReactNode }) =>
    createElement(HyleProvider, { client, children });
}

function makeFilter(keys: string[]): HyleFiltersState["Filter"] {
  return Object.fromEntries(
    keys.map((k) => [
      k,
      () => createElement("input", { "data-testid": `input-${k}`, placeholder: k }),
    ]),
  );
}

// ── Tests ─────────────────────────────────────────────────────────────────────

describe("HyleFormFields", () => {
  it("renders a labeled input for each model field that has a Filter", () => {
    const client = makeClient();
    render(
      createElement(HyleFormFields, {
        blueprint: bp,
        model: "user",
        Filter: makeFilter(["name", "email", "active"]),
      }),
      { wrapper: wrap(client) },
    );

    expect(screen.getByText("Name")).toBeTruthy();
    expect(screen.getByText("Email")).toBeTruthy();
    // boolean fields skip the outer label — verified by testid below
    expect(screen.getByTestId("input-name")).toBeTruthy();
    expect(screen.getByTestId("input-email")).toBeTruthy();
    expect(screen.getByTestId("input-active")).toBeTruthy();
  });

  it("skips fields that have no Filter component", () => {
    const client = makeClient();
    render(
      createElement(HyleFormFields, {
        blueprint: bp,
        model: "user",
        // only name and email supplied; active omitted
        Filter: makeFilter(["name", "email"]),
      }),
      { wrapper: wrap(client) },
    );

    expect(screen.getByTestId("input-name")).toBeTruthy();
    expect(screen.getByTestId("input-email")).toBeTruthy();
    expect(screen.queryByTestId("input-active")).toBeNull();
  });

  it("renders nothing when Filter map is empty", () => {
    const client = makeClient();
    const { container } = render(
      createElement(HyleFormFields, {
        blueprint: bp,
        model: "user",
        Filter: {},
      }),
      { wrapper: wrap(client) },
    );

    // wrapper div exists but contains no field rows
    expect(container.querySelectorAll(".hyle-field-row").length).toBe(0);
  });

  it("uses the label from the blueprint, not the key", () => {
    const client = makeClient();
    render(
      createElement(HyleFormFields, {
        blueprint: bp,
        model: "user",
        Filter: makeFilter(["name"]),
      }),
      { wrapper: wrap(client) },
    );

    // Label text comes from blueprint field label ("Name"), not key ("name")
    expect(screen.getByText("Name")).toBeTruthy();
    expect(screen.queryByText("name")).toBeNull();
  });
});
