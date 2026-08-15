#include <stdlib.h>
#include <string.h>
#include <strings.h>
#include <stdio.h>
#include <ctype.h>
#include <math.h>
#include <unistd.h>
#include <stoma/stoma.h>
#include "hyle/query.h"
#include "hyle/field.h"
#include <ttypt/qmap.h>

/* ---- helpers ---- */

static const char *row_field_val(const hyle_row_set_t *rows,
	const char *id, const char *field)
{
	char key[1024];
	snprintf(key, sizeof(key), "%s:%s", id, field);
	return (const char *)qmap_get(rows->fields_hd, key);
}

static int ci_substr_raw(const char *str, const char *sub)
{
	size_t slen = strlen(str);
	size_t nlen = strlen(sub);
	size_t i;

	if (nlen > slen)
		return 0;

	for (i = 0; i <= slen - nlen; i++) {
		size_t j;
		for (j = 0; j < nlen; j++) {
			if (tolower((unsigned char)str[i + j])
			    != tolower((unsigned char)sub[j]))
				break;
		}
		if (j == nlen)
			return 1;
	}
	return 0;
}

static int ci_substr(const char *str, const char *sub)
{
	char   fstr[4096];
	char   fsub[256];
	size_t slen;
	size_t nlen;
	size_t i;

	if (!sub || !*sub)
		return 1;
	if (!str)
		return 0;

	if (stoma_fold(fsub, sizeof(fsub), sub) < 0 ||
	    stoma_fold(fstr, sizeof(fstr), str) < 0)
		return ci_substr_raw(str, sub);

	slen = strlen(fstr);
	nlen = strlen(fsub);
	if (nlen > slen)
		return 0;

	for (i = 0; i <= slen - nlen; i++) {
		size_t j;
		for (j = 0; j < nlen; j++) {
			if (fstr[i + j] != fsub[j])
				break;
		}
		if (j == nlen)
			return 1;
	}
	return 0;
}

/* ---- multi-ref token match ---- */

/*
 * Returns 1 if any newline-delimited token in haystack equals needle
 * (case-insensitive exact token match).
 */
static int token_match(const char *haystack, const char *needle)
{
	const char *p;
	const char *nl;
	size_t      nlen;

	if (!haystack || !needle || !*needle)
		return !needle || !*needle;
	nlen = strlen(needle);
	p = haystack;
	while (*p) {
		nl = strchr(p, '\n');
		size_t tlen = nl ? (size_t)(nl - p) : strlen(p);

		if (tlen == nlen && strncasecmp(p, needle, nlen) == 0)
			return 1;
		if (!nl)
			break;
		p = nl + 1;
	}
	return 0;
}

static int is_multi_ref(const hyle_field_t *fields, size_t field_count,
	const char *name)
{
	size_t i;

	if (!fields || !name)
		return 0;
	for (i = 0; i < field_count; i++) {
		if (fields[i].name && strcmp(fields[i].name, name) == 0)
			return fields[i].type == HYLE_FIELD_MULTI_REFERENCE;
	}
	return 0;
}

/* ---- hyle_filter_rows ---- */

