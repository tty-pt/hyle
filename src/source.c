#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <limits.h>
#include <ttypt/qmap.h>
#include "hyle/source.h"
#include "hyle/query.h"
#include "hyle/ctx.h"

/* ---- registry ---- */

#define HYLE_SOURCE_MAX 64

typedef struct {
	char               source_id[128];
	unsigned           row_hd;
	unsigned           fields_hd;
	const hyle_field_t *fields;
	size_t             field_count;
	uint32_t           record_id;
	int                row_owned;  /* 1 = libhyle created row_hd */
	void              *user;
} registry_entry_t;

static registry_entry_t registry[HYLE_SOURCE_MAX];
static size_t registry_count = 0;

static registry_entry_t *find_entry(const char *source_id)
{
	size_t i;

	for (i = 0; i < registry_count; i++) {
		if (strcmp(registry[i].source_id, source_id) == 0)
			return &registry[i];
	}
	return NULL;
}

static registry_entry_t *alloc_entry(const char *source_id)
{
	registry_entry_t *e;

	if (!source_id || !source_id[0])
		return NULL;
	if (registry_count >= HYLE_SOURCE_MAX) {
		fprintf(stderr, "hyle_source_register: registry full\n");
		return NULL;
	}
	e = find_entry(source_id);
	if (e)
		return e;
	e = &registry[registry_count++];
	memset(e, 0, sizeof(*e));
	strncpy(e->source_id, source_id, sizeof(e->source_id) - 1);
	return e;
}

/* ---- hyle_source_register ---- */

unsigned hyle_source_register(
	const char *source_id,
	const hyle_field_t *fields,
	size_t field_count,
	uint32_t record_id,
	unsigned flags,
	void *user)
{
	registry_entry_t *e;
	unsigned          flds_hd;
	uint32_t          val_type;

	e = alloc_entry(source_id);
	if (!e)
		return 0;

	/* Derive qmap value type from record_id */
	val_type = record_id ? qmap_record_type_id(record_id) : QM_STR;

	/* row_hd: always created and owned by libhyle */
	if (e->row_owned && e->row_hd) {
		qmap_close(e->row_hd);
		e->row_hd = 0;
	}
	e->row_hd = qmap_open(
		NULL, NULL, QM_STR, val_type, 0x3FF, flags);
	if (!e->row_hd)
		return 0;
	e->row_owned = 1;

	/* fields_hd: always created and owned by libhyle */
	if (e->fields_hd)
		qmap_close(e->fields_hd);

	if (record_id) {
		flds_hd = qmap_open(
			NULL, NULL, QM_STR, val_type, 0x3FF,
			QM_RECORD(record_id));
	} else {
		flds_hd = qmap_open(
			NULL, NULL, QM_STR, QM_STR, 0x3FF, 0);
	}
	if (!flds_hd) {
		qmap_close(e->row_hd);
		e->row_hd    = 0;
		e->row_owned = 0;
		return 0;
	}

	e->fields_hd   = flds_hd;
	e->fields      = fields;
	e->field_count = field_count;
	e->record_id   = record_id;
	e->user        = user;

	return flds_hd;
}

/* ---- hyle_source_put ---- */

int hyle_source_put(const char *source_id,
	const char *row_id,
	const char **names,
	const char **values,
	size_t count)
{
	registry_entry_t *e;
	size_t            i;
	char              key[1024];

	if (!source_id || !row_id)
		return -1;
	e = find_entry(source_id);
	if (!e) {
		fprintf(stderr, "hyle_source_put: source '%s' not found\n",
			source_id);
		return -1;
	}
	qmap_put(e->row_hd, row_id, "");
	for (i = 0; i < count; i++) {
		if (!names[i])
			continue;
		snprintf(key, sizeof(key), "%s:%s", row_id, names[i]);
		qmap_put(e->fields_hd, key, values[i] ? values[i] : "");
	}
	return 0;
}

/* ---- hyle_source_del ---- */

void hyle_source_del(const char *source_id, const char *row_id)
{
	registry_entry_t *e;

	if (!source_id || !row_id)
		return;
	e = find_entry(source_id);
	if (!e)
		return;
	qmap_del(e->row_hd, row_id);
	qmap_del(e->fields_hd, row_id);
}

/* ---- hyle_row_set_free ---- */

void hyle_row_set_free(hyle_row_set_t *rs)
{
	if (!rs)
		return;
	if (rs->row_hd)
		qmap_close(rs->row_hd);
	/* fields_hd always non-owned (points into registry) */
	rs->row_hd    = 0;
	rs->fields_hd = 0;
}

