#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <limits.h>
#include <ttypt/qmap.h>
#include <stoma/stoma.h>
#include "hyle/source.h"
#include "hyle/query.h"
#include "hyle/ctx.h"

/* ---- registry ---- */

#define HYLE_SOURCE_MAX 64

typedef struct {
	char source_id[128];
	unsigned row_hd;
	unsigned fields_hd;
	const hyle_field_t *fields;
	size_t field_count;
	uint32_t record_id;
	int row_owned; /* 1 = libhyle created row_hd */
	void *user;
	/* ordered source fields (zero for normal sources) */
	int ordered;
	char partition_field[64];
	hyle_persist_load_fn load_fn;
	hyle_persist_save_fn save_fn;
	void *persist_user;
	unsigned order_hd;  /* qmap: pval → count (str) */
	unsigned loaded_hd; /* qmap: pval → "" (loaded flag) */
	stoma_db_t *stoma;  /* full-text index, NULL if no searchable field */
	int stoma_dirty;
} registry_entry_t;

static registry_entry_t registry[HYLE_SOURCE_MAX];
static size_t registry_count = 0;

typedef struct {
	char derive_key[128];
	hyle_derive_fn_t fn;
	void *user;
} hyle_derive_reg_t;

static hyle_derive_reg_t derive_registry[32];
static int derive_registry_count = 0;

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

int hyle_register_derive(const char *derive_key, hyle_derive_fn_t fn, void *user)
{
	if (derive_registry_count >= 32)
		return -1;
	derive_registry[derive_registry_count++] = (hyle_derive_reg_t){
		{0}, fn, user
	};
	strncpy(derive_registry[derive_registry_count - 1].derive_key, derive_key, 127);
	return 0;
}


const char *hyle_call_derive(const void *def, const char *derive_key, const char *row_id, const char *field_name, void *user)
{
	if (!derive_key)
		return NULL;
	for (int i = 0; i < derive_registry_count; i++) {
		const hyle_derive_reg_t *dr = &derive_registry[i];
		if (strcmp(dr->derive_key, derive_key) == 0 && dr->fn)
			return dr->fn(def, row_id, field_name, dr->user);
	}
	return NULL;
}



/* ---- hyle_source_register ---- */

unsigned hyle_source_register(
        const char *source_id, const hyle_field_t *fields, size_t field_count,
        uint32_t record_id, unsigned flags, void *user)
{
	registry_entry_t *e;
	unsigned flds_hd;
	uint32_t val_type;
	size_t i;

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
	e->row_hd = qmap_open(NULL, NULL, QM_STR, val_type, 0x3FF, flags);
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
		flds_hd = qmap_open(NULL, NULL, QM_STR, QM_STR, 0x3FF, 0);
	}
	if (!flds_hd) {
		qmap_close(e->row_hd);
		e->row_hd = 0;
		e->row_owned = 0;
		return 0;
	}

	e->fields_hd = flds_hd;
	e->fields = fields;
	e->field_count = field_count;
	e->record_id = record_id;
	e->user = user;

	/* Full-text index: one per source, only when a field opts in. */
	if (e->stoma) {
		stoma_close(e->stoma);
		e->stoma = NULL;
	}
	for (i = 0; i < field_count; i++) {
		if (fields[i].searchable) {
			e->stoma = stoma_open(0);
			break;
		}
	}
	e->stoma_dirty = 1;

	return flds_hd;
}

/* ---- hyle_source_put ---- */

int hyle_source_put(
        const char *source_id, const char *row_id, const char **names,
        const char **values, size_t count)
{
	registry_entry_t *e;
	size_t i;
	char key[1024];

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
		if (e->record_id && !e->ordered) {
			/* Record-aware: per-field write with reference
			 * resolution (slug->position). Ordered sources store
			 * raw composite values and must keep this path. */
			qmap_field_put(
			        e->fields_hd, row_id, names[i],
			        values[i] ? values[i] : "");
		} else {
			snprintf(key, sizeof(key), "%s:%s", row_id, names[i]);
			qmap_put(e->fields_hd, key, values[i] ? values[i] : "");
		}
	}
	if (e->stoma)
		e->stoma_dirty = 1;
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
	if (e->stoma)
		e->stoma_dirty = 1;
}

/* ---- hyle_row_set_free ---- */

void hyle_row_set_free(hyle_row_set_t *rs)
{
	if (!rs)
		return;
	if (rs->row_hd)
		qmap_close(rs->row_hd);
	/* fields_hd always non-owned (points into registry) */
	rs->row_hd = 0;
	rs->fields_hd = 0;
}

/* ---- multi-reference pre-filter ---------------------------------------- */
/*
 * For sources whose multi-reference field values are stored as newline-
 * separated position indices (integers) rather than slugs, resolve
 * slug → position and pre-filter the row set so that apply_view sees
 * plain row IDs without the reference filter.
 *
 * Also handles single REFERENCE fields when ≥2 distinct values are
 * requested; single-value REFERENCE falls through to the residual
 * ci_substr path (byte-identical behaviour).
 *
 * Within-field combination is configurable: OR (union, default) or AND
 * (intersect), determined by the URL <field>_op override or the field's
 * combine flag. Cross-field combination is always AND.
 *
 * Returns a newly-opened pre-filtered row_hd (caller must close), or 0 if no
 * pre-filtering was needed (caller should use the original row_hd directly).
 * Zeroes out the filter value in local_filters for each handled filter.
 */