void hyle_filter_rows(hyle_ctx_t *ctx,
	const hyle_row_set_t *input,
	const char *q,
	const hyle_field_filter_t *filters,
	unsigned filter_count,
	const hyle_field_t *fields,
	size_t field_count,
	hyle_row_set_t *output)
{
	(void)ctx;
	output->fields_hd = input->fields_hd;

	uint32_t cur = qmap_iter(input->row_hd, NULL, 0);
	const void *k;
	const void *v;

	while (qmap_next(&k, &v, cur)) {
		const char *row_id = (const char *)k;

		int match = 1;

		for (unsigned i = 0; i < filter_count; i++) {
			const char *fv = row_field_val(input, row_id,
				filters[i].field);
			int ok;

			if (is_multi_ref(fields, field_count,
				    filters[i].field))
				ok = token_match(fv, filters[i].value);
			else
				ok = ci_substr(fv, filters[i].value);
			if (!ok) {
				match = 0;
				break;
			}
		}
		if (!match)
			continue;

		if (q && *q) {
			int found = ci_substr(row_id, q);
			if (!found) {
				char prefix[1024];
				snprintf(prefix, sizeof(prefix), "%s:", row_id);
				size_t plen = strlen(prefix);

				uint32_t fc = qmap_iter(
					input->fields_hd, NULL, 0);
				const void *fk;
				const void *fv2;
				while (qmap_next(&fk, &fv2, fc)) {
					const char *key = (const char *)fk;
					if (strncmp(key, prefix, plen) != 0)
						continue;
					if (ci_substr(
						(const char *)fv2, q)) {
						found = 1;
						break;
					}
				}
				qmap_fin(fc);
			}
			if (!found)
				continue;
		}

		qmap_put(output->row_hd, row_id, "");
	}
	qmap_fin(cur);
}

/* ---- hyle_sort_rows ---- */

typedef struct {
	const char *id;
	double num_val;
	const char *str_val;
	int is_num;
} sort_entry_t;

static int sort_cmp_asc(const void *a, const void *b)
{
	const sort_entry_t *ea = (const sort_entry_t *)a;
	const sort_entry_t *eb = (const sort_entry_t *)b;

	if (!ea->str_val && !eb->str_val) return 0;
	if (!ea->str_val) return -1;
	if (!eb->str_val) return 1;

	if (ea->is_num && eb->is_num) {
		if (ea->num_val < eb->num_val) return -1;
		if (ea->num_val > eb->num_val) return 1;
		return 0;
	}

	return strcasecmp(ea->str_val, eb->str_val);
}

static int sort_cmp_desc(const void *a, const void *b)
{
	return -sort_cmp_asc(a, b);
}

void hyle_sort_rows(hyle_ctx_t *ctx,
	const hyle_row_set_t *input,
	const char *sort_field,
	bool sort_asc,
	hyle_row_set_t *output)
{
	(void)ctx;
	output->fields_hd = input->fields_hd;

	if (!sort_field) {
		uint32_t cur = qmap_iter(input->row_hd, NULL, 0);
		const void *k;
		const void *v;
		while (qmap_next(&k, &v, cur)) {
			qmap_put(output->row_hd, (const char *)k, "");
		}
		qmap_fin(cur);
		return;
	}

	uint32_t count = qmap_count(input->row_hd, NULL);
	if (count == 0)
		return;

	sort_entry_t *entries = (sort_entry_t *)malloc(
		(size_t)count * sizeof(sort_entry_t));
	if (!entries)
		return;

	uint32_t n = 0;
	uint32_t cur = qmap_iter(input->row_hd, NULL, 0);
	const void *k;
	const void *v;
	int all_num = 1;

	while (qmap_next(&k, &v, cur)) {
		entries[n].id = (const char *)k;
		entries[n].str_val = row_field_val(input,
			entries[n].id, sort_field);

		if (entries[n].str_val && *entries[n].str_val) {
			char *end = NULL;
			entries[n].num_val = strtod(
				entries[n].str_val, &end);
			entries[n].is_num = (end && *end == '\0');
			if (!entries[n].is_num)
				all_num = 0;
		} else {
			entries[n].num_val = 0.0;
			entries[n].is_num = 0;
		}
		n++;
	}
	qmap_fin(cur);

	if (!all_num) {
		for (uint32_t i = 0; i < n; i++)
			entries[i].is_num = 0;
	}

	qsort(entries, n, sizeof(sort_entry_t),
		sort_asc ? sort_cmp_asc : sort_cmp_desc);

	for (uint32_t i = 0; i < n; i++)
		qmap_put(output->row_hd, entries[i].id, "");

	free(entries);
}

/* ---- hyle_paginate ---- */

