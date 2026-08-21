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

#endif
