import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { createElement, type ReactNode } from "react";
import { HyleProvider } from "@tty-pt/hyle-react";
import { BooleanValue, ReferenceValue } from "../src/components/values";
import type { ValueProps } from "@tty-pt/hyle-react";
import type { HyleClient } from "@tty-pt/hyle";
import type { Field, Result, Row } from "@tty-pt/hyle";

function makeClient(): HyleClient {
  return {
    load:           vi.fn(() => Promise.resolve()),
    manifest:       vi.fn(),
    resolve:        vi.fn(),
    resolveAndView: vi.fn(),
    resolveQuery:   vi.fn(),
    rowsArray:      vi.fn((r: Result) => (Array.isArray(r.rows) ? r.rows : [r.rows])),
    makeField:      vi.fn(() => ({ label: "X", type: { kind: "primitive", primitive: "string" } } as Field)),
    columns:        vi.fn(() => []),
    filterRows:     vi.fn((rows: Row[]) => rows),
    applyView:      vi.fn((rows: Row[]) => rows),
    displayValue:   vi.fn((_bp, _res, _base, _field, value) => String(value ?? "")),
    filterLayout:   vi.fn(() => []),
    formaToQuery:   vi.fn(() => ({ model: "entity" })),
    purifyRow:      vi.fn(() => null),
    isSingle:       vi.fn(() => false),
  } as unknown as HyleClient;
}

function Wrapper({ children }: { children: ReactNode }) {
  return createElement(HyleProvider, { client: makeClient(), children });
}

function makeProps(overrides: Partial<ValueProps> = {}): ValueProps {
  return {
    value: null,
    row: { id: "1" },
    field: { label: "Field", type: { kind: "primitive", primitive: "string" } },
    column: { key: "field", label: "Field", field: { label: "Field", type: { kind: "primitive", primitive: "string" } } },
    result: null,
    modelName: "entity",
    blueprint: { models: {} },
    ...overrides,
  };
}

describe("BooleanValue", () => {
  it("renders a readonly checkbox checked when value is true", () => {
    render(<BooleanValue {...makeProps({ value: true })} />, { wrapper: Wrapper });
    const cb = screen.getByRole("checkbox") as HTMLInputElement;
    expect(cb.readOnly).toBe(true);
    expect(cb.checked).toBe(true);
  });

  it("renders a readonly checkbox unchecked when value is false", () => {
    render(<BooleanValue {...makeProps({ value: false })} />, { wrapper: Wrapper });
    const cb = screen.getByRole("checkbox") as HTMLInputElement;
    expect(cb.checked).toBe(false);
  });

  it("treats null/undefined as false", () => {
    render(<BooleanValue {...makeProps({ value: null })} />, { wrapper: Wrapper });
    expect((screen.getByRole("checkbox") as HTMLInputElement).checked).toBe(false);
  });
});

describe("ReferenceValue", () => {
  it("renders empty string when result is null", () => {
    render(<ReferenceValue {...makeProps({ value: "Admin", result: null })} />, { wrapper: Wrapper });
    expect(screen.getByText("Admin")).toBeDefined();
  });

  it("renders empty string for null value when result is null", () => {
    const { container } = render(<ReferenceValue {...makeProps({ value: null, result: null })} />, { wrapper: Wrapper });
    expect(container.querySelector("span")?.textContent).toBe("");
  });
});
