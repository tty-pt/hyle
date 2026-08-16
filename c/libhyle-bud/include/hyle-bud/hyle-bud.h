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

#endif
