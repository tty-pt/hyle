#ifndef HYLE_SOURCE_H
#define HYLE_SOURCE_H

#include <stddef.h>
#include <stdint.h>
#include "ctx.h"
#include "field.h"
#include "query.h"

/* Ordered source flags for hyle_source_register_ordered */
#define HYLE_AUTO_RECORD  0x01  /* Auto-create qmap record from field metadata */

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
size_t       hyle_source_get_field_count(const char *source_id);
const char  *hyle_source_get_field_name(const char *source_id, size_t idx);
hyle_field_type_t hyle_source_get_field_type(const char *source_id, size_t idx);

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

/* ---- Ordered source (positional arrays with pluggable persistence) ------ */

/*
 * Persistence callbacks for ordered sources.
 * load_fn:  called on first access to a partition; should populate items
 *           via hyle_source_put() with keys "{partition_val}__{NNNN}".
 * save_fn:  called after every mutation; should persist the partition's items.
 * user:     opaque pointer (not freed by libhyle).
 */
typedef int (*hyle_persist_load_fn)(const char *source_id,
	const char *partition_val, unsigned fields_hd, void *user);
typedef int (*hyle_persist_save_fn)(const char *source_id,
	const char *partition_val, unsigned fields_hd, void *user);

/*
 * Register an ordered (partitioned positional array) source.
 *
 * partition_field:  field name that scopes items to a partition (e.g. "sb").
 *                   Items are stored with keys "{partition_val}__{NNNN}".
 * load_fn / save_fn:  custom persistence callbacks (may be NULL).
 * persist_user:       passed to load_fn/save_fn.
 *
 * Returns fields_hd on success, 0 on error.
 */
unsigned hyle_source_register_ordered(
	const char *source_id,
	const hyle_field_t *fields, size_t field_count,
	const char *partition_field,
	uint32_t record_id, unsigned flags,
	hyle_persist_load_fn load_fn,
	hyle_persist_save_fn save_fn,
	void *persist_user);

/* Number of items in partition partition_val. */
int hyle_source_ordered_count(const char *source_id,
	const char *partition_val);

/*
 * Get the key for item at position pos in partition partition_val.
 * Returns a pointer to a static buffer (valid until next ordered key_at call).
 * Returns NULL if the item doesn't exist.
 */
const char *hyle_source_ordered_key_at(const char *source_id,
	const char *partition_val, int pos);

/*
 * Append an item to the end of a partition.  Triggers save.
 * names/values are parallel arrays of count entries.
 * Returns 0 on success.
 */
int hyle_source_ordered_append(const char *source_id,
	const char *partition_val,
	const char **names, const char **values, size_t count);

/*
 * Insert an item at position pos.  Shifts items [pos..end] forward.
 * Triggers save.  Returns 0 on success.
 */
int hyle_source_ordered_insert_at(const char *source_id,
	const char *partition_val, int pos,
	const char **names, const char **values, size_t count);

/* Remove the item at position pos.  Shifts items [pos+1..end] backward.
 * Triggers save. */
void hyle_source_ordered_remove_at(const char *source_id,
	const char *partition_val, int pos);

/* Remove all items in the partition.  Triggers save. */
void hyle_source_ordered_clear(const char *source_id,
	const char *partition_val);

/*
 * Explicitly trigger the save callback for a partition.
 * Useful after modifying fields via hyle_source_put() on an ordered
 * source's keys.
 */
void hyle_source_ordered_save(const char *source_id,
	const char *partition_val);

/* ---- Derive field registry ---------------------------------------------- */
/*
 * Function signature for derived field providers.
 * Called during index rebuild to get the searchable value for a derived field.
 *
 * def:     Source definition (opaque, retrieved via hyle_source_get_user())
 * row_id:  Row identifier
 * field_name: Name of the derived field being queried
 *
 * Returns: Pointer to static/thread-local buffer (caller must not free),
 *          or NULL/empty string if no value.
 */
typedef const char *(*hyle_derive_fn_t)(
	const void *def,
	const char *row_id,
	const char *field_name,
	void *user);

/*
 * Register a derive provider for a source.
 * Called by modules during xy_install() after source registration.
 *
 * derive_key:  Key matching hyle_field_t.derive_key (e.g. "song.lyrics_from_data")
 * fn:          Provider function
 * user:        Opaque user data passed to fn
 */
int hyle_register_derive(const char *derive_key, hyle_derive_fn_t fn, void *user);

/*
 * Lookup and call a derive provider.
 * Called internally during index rebuild.
 */
const char *hyle_call_derive(const void *def, const char *derive_key, const char *row_id, const char *field_name, void *user);

#endif