void hyle_paginate(hyle_ctx_t *ctx,
	const hyle_row_set_t *input,
	uint32_t page,
	uint32_t per_page,
	hyle_row_set_t *output,
	uint32_t *total_out)
{
	(void)ctx;
	output->fields_hd = input->fields_hd;

	uint32_t total = qmap_count(input->row_hd, NULL);
	if (total_out)
		*total_out = total;

	if (page == 0 || per_page == 0) {
		uint32_t cur = qmap_iter(input->row_hd, NULL, 0);
		const void *k;
		const void *v;
		while (qmap_next(&k, &v, cur)) {
			qmap_put(output->row_hd, (const char *)k, "");
		}
		qmap_fin(cur);
		return;
	}

	uint32_t skip = (page - 1) * per_page;
	if (skip >= total)
		return;

	uint32_t remain = total - skip;
	uint32_t take = remain < per_page ? remain : per_page;

	uint32_t pos = 0;
	uint32_t emitted = 0;
	uint32_t cur = qmap_iter(input->row_hd, NULL, 0);
	const void *k;
	const void *v;

	while (qmap_next(&k, &v, cur) && emitted < take) {
		if (pos >= skip) {
			qmap_put(output->row_hd, (const char *)k, "");
			emitted++;
		}
		pos++;
	}
	qmap_fin(cur);
}

/* ---- hyle_apply_view ---- */

static unsigned open_row_hd(void)
{
	return qmap_open(NULL, NULL, QM_STR, QM_STR, 0xFF, 0);
}

static void copy_row_hd(unsigned dst, unsigned src)
{
	uint32_t cur = qmap_iter(src, NULL, 0);
	const void *k;
	const void *v;
	while (qmap_next(&k, &v, cur)) {
		qmap_put(dst, (const char *)k, "");
	}
	qmap_fin(cur);
}

void hyle_apply_view(hyle_ctx_t *ctx,
	const hyle_row_set_t *input,
	const hyle_query_t *query,
	const hyle_field_t *fields,
	size_t field_count,
	hyle_row_set_t *output,
	uint32_t *total_out)
{
	hyle_row_set_t filtered;
	hyle_row_set_t sorted;
	unsigned filtered_hd = 0;
	unsigned sorted_hd = 0;
	int close_filtered = 0;
	int close_sorted = 0;

	if ((query->q && *query->q) || query->filter_count > 0) {
		filtered_hd = open_row_hd();
		filtered.row_hd = filtered_hd;
		filtered.fields_hd = input->fields_hd;
		hyle_filter_rows(ctx, input, query->q,
			query->filters, query->filter_count,
			fields, field_count, &filtered);
		close_filtered = 1;
	} else {
		filtered = *input;
	}

	if (query->sort_field) {
		sorted_hd = open_row_hd();
		sorted.row_hd = sorted_hd;
		sorted.fields_hd = filtered.fields_hd;
		hyle_sort_rows(ctx, &filtered, query->sort_field,
			query->sort_asc, &sorted);
		close_sorted = 1;
	} else {
		sorted = filtered;
	}

	uint32_t total = qmap_count(sorted.row_hd, NULL);
	if (total_out)
		*total_out = total;

	if (query->page > 0 && query->per_page > 0) {
		output->row_hd = open_row_hd();
		output->fields_hd = input->fields_hd;
		hyle_paginate(ctx, &sorted, query->page,
			query->per_page, output, NULL);
	} else {
		output->row_hd = open_row_hd();
		output->fields_hd = input->fields_hd;
		if (close_sorted)
			copy_row_hd(output->row_hd, sorted.row_hd);
		else if (close_filtered)
			copy_row_hd(output->row_hd, filtered.row_hd);
		else
			copy_row_hd(output->row_hd, input->row_hd);
	}

	if (close_sorted && sorted_hd)
		qmap_close(sorted_hd);
	if (close_filtered && filtered_hd)
		qmap_close(filtered_hd);
}