#define PREFILTER_MAX_FIELDS 16
#define PREFILTER_MAX_VALUES 32

/* Intersection of two qmap sets; returns a new handle (caller closes it
 * plus both inputs). 0 when either input is empty or alloc fails. */
static unsigned intersect_hds(unsigned a, unsigned b)
{
	unsigned out_hd;
	uint32_t cur;
	const void *k;
	const void *v;

	if (!a || !b)
		return 0;
	if (qmap_count(a, NULL) > qmap_count(b, NULL)) {
		unsigned tmp = a;

		a = b;
		b = tmp;
	}
	out_hd = qmap_open(NULL, NULL, QM_STR, QM_STR, 0xFF, 0);
	if (!out_hd)
		return 0;
	cur = qmap_iter(a, NULL, 0);
	while (qmap_next(&k, &v, cur)) {
		if (qmap_get(b, (const char *)k))
			qmap_put(out_hd, (const char *)k, "");
	}
	qmap_fin(cur);
	return out_hd;
}

/* True when any newline-separated token of fv equals any of vals[0..n-1]. */
static int multi_ref_token_match(
        const char *fv, const char *const *vals, int n)
{
	const char *p;

	if (!fv || n <= 0)
		return 0;
	p = fv;
	while (*p) {
		const char *nl = strchr(p, '\n');
		size_t len = nl ? (size_t)(nl - p) : strlen(p);
		int vi;

		for (vi = 0; vi < n; vi++) {
			if (len == strlen(vals[vi]) &&
			    strncmp(p, vals[vi], len) == 0)
				return 1;
		}
		if (!nl)
			break;
		p = nl + 1;
	}
	return 0;
}

/* True when every value in vals[0..n-1] appears as a newline-separated
 * token of fv. AND-within-field counterpart of multi_ref_token_match. */
static int multi_ref_all_tokens_match(
        const char *fv, const char *const *vals, int n)
{
	int vi;

	if (!fv || n <= 0)
		return 0;
	for (vi = 0; vi < n; vi++) {
		const char *p = fv;
		int found = 0;

		while (*p) {
			const char *nl = strchr(p, '\n');
			size_t len = nl ? (size_t)(nl - p) : strlen(p);

			if (len == strlen(vals[vi]) &&
			    strncmp(p, vals[vi], len) == 0) {
				found = 1;
				break;
			}
			if (!nl)
				break;
			p = nl + 1;
		}
		if (!found)
			return 0;
	}
	return 1;
}

/* True when fv exactly equals any of vals[0..n-1]. Single REFERENCE
 * fields store one slug per row. */
static int ref_value_matches(
        const char *fv, const char *const *vals, int n)
{
	int vi;

	if (!fv || n <= 0)
		return 0;
	for (vi = 0; vi < n; vi++) {
		if (strcmp(fv, vals[vi]) == 0)
			return 1;
	}
	return 0;
}

