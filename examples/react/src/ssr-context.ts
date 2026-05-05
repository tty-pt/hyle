import { createContext, useContext } from "react";
import type { Row } from "@tty-pt/hyle-react";

export type SsrLookups = Record<string, Record<string, Row>>;

export type SsrListData = {
  rows: Row[];
  total: number;
  lookups: SsrLookups;
  page: number;
  perPage: number;
  sortField: string;
  sortAscending: boolean;
  /** Active filter values from the URL query string — used to seed useFilters on hydration */
  filters?: Record<string, string | string[]>;
};

export type SsrFormData = {
  values: Record<string, unknown>;
  errors?: Array<{ field: string; message: string }>;
};

export type SsrData = {
  list?: SsrListData;
  form?: SsrFormData;
};

export const SsrDataContext = createContext<SsrData | null>(null);

export function useSsrData(): SsrData | null {
  return useContext(SsrDataContext);
}
