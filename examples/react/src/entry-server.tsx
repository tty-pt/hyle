import { renderToString } from "react-dom/server";
import { StaticRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { HyleProvider } from "@tty-pt/hyle-react";
import type { Blueprint, Column, HyleClient, Manifest, Query } from "@tty-pt/hyle-react";
import { SsrDataContext, type SsrData } from "./ssr-context";
import { blueprint } from "./demo-data";
import App from "./App";

// ── Pure-JS SSR hyle client ───────────────────────────────────────────────────
// No WASM: implements only `manifest` (plan) in JS so filter columns render.
// All other methods throw — they're never called during renderToString.

function collectLookups(bp: Blueprint, modelName: string): string[] {
  const model = bp.models[modelName];
  if (!model) return [];
  const result: string[] = [];
  for (const field of Object.values(model.fields)) {
    if (field.type.kind === "reference") {
      result.push(field.type.reference.entity);
    } else if (
      field.type.kind === "array" &&
      field.type.item.kind === "reference"
    ) {
      result.push(field.type.item.reference.entity);
    }
  }
  return [...new Set(result)];
}

function ssrManifest(bp: Blueprint, query: Query): Manifest {
  const model = bp.models[query.model];
  const fields = model ? Object.keys(model.fields) : [];
  const lookups = collectLookups(bp, query.model);
  return {
    base: query.model,
    fields,
    filter: (query.where as Record<string, unknown> | undefined) ?? {},
    filterFields: query.filters ?? [],
    ...(lookups.length > 0 ? { lookups } : {}),
    ...(query.page !== undefined ? { page: query.page } : {}),
    ...(query.perPage !== undefined ? { perPage: query.perPage } : {}),
    ...(query.sort !== undefined ? { sort: query.sort } : {}),
    ...(query.method !== undefined ? { method: query.method } : {}),
  };
}

function notAvailable(name: string): never {
  throw new Error(`${name} is not available during SSR`);
}

function ssrColumns(bp: Blueprint, manifest: Manifest): Column[] {
  const model = bp.models[manifest.base];
  if (!model) return [];
  const fieldKeys = manifest.fields.length > 0 ? manifest.fields : Object.keys(model.fields);
  return fieldKeys
    .filter((k) => k in model.fields)
    .map((k) => ({
      key: k,
      field: model.fields[k],
      label: model.fields[k].label ?? k,
    }));
}

const ssrClient: HyleClient = {
  load: () => Promise.resolve(),
  manifest: (bp, query) => ssrManifest(bp, query),
  resolve: () => notAvailable("resolve"),
  resolveAndView: () => notAvailable("resolveAndView"),
  resolveQuery: () => notAvailable("resolveQuery"),
  rowsArray: () => notAvailable("rowsArray"),
  makeField: () => notAvailable("makeField"),
  columns: (bp, manifest) => ssrColumns(bp, manifest),
  filterRows: (rows) => rows,
  displayValue: (_bp, _result, _model, _field, value) => String(value ?? ""),
  filterLayout: () => [],
  formaToQuery: () => notAvailable("formaToQuery"),
  purifyRow: () => null,
  applyView: (rows) => rows,
  isSingle: () => false,
};

// ── Server render entry point ─────────────────────────────────────────────────

export function render(url: string, ssrData: SsrData): string {
  // Fresh QueryClient per request so no state bleeds between renders.
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, enabled: false } },
  });

  return renderToString(
    <QueryClientProvider client={queryClient}>
      <StaticRouter location={url}>
        <HyleProvider client={ssrClient} blueprint={blueprint} initialStatus="ready">
          <SsrDataContext.Provider value={ssrData}>
            <App />
          </SsrDataContext.Provider>
        </HyleProvider>
      </StaticRouter>
    </QueryClientProvider>,
  );
}