static unsigned prefilter_multi_ref(
        const registry_entry_t *e, hyle_field_filter_t *local_filters,
        unsigned filter_count, unsigned base_hd)
{
	char field_names[PREFILTER_MAX_FIELDS][64];
	const registry_entry_t *field_target[PREFILTER_MAX_FIELDS];
	int field_type[PREFILTER_MAX_FIELDS];
	int field_sj[PREFILTER_MAX_FIELDS]; /* index into e->fields for combine */
	int ref_hd[PREFILTER_MAX_FIELDS][PREFILTER_MAX_VALUES];
	int ref_hd_count[PREFILTER_MAX_FIELDS];
	int handled[PREFILTER_MAX_VALUES];
	int nfields = 0;
	int nhandled = 0;
	unsigned pre_hd = 0;
	unsigned fi;

	for (fi = 0; fi < PREFILTER_MAX_FIELDS; fi++)
		ref_hd_count[fi] = 0;

	/* Pass 0: collect distinct reference fields and handled filter indexes. */
	for (fi = 0; fi < filter_count; fi++) {
		const char *fname = local_filters[fi].field;
		const char *slug = local_filters[fi].value;
		size_t sj;
		int k;
		registry_entry_t *target = NULL;

		if (!fname || !slug || !slug[0])
			continue;
		for (sj = 0; sj < e->field_count; sj++) {
			if (strcmp(e->fields[sj].name, fname) == 0)
				break;
		}
		if (sj >= e->field_count)
			continue;
		if (e->fields[sj].type != HYLE_FIELD_MULTI_REFERENCE &&
		    e->fields[sj].type != HYLE_FIELD_REFERENCE)
			continue;
		/* MULTI_REFERENCE needs target for qmap_pos slug→pos resolution */
		if (e->fields[sj].type == HYLE_FIELD_MULTI_REFERENCE) {
			target = e->fields[sj].target_source
			                 ? find_entry(e->fields[sj].target_source)
			                 : NULL;
			if (!target || !target->fields_hd)
				continue;
		}
		if (nfields >= PREFILTER_MAX_FIELDS ||
		    nhandled >= PREFILTER_MAX_VALUES)
			break;
		for (k = 0; k < nfields; k++) {
			if (strcmp(field_names[k], fname) == 0)
				break;
		}
		if (k >= nfields) {
			strncpy(field_names[nfields], fname,
			        sizeof(field_names[0]) - 1);
			field_names[nfields][sizeof(field_names[0]) - 1] = '\0';
			field_target[nfields] = target;
			field_type[nfields] = (int)e->fields[sj].type;
			field_sj[nfields] = (int)sj;
			ref_hd_count[nfields] = 0;
			nfields++;
		}
		handled[nhandled++] = (int)fi;
		/* Track REFERENCE filter indexes per field for distinct-value count */
		if (e->fields[sj].type == HYLE_FIELD_REFERENCE &&
		    ref_hd_count[k] < PREFILTER_MAX_VALUES)
			ref_hd[k][ref_hd_count[k]++] = (int)fi;
	}
	if (nhandled == 0)
		return 0;

	/* Prune REFERENCE fields with <2 distinct values — keep them on the
	 * residual ci_substr path so single-value behaviour is byte-identical. */
	{
		int fk2;

		for (fk2 = 0; fk2 < nfields; fk2++) {
			const char *distinct[PREFILTER_MAX_VALUES];
			int nd = 0;
			int hi;

			if (field_type[fk2] != HYLE_FIELD_REFERENCE)
				continue;
			for (hi = 0; hi < ref_hd_count[fk2]; hi++) {
				const char *v =
				        local_filters[ref_hd[fk2][hi]].value;
				int d;

				if (!v)
					continue;
				for (d = 0; d < nd; d++)
					if (strcmp(distinct[d], v) == 0)
						break;
				if (d >= nd && nd < PREFILTER_MAX_VALUES)
					distinct[nd++] = v;
			}
			if (nd < 2) {
				int h2;

				for (h2 = 0; h2 < ref_hd_count[fk2]; h2++) {
					int drop = ref_hd[fk2][h2];
					int h3;

					for (h3 = 0; h3 < nhandled; h3++) {
						if (handled[h3] == drop) {
							handled[h3] =
							        handled[nhandled -
							                1];
							nhandled--;
							break;
						}
					}
				}
			}
		}
	}
	if (nhandled == 0)
		return 0;

	/* Pass 1: per field, union (OR) or intersect (AND) every value's
	 * matching rows into one set; intersect the sets across fields. */
	{
		int fk;

		for (fk = 0; fk < nfields; fk++) {
			char pos_vals[PREFILTER_MAX_VALUES][64];
			const char *vals[PREFILTER_MAX_VALUES];
			int nvals = 0;
			const char *override_op = NULL;
			int mode;
			unsigned set_hd;
			uint32_t cur;
			const void *k;
			const void *v;
			int hi;

			for (hi = 0; hi < nhandled; hi++) {
				const char *fname =
				        local_filters[handled[hi]].field;
				const char *slug =
				        local_filters[handled[hi]].value;

				if (strcmp(fname, field_names[fk]) != 0)
					continue;
				/* Collect override_op from any filter's op */
				if (local_filters[handled[hi]].op &&
				    !override_op)
					override_op =
					        local_filters[handled[hi]].op;

				if (field_type[fk] ==
				    HYLE_FIELD_MULTI_REFERENCE) {
					uint32_t pos = qmap_pos(
					        field_target[fk]->fields_hd,
					        slug);

					if (pos != UINT32_MAX)
						snprintf(pos_vals[nvals],
						         sizeof(pos_vals[nvals]),
						         "%u", pos);
					else
						snprintf(pos_vals[nvals],
						         sizeof(pos_vals[nvals]),
						         "%s", slug);
					vals[nvals] = pos_vals[nvals];
				} else {
					/* REFERENCE: raw slug, no pos lookup */
					vals[nvals] = slug;
				}
				nvals++;
			}

			if (nvals == 0)
				continue;

			mode = override_op
			               ? (strcmp(override_op, "and") == 0)
			               : e->fields[field_sj[fk]].combine;

			set_hd = qmap_open(NULL, NULL, QM_STR, QM_STR, 0xFF,
			                   0);
			if (!set_hd)
				continue;
			cur = qmap_iter(base_hd ? base_hd : e->row_hd, NULL,
			                0);
			while (qmap_next(&k, &v, cur)) {
				const char *rid = (const char *)k;
				const char *fv;
				int m;

				fv = qmap_field_get(e->fields_hd, rid,
				                    field_names[fk]);
				if (field_type[fk] == HYLE_FIELD_REFERENCE) {
					/* AND on a single-valued field: a value
					 * can't equal ≥2 distinct slugs →
					 * always empty (correct degenerate). */
					m = mode ? 0
					         : ref_value_matches(fv, vals,
					                             nvals);
				} else if (mode) {
					m = multi_ref_all_tokens_match(fv, vals,
					                               nvals);
				} else {
					m = multi_ref_token_match(fv, vals,
					                          nvals);
				}
				if (m)
					qmap_put(set_hd, rid, "");
			}
			qmap_fin(cur);

			if (pre_hd) {
				unsigned next_hd =
				        intersect_hds(pre_hd, set_hd);

				qmap_close(pre_hd);
				qmap_close(set_hd);
				pre_hd = next_hd;
			} else {
				pre_hd = set_hd;
			}
		}
	}

	/* Pass 2: drop the handled values so apply_view doesn't
	 * double-filter. */
	{
		int hi;

		for (hi = 0; hi < nhandled; hi++)
			local_filters[handled[hi]].value = NULL;
	}

	return pre_hd;
}

