export type FieldType = {
    kind: "primitive";
    primitive: "string" | "number" | "boolean" | "file";
} | {
    kind: "reference";
    reference: {
        entity: string;
        displayField: string;
    };
} | {
    kind: "array";
    item: FieldType;
} | {
    kind: "shape";
    fields: Record<string, ShapeField>;
};
export type ShapeField = {
    label: string;
    type: FieldType;
    options?: FieldOptions;
};
export type FieldOptions = {
    sort?: "string" | "numeric" | "date" | "none";
    input?: {
        kind: string;
        props?: Record<string, unknown>;
    };
    fixedValue?: unknown;
    rule?: string;
    metadata?: Record<string, unknown>;
};
export type Field = {
    label: string;
    type: FieldType;
    options?: FieldOptions;
};
/**
 * A `Field` tagged with a phantom TypeScript type `T` representing the
 * value type of this field (e.g. `string`, `boolean`, `number`).
 * At runtime this is identical to `Field` — the `__type` brand is never set.
 */
export type TypedField<T> = Field & {
    readonly __type?: T;
};
/** A `Model` whose fields carry TypeScript value types via `TypedField`. */
export type TypedModel<TFields extends Record<string, TypedField<unknown>>> = {
    label?: string;
    fields: TFields;
};
/** A `Blueprint` whose models carry TypeScript value types. Assignable to `Blueprint`. */
export type TypedBlueprint<TModels extends Record<string, TypedModel<Record<string, TypedField<unknown>>>>> = {
    models: TModels;
};
/**
 * Infer the row shape from a `TypedModel`.
 * Always includes `id?: unknown` since ids are server-assigned.
 */
export type RowOf<TModel> = TModel extends TypedModel<infer TFields> ? {
    [K in keyof TFields]: TFields[K] extends TypedField<infer T> ? T : never;
} & {
    id?: unknown;
} : Row;
export type Column = {
    key: string;
    field: Field;
    label: string;
};
export type Model = {
    label?: string;
    fields: Record<string, Field>;
};
export type Blueprint = {
    models: Record<string, Model>;
};
export type Query = {
    model: string;
    select?: string[];
    where?: Record<string, unknown>;
    filters?: string[][];
    page?: number;
    perPage?: number;
    sort?: Sort;
    method?: string;
};
export type Sort = {
    field: string;
    ascending: boolean;
};
export type Manifest = {
    base: string;
    id?: unknown;
    fields: string[];
    filter?: Record<string, unknown>;
    lookups?: string[];
    inlines?: string[];
    page?: number;
    perPage?: number;
    sort?: Sort;
    method?: string;
    /** 2-D filter layout carried from query.filters */
    filterFields?: string[][];
};
export type Row = Record<string, unknown>;
export type ModelResult = {
    result: Row | Row[];
    total: number;
};
export type Source = Record<string, ModelResult>;
export type Result = {
    rows: Row | Row[];
    total: number;
    lookups?: Record<string, Record<string, Row>>;
};
export type FormaFieldType = string | {
    array: FormaFieldType;
} | {
    shape: FormaField[];
};
export type FormaField = {
    name: string;
    label: string;
    fieldType?: FormaFieldType;
    detailType?: FormaFieldType;
    formType?: FormaFieldType;
    columnType?: FormaFieldType;
    fixedValue?: unknown;
    rule?: string;
};
export type Forma = {
    fields: FormaField[];
    detail?: string[];
    form?: string[];
    column?: string[];
    filters?: string[][];
};
export type FormaContext = "detail" | "form" | "column";
export type PurifyError = {
    field: string;
    rule: string;
    message: string;
};
//# sourceMappingURL=types.d.ts.map