/* ---- multi-reference pre-filter ---------------------------------------- */
/*
 * For sources whose multi-reference field values are stored as newline-
 * separated position indices (integers) rather than slugs, resolve
 * slug → position and pre-filter the row set so that apply_view sees
 * plain row IDs without the reference filter.
 *
 * Returns a newly-opened pre-filtered row_hd (caller must close), or 0 if no
 * pre-filtering was needed (caller should use the original row_hd directly).
 * Zeroes out the filter value in local_filters for each handled filter.
 */
static unsigned prefilter_multi_ref(
	const registry_entry_t *e,
	hyle_field_filter_t *local_filters,
	unsigned filter_count)
{
	unsigned  fi;
	unsigned  pre_hd = 0;

	for (fi = 0; fi < filter_count; fi++) {
		const char        *fname = local_filters[fi].field;
		const char        *slug  = local_filters[fi].value;
		size_t             sj;
		registry_entry_t  *target;
		uint32_t           pos;
		char               pos_str[32];
		unsigned           new_hd;
		uint32_t           cur;
		const void        *k;
		const void        *v;

		if (!fname || !slug || !slug[0])
			continue;

		/* Find field in schema */
		for (sj = 0; sj < e->field_count; sj++) {
			if (strcmp(e->fields[sj].name, fname) == 0)
				break;
		}
		if (sj >= e->field_count)
			continue;
		if (e->fields[sj].type != HYLE_FIELD_MULTI_REFERENCE)
			continue;
		if (!e->fields[sj].target_source)
			continue;

		target = find_entry(e->fields[sj].target_source);
		if (!target || !target->fields_hd)
			continue;

		pos = qmap_pos(target->fields_hd, slug);
		if (pos != UINT32_MAX)
			snprintf(pos_str, sizeof(pos_str), "%u", pos);
		else
			snprintf(pos_str, sizeof(pos_str), "%s", slug);

		new_hd = qmap_open(NULL, NULL, QM_STR, QM_STR, 0xFF, 0);
		if (!new_hd)
			continue;

		cur = qmap_iter(
			pre_hd ? pre_hd : e->row_hd, NULL, 0);
		while (qmap_next(&k, &v, cur)) {
			const char *rid = (const char *)k;
			const char *fv  = qmap_field_get(
				e->fields_hd, rid, fname);
			const char *p;

			if (!fv)
				continue;
			p = fv;
			while (*p) {
				const char *nl  = strchr(p, '\n');
				size_t      len = nl
					? (size_t)(nl - p)
					: strlen(p);

				if (len == strlen(pos_str) &&
				    strncmp(p, pos_str, len) == 0)
				{
					qmap_put(new_hd, rid, "");
					break;
				}
				if (!nl)
					break;
				p = nl + 1;
			}
		}
		qmap_fin(cur);

		/* Replace previous pre_hd with the new one */
		if (pre_hd)
			qmap_close(pre_hd);
		pre_hd = new_hd;

		/* Clear filter so apply_view won't double-filter */
		local_filters[fi].value = NULL;
	}

	return pre_hd;
}

/* ---- hyle_source_query ---- */

int hyle_source_query(const char *source_id,
	const hyle_query_t *query,
	hyle_row_set_t *out,
	size_t *total_out)
{
	registry_entry_t     *e;
	hyle_row_set_t        input;
	hyle_ctx_t           *ctx;
	uint32_t              total32 = 0;
	hyle_field_filter_t  *local_filters = NULL;
	hyle_query_t          local_query;
	unsigned              pre_hd = 0;

	if (!source_id || !query || !out)
		return -1;
	e = find_entry(source_id);
	if (!e) {
		fprintf(stderr,
			"hyle_source_query: source '%s' not found\n",
			source_id);
		return -1;
	}
	ctx = hyle_ctx_new();
	if (!ctx)
		return -1;

	/* Build a mutable copy of the query for pre-filter mutation */
	local_query = *query;
	if (query->filter_count > 0) {
		local_filters = malloc(
			query->filter_count * sizeof(hyle_field_filter_t));
		if (!local_filters) {
			hyle_ctx_free(ctx);
			return -1;
		}
		memcpy(local_filters, query->filters,
			query->filter_count * sizeof(hyle_field_filter_t));
		local_query.filters = local_filters;
	}

	/* Multi-reference pre-filter for typed-record sources */
	pre_hd = prefilter_multi_ref(e, local_query.filters,
		local_query.filter_count);

	input.row_hd    = pre_hd ? pre_hd : e->row_hd;
	input.fields_hd = e->fields_hd;
	memset(out, 0, sizeof(*out));

	hyle_apply_view(ctx, &input, &local_query,
		e->fields, e->field_count, out, &total32);

	if (pre_hd)
		qmap_close(pre_hd);
	free(local_filters);
	hyle_ctx_free(ctx);

	if (total_out)
		*total_out = (size_t)total32;
	return 0;
}