/* ---- full-text pre-filter --------------------------------------------- */
/*
 * For searchable fields, resolve each filter against the stoma index and
 * intersect the matching row sets, so apply_view only sees rows that match.
 * Rebuilds the index lazily (stoma_clear + re-index) the first time a
 * searchable filter arrives after any mutation.
 *
 * Returns a newly-opened candidate row_hd (caller must close), or 0 if no
 * full-text pre-filtering was needed. Zeroes the filter value in
 * local_filters for each handled filter.
 */
static unsigned prefilter_fts(
        registry_entry_t *e, hyle_field_filter_t *local_filters,
        unsigned filter_count)
{
	unsigned fi;
	unsigned fts_hd = 0;

	if (!e->stoma)
		return 0;

	/* Any searchable filter present? Rebuild the index if stale. */
	for (fi = 0; fi < filter_count; fi++) {
		size_t sj;

		if (!local_filters[fi].field)
			continue;
		for (sj = 0; sj < e->field_count; sj++) {
			if (strcmp(e->fields[sj].name,
			           local_filters[fi].field) == 0)
				break;
		}
		if (sj < e->field_count && e->fields[sj].searchable)
			break;
	}
	if (fi >= filter_count)
		return 0;

	if (e->stoma_dirty) {
		uint32_t cur;
		const void *k;
		const void *v;

		stoma_clear(e->stoma);
		cur = qmap_iter(e->row_hd, NULL, 0);
		while (qmap_next(&k, &v, cur)) {
			const char *rid = (const char *)k;
			size_t sj;

			for (sj = 0; sj < e->field_count; sj++) {
				char key[1024];
				const char *fv;
				const char *fname = e->fields[sj].name;

				if (!e->fields[sj].searchable || !fname || !fname[0])
					continue;
				if (!rid || !rid[0])
					continue;
				if (e->fields[sj].type == HYLE_FIELD_DERIVED) {
					const void *def = e->user;
					fv = hyle_call_derive(def, e->fields[sj].derive_key, rid, fname, NULL);
				} else {
					snprintf(
					        key, sizeof(key), "%s:%s", rid,
					        fname);
					fv = qmap_get(e->fields_hd, key);
				}
				if (fv && *fv) {
					stoma_index(
					        e->stoma, fname,
					        rid, fv);
					if (e->fields[sj].type == HYLE_FIELD_DERIVED) {
						snprintf(key, sizeof(key), "%s:%s", rid, fname);
						qmap_put(e->fields_hd, key, fv);
					}
				}
			}
		}
		qmap_fin(cur);
		e->stoma_dirty = 0;
	}

	for (fi = 0; fi < filter_count; fi++) {
		const char *fname = local_filters[fi].field;
		const char *value = local_filters[fi].value;
		size_t sj;
		size_t vlen;
		char *inner;
		unsigned tmp_hd;
		uint32_t cur;
		const void *k;
		const void *v;
		int handled = 0;

		if (!fname || !value || !value[0])
			continue;
		for (sj = 0; sj < e->field_count; sj++) {
			if (strcmp(e->fields[sj].name, fname) == 0)
				break;
		}
		if (sj >= e->field_count)
			continue;
		if (!e->fields[sj].searchable)
			continue;

		tmp_hd = qmap_open(NULL, NULL, QM_STR, QM_STR, 0xFF, 0);
		if (!tmp_hd)
			continue;

		vlen = strlen(value);
		if (value[0] == '"') {
			/* Quoted value: positional phrase query on the inner
			 * text. The value points into the parsed query string,
			 * so copy it before stripping the trailing quote. */
			if (vlen >= 2 && value[vlen - 1] == '"') {
				inner = malloc(vlen - 1);
				if (!inner) {
					qmap_close(tmp_hd);
					continue;
				}
				memcpy(inner, value + 1, vlen - 2);
				inner[vlen - 2] = '\0';
				if (inner[0])
					stoma_query_phrase(
					        e->stoma, fname, inner, tmp_hd,
					        &handled);
				else
					handled = 0;
				free(inner);
			} else {
				handled = 0;
			}
		} else {
			stoma_query(e->stoma, fname, value, tmp_hd, &handled);
		}
		if (!handled) {
			qmap_close(tmp_hd);
			continue;
		}

		if (fts_hd) {
			/* Intersect: keep rows present in both sets */
			unsigned next_hd =
			        qmap_open(NULL, NULL, QM_STR, QM_STR, 0xFF, 0);
			if (!next_hd) {
				qmap_close(tmp_hd);
				continue;
			}
			cur = qmap_iter(fts_hd, NULL, 0);
			while (qmap_next(&k, &v, cur)) {
				const char *rid = (const char *)k;

				if (qmap_get(tmp_hd, rid))
					qmap_put(next_hd, rid, "");
			}
			qmap_fin(cur);
			qmap_close(fts_hd);
			qmap_close(tmp_hd);
			fts_hd = next_hd;
		} else {
			fts_hd = tmp_hd;
		}

		local_filters[fi].value = NULL;
	}

	return fts_hd;
}

