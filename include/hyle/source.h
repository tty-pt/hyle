#ifndef HYLE_SOURCE_H
#define HYLE_SOURCE_H

#include <stddef.h>
#include <stdint.h>
#include "ctx.h"
#include "field.h"
#include "query.h"

/*
 * Register a source.
 *
 * Creates both the row map (row_hd) and the fields map (fields_hd) internally.
 * record_id: 0 → plain QM_STR value maps; non-zero → typed record qmaps
 *            created with qmap_record_type_id(record_id) + QM_RECORD flag.
 *
 * flags:     Extra qmap flags for the row_hd (e.g. QM_SORTED, QM_AINDEX).
 *
 * user:      Opaque pointer stored in the registry.  Retrieve later with
 *            hyle_source_get_user().  libhyle does not free it.
 *
 * Returns fields_hd on success, 0 on error.
 */
unsigned hyle_source_register(
	const char *source_id,
	const hyle_field_t *fields,
	size_t field_count,
	uint32_t record_id,
	unsigned flags,
	void *user);

/*
 * Add or update a row in a registered source.
 * Writes row_id → "" in row_hd, and row_id:name → value for each field
 * in fields_hd.  names/values are parallel arrays of count entries.
 * Returns 0 on success.
 */
int hyle_source_put(const char *source_id,
	const char *row_id,
	const char **names,
	const char **values,
	size_t count);

/*
 * Delete a row from a registered source.
 * Removes row_id from row_hd and all row_id:* entries from fields_hd.
 */
void hyle_source_del(const char *source_id, const char *row_id);

/*
 * Filter → sort → paginate over the source's live qmaps.
 * Handles multi-reference position pre-filtering for typed-record sources
 * before delegating to hyle_apply_view.
 * *total_out receives total matching rows before pagination.
 * Caller must call hyle_row_set_free() on *out when done.
 * Returns 0 on success.
 */
int hyle_source_query(const char *source_id,
	const hyle_query_t *query,
	hyle_row_set_t *out,
	size_t *total_out);

/* Free the row_hd handle inside a row_set (fields_hd is never owned). */
void hyle_row_set_free(hyle_row_set_t *rs);

/* ---- Registry accessors ------------------------------------------------- */

unsigned     hyle_source_get_row_hd(const char *source_id);
unsigned     hyle_source_get_fields_hd(const char *source_id);
void        *hyle_source_get_user(const char *source_id);
void         hyle_source_set_user(const char *source_id, void *user);
size_t       hyle_source_count(void);
const char  *hyle_source_id_at(size_t i);

/* ---- FFI helpers (Rust bridge) ----------------------------------------- */

typedef struct {
	const char  *id;
	const char **field_names;
	const char **field_values;
	size_t       field_count;
} hyle_source_row_t;

/*
 * Convert a row_set to a flat row array.
 * Caller must free with hyle_source_rows_free(*rows_out, *count_out).
 * Returns 0 on success.
 */
int hyle_row_set_to_rows(const hyle_row_set_t *rs,
	hyle_source_row_t **rows_out,
	size_t *count_out);

void hyle_source_rows_free(hyle_source_row_t *rows, size_t count);

#endif
