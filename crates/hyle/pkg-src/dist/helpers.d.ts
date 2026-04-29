import type { Field, TypedField } from "./types";
type ShapeOf<F extends Record<string, TypedField<unknown>>> = {
    [K in keyof F]: F[K] extends TypedField<infer T> ? T : never;
};
export declare const field: {
    string(label: string, options?: Field["options"]): TypedField<string>;
    number(label: string, options?: Field["options"]): TypedField<number>;
    boolean(label: string, options?: Field["options"]): TypedField<boolean>;
    file(label: string, options?: Field["options"]): TypedField<string>;
    ref(label: string, entity: string, options?: Field["options"]): TypedField<string>;
    array<T>(label: string, item: TypedField<T>, options?: Field["options"]): TypedField<T[]>;
    shape<F extends Record<string, TypedField<unknown>>>(label: string, fields: F, options?: Field["options"]): TypedField<ShapeOf<F>>;
};
export {};
//# sourceMappingURL=helpers.d.ts.map