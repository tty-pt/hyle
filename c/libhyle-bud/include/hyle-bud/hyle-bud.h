#ifndef HYLE_BUD_H
#define HYLE_BUD_H

#include <bud/bud.h>
#include <bud/bud_jsx.h>
#include <hyle/picker.h>
#include <hyle/schema.h>

void hyle_bud_state_apply(
        void *state, const hyle_schema_desc_t *fields, const char *json);
void hyle_bud_state_apply_len(
        void *state, const hyle_schema_desc_t *fields, const char *json,
        size_t len);

#define HYLE_BUD_STRING 0
#define HYLE_BUD_INT 1
#define HYLE_BUD_BOOL 2
#define HYLE_BUD_NULLABLE_STRING 3
#define HYLE_BUD_REFERENCE 4
#define HYLE_BUD_MULTI_REFERENCE 5
#define HYLE_BUD_DERIVED 99

typedef hyle_option_t hyle_bud_option_t;
typedef hyle_picker_desc_t hyle_bud_picker_desc_t;
typedef hyle_picker_entry_t hyle_bud_picker_entry_t;
typedef hyle_picker_view_t hyle_bud_picker_view_t;
typedef hyle_picker_buffer_t hyle_bud_picker_buffer_t;

#define HYLE_BUD_PICKER_MAX_OPTS HYLE_PICKER_MAX_OPTS
#define HYLE_BUD_PICKER_MAX_SEL HYLE_PICKER_MAX_SEL
#define HYLE_BUD_PICKER_MAX_FIELDS HYLE_PICKER_MAX_FIELDS

struct json_object;

void hyle_bud_picker_state_from_json(
        const char *json, size_t jlen, const char *key, const char *target,
        int multi, const char *q, int page,
        hyle_bud_picker_buffer_t *buf, hyle_bud_picker_view_t *pv_out);

void hyle_bud_picker_state_to_json(
        const hyle_bud_picker_view_t *pv, struct json_object *j_root);

bud_node *hyle_bud_filter_field(
	const char *key,
	const char *label,
	int type,
	const char *current_value,
	const hyle_bud_option_t *options,
	int noptions,
	const char *filter_style);

/* Multi-select dropdown widget (SSR-first, WASM-enhanced).
 * filter_style "dropdown" selects it for HYLE_BUD_MULTI_REFERENCE fields. */
void hyle_bud_ms_reset(void);
bud_node *hyle_bud_multiselect_field(
	const char *key,
	const char *label,
	const char *current_value,
	const hyle_bud_option_t *options,
	int noptions);

/* Dropdown single-select widget (SSR-first, WASM-enhanced).
 * filter_style "dropdown" selects it for HYLE_BUD_REFERENCE fields. */
bud_node *hyle_bud_reference_select_dropdown(
	const char *key,
	const char *label,
	const char *current_value,
	const hyle_bud_option_t *options,
	int noptions);

bud_node *hyle_bud_table_header(
	const char **col_keys,
	const char **col_labels,
	int ncols,
	const char *sort_field,
	int sort_asc,
	const char *qs);

bud_node *hyle_bud_table_body(
	const char **col_keys,
	const char **col_labels,
	int ncols,
	const char **ids,
	int nids,
	const char **values,
	const char *module);

bud_node *hyle_bud_table(
	const char **col_keys,
	const char **col_labels,
	int ncols,
	const char **ids,
	int nids,
	const char **values,
	const char *module,
	const char *sort_field,
	int sort_asc,
	const char *qs);

bud_node *hyle_bud_pagination(
	int page,
	int per_page,
	int total,
	int row_count,
	const char *qs);

/* Row-action descriptor for hyle_bud_table_actions: makes each row
 * uniformly activable via a stretched overlay element in the first cell.
 * LINK renders an <a> (href defaults to /{module}/{id}); SUBMIT renders
 * a <button type=submit> that posts to the form named by form_id via the
 * HTML5 form= attribute (no-JS friendly, works inside another enclosing
 * form). css_class lets an <a> look like a button. */
typedef enum {
	HYLE_ROW_ACTION_NONE = 0,
	HYLE_ROW_ACTION_LINK,
	HYLE_ROW_ACTION_SUBMIT
} hyle_row_action_kind_t;

typedef struct {
	int kind;                /* hyle_row_action_kind_t */
	const char *css_class;   /* extra class, e.g. "btn" */
	const char *label;       /* visible text; ""/NULL = pure overlay */
	const char *aria_base;   /* aria-label prefix, e.g. "Add"/"Open" */
	const char *href_base;   /* LINK: overrides "/{module}/" default */
	const char *form_id;     /* SUBMIT: target form's id attribute */
	const char *field_name;  /* SUBMIT: e.g. "song_id"; value = row id */
} hyle_bud_row_action_t;

bud_node *hyle_bud_table_actions(
	const char **col_keys,
	const char **col_labels,
	int ncols,
	const char **ids,
	int nids,
	const char **values,
	const char *module,
	const char *sort_field,
	int sort_asc,
	const char *qs,
	const hyle_bud_row_action_t *action);

/* Full widget (trigger + panel). Returns NULL for an unusable desc. */
bud_node *hyle_bud_picker_field(const hyle_bud_picker_desc_t *d);

/* Panel innards + summary span HTML for the fragment route's reset
 * path. Caller owns both buffers. */
void hyle_bud_picker_slots(const hyle_bud_picker_desc_t *d,
        char *panel, size_t panel_sz, char *values, size_t values_sz);

/* Option-row chunk only — the append path (infinite scroll). Renders
 * the desc's current page; caller owns the buffer. */
void hyle_bud_picker_rows(const hyle_bud_picker_desc_t *d,
        char *rows, size_t rows_sz);

/*
 * Helper to split query-string into key-value pairs without modifying input
 */
size_t hyle_bud_query_param(
        const char *qs, const char *key, char *out, size_t out_sz);

/*
 * Active picker scope discovery helper
 */
int hyle_bud_pick_find_active_scope(const char *qs, char *scope_buf, size_t scope_sz);

/*
 * C-Struct to JSON State Overlays for WASM Hydration
 */
int hyle_bud_state_overlay_from_desc(
        struct json_object *jo,
        const void *state,
        const bud_field_desc_t *fields,
        int int_kind,
        int str_kind);

struct json_object *hyle_bud_state_overlay_array(
        const void *items,
        int count,
        size_t elem_size,
        const bud_field_desc_t *fields,
        int int_kind,
        int str_kind);

#endif
