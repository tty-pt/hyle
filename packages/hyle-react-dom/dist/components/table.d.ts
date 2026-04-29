import type { Blueprint, Row, HyleListState, HyleFiltersState, HyleFieldComponents } from "@tty-pt/hyle-react";
export declare function HyleTableBody({ blueprint: blueprintProp, list, filters, onRowClick, selectedId, components, }: {
    blueprint?: Blueprint;
    list: HyleListState;
    filters?: HyleFiltersState;
    onRowClick?: (row: Row) => void;
    selectedId?: unknown;
    components?: HyleFieldComponents;
}): import("react/jsx-runtime").JSX.Element;
export declare function HyleTableFilters({ filters }: {
    filters: HyleFiltersState;
}): import("react/jsx-runtime").JSX.Element;
export declare function HyleTablePagination({ list }: {
    list: HyleListState;
}): import("react/jsx-runtime").JSX.Element | null;
export declare function HyleTable({ blueprint, list, filters, onRowClick, selectedId, components, }: {
    blueprint?: Blueprint;
    list: HyleListState;
    filters?: HyleFiltersState;
    onRowClick?: (row: Row) => void;
    selectedId?: unknown;
    components?: HyleFieldComponents;
}): import("react/jsx-runtime").JSX.Element;
//# sourceMappingURL=table.d.ts.map