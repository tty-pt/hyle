#ifndef HYLE_BUD_H
#define HYLE_BUD_H

#include <bud/bud.h>
#include <bud/bud_jsx.h>

#define HYLE_BUD_STRING 0
#define HYLE_BUD_INT 1
#define HYLE_BUD_BOOL 2
#define HYLE_BUD_NULLABLE_STRING 3
#define HYLE_BUD_REFERENCE 4
#define HYLE_BUD_MULTI_REFERENCE 5

typedef struct {
	const char *id;
	const char *label;
} hyle_bud_option_t;

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

/* Paginated omnisearch picker for item-form REFERENCE and
 * MULTI_REFERENCE fields. SSR-first: one builder draws the widget for
 * the SSR page, the no-JS round-trip re-render, and both fragment
 * shapes (reset slots / append rows). Options/selections are borrowed,
 * valid through render only; no state is retained across events.
 * url_tmpl is the fragment URL template with literal {q} / {page} /
 * {sel} slots substituted by the transport (htdocs/hyle-fragments.js). */
typedef struct {
	const char *key;         /* form field name */
	const char *label;       /* human label */
	const char *source;      /* target dataset id, e.g. "grp.items" */
	int multi;               /* 0 = radio (single), 1 = checkbox (multi) */
	const char *get_form_id; /* sibling GET form id */
	const char *url_tmpl;    /* fragment URL with {q},{page},{sel} slots */
	const hyle_bud_option_t *page_opts; int npage; /* current page (borrowed) */
	const hyle_bud_option_t *sel;            int nsel; /* pinned selections */
	const char *q;
	int page, per_page, total;
	const char *search_param;/* custom search input name; NULL defaults to "pick_q_<key>" */
	const char *page_param;  /* custom page input name; NULL defaults to "pick_page_<key>" */
} hyle_bud_picker_desc_t;

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

#define HYLE_BUD_PICKER_MAX_OPTS 64
#define HYLE_BUD_PICKER_MAX_SEL 64
#define HYLE_BUD_PICKER_MAX_FIELDS 8

typedef struct {
	const char *key;
	const char *label;
	const char *target;
	int multi;
	const char *search_param;
	const char *page_param;
	const hyle_bud_option_t *page_opts; int npage;
	const hyle_bud_option_t *sel;       int nsel;
	const char *q;
	int page, per_page, total;
} hyle_bud_picker_entry_t;

typedef struct {
	int n;
	hyle_bud_picker_entry_t entries[HYLE_BUD_PICKER_MAX_FIELDS];
} hyle_bud_picker_view_t;

typedef struct {
	hyle_bud_option_t opts[HYLE_BUD_PICKER_MAX_OPTS];
	hyle_bud_option_t sel[HYLE_BUD_PICKER_MAX_OPTS];
	char opt_ids[HYLE_BUD_PICKER_MAX_OPTS][128];
	char opt_labels[HYLE_BUD_PICKER_MAX_OPTS][256];
	char sel_ids[HYLE_BUD_PICKER_MAX_OPTS][128];
	char sel_labels[HYLE_BUD_PICKER_MAX_OPTS][256];
} hyle_bud_picker_buffer_t;

struct json_object;

void hyle_bud_picker_state_from_json(
        const char *json, size_t jlen, const char *key, const char *target,
        int multi, const char *q, int page,
        hyle_bud_picker_buffer_t *buf, hyle_bud_picker_view_t *pv_out);

void hyle_bud_picker_state_to_json(
        const hyle_bud_picker_view_t *pv, struct json_object *j_root);

#endif