/* ---- q pre-filter (stored values + resolved ref labels) --------------- */

static const char *display_field_of(const registry_entry_t *t)
{
	size_t i;

	if (!t)
		return NULL;
	for (i = 0; i < t->field_count; i++) {
		if (t->fields[i].name &&
		    strcmp(t->fields[i].name, "id") != 0)
			return t->fields[i].name;
	}
	return NULL;
}

static const char *ref_token_label(
        const registry_entry_t *target, const char *token, const char *df)
{
	const char *slug;
	char key[1024];

	if (!target || !token || !*token || !df)
		return NULL;
	slug = token;
	if (token[0] >= '0' && token[0] <= '9') {
		const char *pos_slug;

		pos_slug = qmap_get_key(
		        target->fields_hd, (uint32_t)atoi(token));
		if (pos_slug)
			slug = pos_slug;
	}
	snprintf(key, sizeof(key), "%s:%s", slug, df);
	return (const char *)qmap_get(target->fields_hd, key);
}

static int token_label_matches(
        const registry_entry_t *target, const char *token, const char *df,
        const char *q, int punct_norm)
{
	const char *label;

	label = ref_token_label(target, token, df);
	return label && (punct_norm ? hyle_ci_substr_punct(label, q) : hyle_ci_substr(label, q));
}

static int stored_labels_match(
        const registry_entry_t *target, const char *stored, const char *df,
        const char *q, int punct_norm)
{
	const char *p;

	p = stored;
	while (*p) {
		const char *nl;
		char token[256];
		size_t n;

		nl = strchr(p, '\n');
		n = nl ? (size_t)(nl - p) : strlen(p);
		if (n >= sizeof(token))
			n = sizeof(token) - 1;
		if (n > 0) {
			memcpy(token, p, n);
			token[n] = '\0';
			if (token_label_matches(target, token, df, q, punct_norm))
				return 1;
		}
		if (!nl)
			break;
		p = nl + 1;
	}
	return 0;
}

static int row_matches_q(
        const registry_entry_t *e, const char *row_id, const char *q, int punct_norm)
{
	size_t i;

	if (punct_norm ? hyle_ci_substr_punct(row_id, q) : hyle_ci_substr(row_id, q))
		return 1;
	for (i = 0; i < e->field_count; i++) {
		const hyle_field_t *f = &e->fields[i];
		char key[1024];
		const char *stored;
		const registry_entry_t *target;
		const char *df;

		if (!f->name)
			continue;
		snprintf(key, sizeof(key), "%s:%s", row_id, f->name);
		stored = (const char *)qmap_get(e->fields_hd, key);
		if (stored && (punct_norm ? hyle_ci_substr_punct(stored, q) : hyle_ci_substr(stored, q)))
			return 1;
		if (f->type != HYLE_FIELD_REFERENCE &&
		    f->type != HYLE_FIELD_MULTI_REFERENCE)
			continue;
		if (!f->target_source || !f->target_source[0] || !stored)
			continue;
		target = find_entry(f->target_source);
		if (!target)
			continue;
		df = display_field_of(target);
		if (!df)
			continue;
		if (stored_labels_match(target, stored, df, q, punct_norm))
			return 1;
	}
	return 0;
}

static unsigned prefilter_q(
        const registry_entry_t *e, const char *q, unsigned base_hd)
{
	unsigned q_hd;
	uint32_t cur;
	const void *k;
	const void *v;

	if (!q || !*q || !base_hd)
		return 0;
	q_hd = qmap_open(NULL, NULL, QM_STR, QM_STR, 0xFF, 0);
	if (!q_hd)
		return 0;
	cur = qmap_iter(base_hd, NULL, 0);
	while (qmap_next(&k, &v, cur)) {
		const char *rid = (const char *)k;

		if (row_matches_q(e, rid, q, 1))
			qmap_put(q_hd, rid, "");
	}
	qmap_fin(cur);
	return q_hd;
}

/* ---- hyle_source_query ---- */