/* ---- accessors ---- */

unsigned hyle_source_get_row_hd(const char *source_id)
{
	const registry_entry_t *e = find_entry(source_id);
	return e ? e->row_hd : 0;
}

unsigned hyle_source_get_fields_hd(const char *source_id)
{
	const registry_entry_t *e = find_entry(source_id);
	return e ? e->fields_hd : 0;
}

void *hyle_source_get_user(const char *source_id)
{
	const registry_entry_t *e = find_entry(source_id);
	return e ? e->user : NULL;
}

void hyle_source_set_user(const char *source_id, void *user)
{
	registry_entry_t *e = find_entry(source_id);
	if (e)
		e->user = user;
}

size_t hyle_source_count(void)
{
	return registry_count;
}

const char *hyle_source_id_at(size_t i)
{
	return i < registry_count ? registry[i].source_id : NULL;
}

/* ---- hyle_source_rows_free ---- */

void hyle_source_rows_free(hyle_source_row_t *rows, size_t count)
{
	size_t i;
	size_t j;

	if (!rows)
		return;
	for (i = 0; i < count; i++) {
		if (rows[i].field_names) {
			for (j = 0; j < rows[i].field_count; j++)
				free((void *)rows[i].field_names[j]);
			free((void *)rows[i].field_names);
		}
		if (rows[i].field_values) {
			for (j = 0; j < rows[i].field_count; j++)
				free((void *)rows[i].field_values[j]);
			free((void *)rows[i].field_values);
		}
		free((void *)rows[i].id);
	}
	free(rows);
}

/* ---- hyle_row_set_to_rows ---- */

int hyle_row_set_to_rows(const hyle_row_set_t *rs,
	hyle_source_row_t **rows_out,
	size_t *count_out)
{
	uint32_t           total;
	hyle_source_row_t *rows;
	size_t             ri;
	uint32_t           cur;
	const void        *k;
	const void        *v;
	uint32_t           fcur;
	const void        *fk;
	const void        *fv;
	char               prefix[1024];
	size_t             plen;

	if (!rs || !rows_out || !count_out)
		return -1;

	total = qmap_count(rs->row_hd, NULL);
	*count_out = (size_t)total;
	*rows_out = NULL;

	if (total == 0)
		return 0;

	rows = (hyle_source_row_t *)calloc(total, sizeof(hyle_source_row_t));
	if (!rows)
		return -1;

	ri  = 0;
	cur = qmap_iter(rs->row_hd, NULL, 0);
	while (qmap_next(&k, &v, cur)) {
		const char  *id  = (const char *)k;
		size_t       cap = 16;
		size_t       cnt = 0;
		const char **names  = (const char **)malloc(
			cap * sizeof(char *));
		const char **values = (const char **)malloc(
			cap * sizeof(char *));

		if (!names || !values) {
			free(names);
			free(values);
			qmap_fin(cur);
			hyle_source_rows_free(rows, ri);
			return -1;
		}

		snprintf(prefix, sizeof(prefix), "%s:", id);
		plen = strlen(prefix);

		fcur = qmap_iter(rs->fields_hd, NULL, 0);
		while (qmap_next(&fk, &fv, fcur)) {
			const char *key = (const char *)fk;
			const char **tmp;

			if (strncmp(key, prefix, plen) != 0)
				continue;
			if (cnt >= cap) {
				cap *= 2;
				tmp = (const char **)realloc(
					(void *)names,  cap * sizeof(char *));
				if (!tmp) {
					qmap_fin(fcur);
					qmap_fin(cur);
					free((void *)names);
					free((void *)values);
					hyle_source_rows_free(rows, ri);
					return -1;
				}
				names = tmp;
				tmp = (const char **)realloc(
					(void *)values, cap * sizeof(char *));
				if (!tmp) {
					qmap_fin(fcur);
					qmap_fin(cur);
					free((void *)names);
					free((void *)values);
					hyle_source_rows_free(rows, ri);
					return -1;
				}
				values = tmp;
			}
			names[cnt]  = strdup(key + plen);
			values[cnt] = strdup((const char *)fv);
			cnt++;
		}
		qmap_fin(fcur);

		rows[ri].id           = strdup(id);
		rows[ri].field_names  = names;
		rows[ri].field_values = values;
		rows[ri].field_count  = cnt;
		ri++;
	}
	qmap_fin(cur);

	*rows_out = rows;
	return 0;
}
