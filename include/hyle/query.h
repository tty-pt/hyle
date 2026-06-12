#ifndef HYLE_QUERY_H
#define HYLE_QUERY_H

#include <stdbool.h>
#include <stdint.h>
#include <stddef.h>
#include "ctx.h"
#include "field.h"

typedef struct {
	const char *field;
	const char *value;
} hyle_field_filter_t;

typedef struct {
	const char *sort_field;
	bool sort_asc;
	uint32_t page;
	uint32_t per_page;
	const char *q;
	hyle_field_filter_t *filters;
	unsigned filter_count;
	const char **include;
	unsigned include_count;
} hyle_query_t;

typedef struct {
	unsigned row_hd;
	unsigned fields_hd;
} hyle_row_set_t;

/*
 * Parse URL-encoded query string into a hyle_query_t.
 * Modifies query_str in place (null-terminates tokens).
 * Caller must keep query_str alive while the query_t is in use.
 * Allocates ->filters and ->include arrays via malloc; caller
 * must call hyle_query_clear() to free them.
 */
int hyle_parse_query(char *query_str, hyle_query_t *out);

/* Free dynamically allocated members of a parsed query. */
void hyle_query_clear(hyle_query_t *q);

/*
 * Filter rows: emit matching row IDs to output->row_hd.
 * Edits: output->row_hd written, output->fields_hd set to input->fields_hd.
 * q != NULL     → full-text search across id + all string fields.
 * filters       → per-field match (AND).
 * fields/field_count → optional field schema; when provided,
 *   HYLE_FIELD_MULTI_REFERENCE fields use newline-boundary token matching
 *   instead of substring match.  Pass NULL/0 for pure substring behaviour.
 */
void hyle_filter_rows(hyle_ctx_t *ctx,
	const hyle_row_set_t *input,
	const char *q,
	const hyle_field_filter_t *filters,
	unsigned filter_count,
	const hyle_field_t *fields,
	size_t field_count,
	hyle_row_set_t *output);

/*
 * Sort rows by sort_field. Auto-detects numeric vs string.
 * sort_field == NULL → passthrough.
 * output->fields_hd set to input->fields_hd.
 */
void hyle_sort_rows(hyle_ctx_t *ctx,
	const hyle_row_set_t *input,
	const char *sort_field,
	bool sort_asc,
	hyle_row_set_t *output);

/*
 * Paginate: emit only the requested page.
 * page=0 or per_page=0 → passthrough (all rows).
 * *total_out receives total matching rows before pagination.
 */
void hyle_paginate(hyle_ctx_t *ctx,
	const hyle_row_set_t *input,
	uint32_t page,
	uint32_t per_page,
	hyle_row_set_t *output,
	uint32_t *total_out);

/*
 * Combined pipeline: filter → sort → paginate in one call.
 * Intermediate results use ctx->scratch.
 * fields/field_count forwarded to hyle_filter_rows (may be NULL/0).
 */
void hyle_apply_view(hyle_ctx_t *ctx,
	const hyle_row_set_t *input,
	const hyle_query_t *query,
	const hyle_field_t *fields,
	size_t field_count,
	hyle_row_set_t *output,
	uint32_t *total_out);

#endif