int hyle_source_query(
        const char *source_id, const hyle_query_t *query, hyle_row_set_t *out,
        size_t *total_out)
{
	registry_entry_t *e;
	hyle_row_set_t input;
	hyle_ctx_t *ctx;
	uint32_t total32 = 0;
	hyle_field_filter_t *local_filters = NULL;
	hyle_query_t local_query;
	unsigned pre_hd = 0;
	unsigned fts_hd = 0;
	unsigned q_hd = 0;
	unsigned base;

	if (!source_id || !query || !out)
		return -1;
	e = find_entry(source_id);
	if (!e) {
		fprintf(stderr, "hyle_source_query: source '%s' not found\n",
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

	/* Full-text pre-filter first (smallest candidate set), then multi-ref
	 */
	fts_hd =
	        prefilter_fts(e, local_query.filters, local_query.filter_count);

	/* Multi-reference pre-filter for typed-record sources */
	pre_hd = prefilter_multi_ref(
	        e, local_query.filters, local_query.filter_count,
	        fts_hd ? fts_hd : e->row_hd);

	base = pre_hd ? pre_hd : (fts_hd ? fts_hd : e->row_hd);
	if (local_query.q && *local_query.q) {
		q_hd = prefilter_q(e, local_query.q, base);
		if (q_hd) {
			input.row_hd = q_hd;
			local_query.q = NULL;
		} else {
			input.row_hd = base;
		}
	} else {
		input.row_hd = base;
	}
	input.fields_hd = e->fields_hd;
	memset(out, 0, sizeof(*out));

	hyle_apply_view(
	        ctx, &input, &local_query, e->fields, e->field_count, out,
	        &total32);

	if (q_hd)
		qmap_close(q_hd);
	if (pre_hd)
		qmap_close(pre_hd);
	if (fts_hd)
		qmap_close(fts_hd);
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

size_t hyle_source_get_field_count(const char *source_id)
{
	registry_entry_t *e = find_entry(source_id);
	return e ? e->field_count : 0;
}

const char *hyle_source_get_field_name(const char *source_id, size_t idx)
{
	registry_entry_t *e = find_entry(source_id);
	if (!e || idx >= e->field_count)
		return NULL;
	return e->fields[idx].name;
}

hyle_field_type_t hyle_source_get_field_type(const char *source_id, size_t idx)
{
	registry_entry_t *e = find_entry(source_id);
	if (!e || idx >= e->field_count)
		return 0;
	return e->fields[idx].type;
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

int hyle_row_set_to_rows(
        const hyle_row_set_t *rs, hyle_source_row_t **rows_out,
        size_t *count_out)
{
	uint32_t total;
	hyle_source_row_t *rows;
	size_t ri;
	uint32_t cur;
	const void *k;
	const void *v;
	uint32_t fcur;
	const void *fk;
	const void *fv;
	char prefix[1024];
	size_t plen;

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

	ri = 0;
	cur = qmap_iter(rs->row_hd, NULL, 0);
	while (qmap_next(&k, &v, cur)) {
		const char *id = (const char *)k;
		size_t cap = 16;
		size_t cnt = 0;
		const char **names =
		        (const char **)malloc(cap * sizeof(char *));
		const char **values =
		        (const char **)malloc(cap * sizeof(char *));

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
				        (void *)names, cap * sizeof(char *));
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
			names[cnt] = strdup(key + plen);
			values[cnt] = strdup((const char *)fv);
			cnt++;
		}
		qmap_fin(fcur);

		rows[ri].id = strdup(id);
		rows[ri].field_names = names;
		rows[ri].field_values = values;
		rows[ri].field_count = cnt;
		ri++;
	}
	qmap_fin(cur);

	*rows_out = rows;
	return 0;
}

/* ===================================================================
 * Ordered source: positional arrays per partition, with pluggable
 * persistence callbacks.
 *
 * Keys are "{partition_val}__{NNNN}" stored in row_hd / fields_hd.
 * order_hd:  pval → count_as_string
 * loaded_hd: pval → ""  (marks whether partition has been loaded)
 * =================================================================== */

/* ---- helpers ----------------------------------------------------------- */

static void ordered_build_key(char *buf, size_t sz, const char *pval, int pos)
{
	snprintf(buf, sz, "%s__%04d", pval, pos);
}

/* Count items prefixed with "pval__" by iterating row_hd. */
static int ordered_partition_count(unsigned row_hd, const char *pval)
{
	size_t plen = strlen(pval);
	int count = 0;
	uint32_t cur = qmap_iter(row_hd, NULL, 0);
	const void *k, *v;
	while (qmap_next(&k, &v, cur)) {
		const char *key = (const char *)k;
		if (strncmp(key, pval, plen) == 0 && key[plen] == '_' &&
		    key[plen + 1] == '_')
			count++;
	}
	qmap_fin(cur);
	return count;
}

/* Move field values from position `from` to `to` within the partition. */
static void
ordered_move_key(registry_entry_t *e, const char *pval, int from, int to)
{
	char old_key[128], new_key[128];
	ordered_build_key(old_key, sizeof(old_key), pval, from);
	ordered_build_key(new_key, sizeof(new_key), pval, to);

	if (e->record_id) {
		/* Record-aware: copy entire struct to avoid qmap_put resize
		 * invalidating the pointer returned by qmap_get. */
		const void *data = qmap_get(e->fields_hd, old_key);
		if (data) {
			size_t sz = qmap_type_len(qmap_get_vtype(e->fields_hd));
			void *buf = malloc(sz);
			if (buf) {
				memcpy(buf, data, sz);
				qmap_put(e->fields_hd, new_key, buf);
				free(buf);
			}
		}
	} else {
		/* Plain map: iterate and rename field entries. */
		size_t olen = strlen(old_key);
		uint32_t cur = qmap_iter(e->fields_hd, NULL, 0);
		const void *k, *v;
		while (qmap_next(&k, &v, cur)) {
			const char *fkey = (const char *)k;
			if (strncmp(fkey, old_key, olen) != 0)
				continue;
			if (fkey[olen] != ':')
				continue;
			const char *fname = fkey + olen + 1;
			char nk[256];
			snprintf(nk, sizeof(nk), "%s:%s", new_key, fname);
			qmap_put(e->fields_hd, nk, (const char *)v);
		}
		qmap_fin(cur);
	}
	qmap_put(e->row_hd, new_key, "");
	qmap_del(e->row_hd, old_key);
	qmap_del(e->fields_hd, old_key);
	if (e->stoma)
		e->stoma_dirty = 1;
}

static int ordered_ensure_loaded(const char *source_id, const char *pval)
{
	registry_entry_t *e;
	int count;
	char c[16];

	e = find_entry(source_id);
	if (!e || !e->ordered)
		return -1;
	if (!pval || !pval[0])
		return -1;
	if (qmap_get(e->loaded_hd, pval))
		return 0;
	qmap_put(e->loaded_hd, pval, "");
	if (e->load_fn)
		e->load_fn(source_id, pval, e->fields_hd, e->persist_user);
	count = ordered_partition_count(e->row_hd, pval);
	snprintf(c, sizeof(c), "%d", count);
	qmap_put(e->order_hd, pval, c);
	return 0;
}

static int ordered_save(const char *source_id, const char *pval)
{
	registry_entry_t *e = find_entry(source_id);
	if (!e || !e->ordered)
		return -1;
	if (!e->save_fn)
		return 0;
	return e->save_fn(source_id, pval, e->fields_hd, e->persist_user);
}

/* ---- auto-record (create qmap record from hyle field metadata) --------- */

#define HYLE_AUTO_MAX_FIELDS 64

static uint32_t
hyle_source_auto_record(const hyle_field_t *fields, size_t field_count)
{
	qmap_record_field_t qfields[HYLE_AUTO_MAX_FIELDS];
	size_t target_idx[HYLE_AUTO_MAX_FIELDS];
	size_t n;
	size_t struct_size;
	size_t i;
	uint32_t rid;

	n = 0;
	struct_size = 0;
	for (i = 0; i < field_count && n < HYLE_AUTO_MAX_FIELDS; i++) {
		size_t sz;
		uint32_t qt;

		switch (fields[i].type) {
		case HYLE_FIELD_STRING:
		case HYLE_FIELD_NULLABLE_STRING:
			sz = fields[i].max_length > 0 ? fields[i].max_length
			                              : 256;
			qt = QM_STR;
			break;
		case HYLE_FIELD_INT:
		case HYLE_FIELD_BOOL:
			sz = 16;
			qt = QM_STR;
			break;
		case HYLE_FIELD_REFERENCE:
			sz = 128;
			qt = QM_REFERENCE;
			break;
		case HYLE_FIELD_MULTI_REFERENCE:
			sz = 1024;
			qt = QM_MULTI_REFERENCE;
			break;
		default:
			continue;
		}

		qfields[n].name = fields[i].name;
		qfields[n].type = qt;
		qfields[n].offset = struct_size;
		qfields[n].max_size = sz;
		qfields[n].target_record = 0;
		qfields[n].target_hd = 0;
		qfields[n].inverse = NULL;
		target_idx[n] = i;
		struct_size += sz;
		n++;
	}

	if (n == 0)
		return 0;

	rid = qmap_record_register("hyle_auto", struct_size, qfields, n);

	if (rid == UINT32_MAX)
		return 0;

	/* Resolve target handles for reference fields */
	for (i = 0; i < n; i++) {
		if (qfields[i].type == QM_REFERENCE ||
		    qfields[i].type == QM_MULTI_REFERENCE)
		{
			size_t fi = target_idx[i];
			if (fields[fi].target_source) {
				unsigned thd = hyle_source_get_fields_hd(
				        fields[fi].target_source);
				if (thd)
					qmap_record_field_set_target_hd(
					        rid, qfields[i].name, thd);
			}
		}
	}

	return rid;
}

/* ---- public API ------------------------------------------------------- */

unsigned hyle_source_register_ordered(
        const char *source_id, const hyle_field_t *fields, size_t field_count,
        const char *partition_field, uint32_t record_id, unsigned flags,
        hyle_persist_load_fn load_fn, hyle_persist_save_fn save_fn,
        void *persist_user)
{
	unsigned fhd;
	registry_entry_t *e;
	unsigned hyle_flags;

	hyle_flags = flags & HYLE_AUTO_RECORD;
	flags = flags & ~HYLE_AUTO_RECORD;

	if ((hyle_flags & HYLE_AUTO_RECORD) && record_id == 0 && fields &&
	    field_count > 0)
		record_id = hyle_source_auto_record(fields, field_count);

	fhd = hyle_source_register(
	        source_id, fields, field_count, record_id, flags, NULL);
	if (!fhd)
		return 0;
	e = find_entry(source_id);
	if (!e)
		return 0;
	/* Close previous order/loaded maps if re-registering */
	if (e->order_hd)
		qmap_close(e->order_hd);
	if (e->loaded_hd)
		qmap_close(e->loaded_hd);
	e->ordered = 1;
	strncpy(e->partition_field, partition_field ? partition_field : "",
	        sizeof(e->partition_field) - 1);
	e->load_fn = load_fn;
	e->save_fn = save_fn;
	e->persist_user = persist_user;
	e->order_hd = qmap_open(NULL, NULL, QM_STR, QM_STR, 0x3F, 0);
	e->loaded_hd = qmap_open(NULL, NULL, QM_STR, QM_STR, 0x3F, 0);
	return fhd;
}

int hyle_source_ordered_count(const char *source_id, const char *pval)
{
	registry_entry_t *e;
	const char *c;

	if (ordered_ensure_loaded(source_id, pval) != 0)
		return 0;
	e = find_entry(source_id);
	if (!e)
		return 0;
	c = qmap_get(e->order_hd, pval);
	return c ? atoi(c) : 0;
}

const char *
hyle_source_ordered_key_at(const char *source_id, const char *pval, int pos)
{
	static char key[128];

	if (ordered_ensure_loaded(source_id, pval) != 0)
		return NULL;
	ordered_build_key(key, sizeof(key), pval, pos);
	return key;
}

int hyle_source_ordered_append(
        const char *source_id, const char *pval, const char **names,
        const char **values, size_t count)
{
	registry_entry_t *e;
	int n;
	char key[128], c[16];

	if (ordered_ensure_loaded(source_id, pval) != 0)
		return -1;
	n = hyle_source_ordered_count(source_id, pval);
	ordered_build_key(key, sizeof(key), pval, n);
	hyle_source_put(source_id, key, names, values, count);
	e = find_entry(source_id);
	if (!e)
		return -1;
	snprintf(c, sizeof(c), "%d", n + 1);
	qmap_put(e->order_hd, pval, c);
	ordered_save(source_id, pval);
	return 0;
}

int hyle_source_ordered_insert_at(
        const char *source_id, const char *pval, int pos, const char **names,
        const char **values, size_t count)
{
	registry_entry_t *e;
	int n;
	char key[128], c[16];

	if (ordered_ensure_loaded(source_id, pval) != 0)
		return -1;
	e = find_entry(source_id);
	if (!e)
		return -1;
	n = hyle_source_ordered_count(source_id, pval);
	if (pos < 0 || pos > n)
		return -1;
	/* Reindex forward: [pos..n-1] → [pos+1..n] */
	for (n = n - 1; n >= pos; n--)
		ordered_move_key(e, pval, n, n + 1);
	/* n is now pos-1; restore */
	n = hyle_source_ordered_count(source_id, pval);
	ordered_build_key(key, sizeof(key), pval, pos);
	hyle_source_put(source_id, key, names, values, count);
	n++;
	snprintf(c, sizeof(c), "%d", n);
	qmap_put(e->order_hd, pval, c);
	ordered_save(source_id, pval);
	return 0;
}

void hyle_source_ordered_remove_at(
        const char *source_id, const char *pval, int pos)
{
	registry_entry_t *e;
	int n;
	char key[128], c[16];
	int i;

	if (ordered_ensure_loaded(source_id, pval) != 0)
		return;
	e = find_entry(source_id);
	if (!e)
		return;
	n = hyle_source_ordered_count(source_id, pval);
	if (pos < 0 || pos >= n)
		return;
	ordered_build_key(key, sizeof(key), pval, pos);
	hyle_source_del(source_id, key);
	/* Reindex backward: [pos+1..n-1] → [pos..n-2] */
	for (i = pos; i < n - 1; i++)
		ordered_move_key(e, pval, i + 1, i);
	n--;
	snprintf(c, sizeof(c), "%d", n);
	qmap_put(e->order_hd, pval, c);
	ordered_save(source_id, pval);
}

void hyle_source_ordered_clear(const char *source_id, const char *pval)
{
	registry_entry_t *e;
	int n;
	int i;
	char key[128];

	if (ordered_ensure_loaded(source_id, pval) != 0)
		return;
	e = find_entry(source_id);
	if (!e)
		return;
	n = hyle_source_ordered_count(source_id, pval);
	/* Delete all positional keys */
	for (i = 0; i < n; i++) {
		ordered_build_key(key, sizeof(key), pval, i);
		hyle_source_del(source_id, key);
	}
	qmap_put(e->order_hd, pval, "0");
	ordered_save(source_id, pval);
}

void hyle_source_ordered_save(const char *source_id, const char *pval)
{
	ordered_save(source_id, pval);
}
