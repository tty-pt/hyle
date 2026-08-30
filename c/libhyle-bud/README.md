# libhyle-bud — Hyle to Bud UI Bridge

The dedicated bridge connecting Hyle canonical data schemas (`hyle_schema_desc_t`) to Bud HTML DOM and WASM components.

## Overview

`libhyle-bud` is the **only** library permitted to link both `hyle` and `bud`. It enables declarative UI rendering directly from data schemas without hardcoding form fields or filter bars in application code:
- **Universal Schema Filter (`hyle_bud_filter`):** Automatically renders text search fields, boolean switches, single-reference dropdowns, or multi-reference facet pickers based on schema field definitions.
- **Declarative Form Builder (`hyle_bud_form`):** Builds complete HTML forms with labels, inputs, validation attributes, and CSRF tokens.
- **State Unpacking (`hyle_bud_state_apply_len`):** Hydrates WASM state structs from server-rendered `bud-state` JSON using zero-copy stride layouts.
- **Omni-Dropdown Pickers:** Generates No-JS fallback forms and hot-swappable fragment slots for live client-side search.

## Key APIs (`hyle-bud/hyle-bud.h`)

```c
/* Unpack bud-state JSON into state struct in WASM */
void hyle_bud_state_apply_len(void *state, const hyle_schema_desc_t *fields,
                              const char *json, size_t len);

/* Universal schema-driven filter component */
bud_node *hyle_bud_filter(const hyle_schema_desc_t *desc,
                          const char *field_name,
                          const char *current_value,
                          const hyle_bud_picker_view_t *pv);

/* Declarative form builder from schema */
bud_node *hyle_bud_form(const hyle_schema_desc_t *schema,
                        const void *record,
                        const char *action,
                        const char *cancel_href,
                        const char *submit_label,
                        const char *csrf_token,
                        const hyle_bud_picker_view_t *pv,
                        const char *vstr_val);
```

## Dependencies

- `external/hyle` — Canonical data schemas
- `external/bud` — HTML AST and WASM bridge
- `external/libqmap` — Option resolution
