#include "hyle-source/hyle_source.h"
#include "source_utils.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <ttypt/qmap.h>
#include <hyle/source.h>

int hyle_source_get_display_field(const char *dataset_id, char *out, size_t sz)
{
	unsigned schema_hd;
	uint32_t cur;
	const void *key;
	const void *val;
	hyle_source_def_t *def;

	if (!out || sz == 0)
		return -1;
	out[0] = '\0';
	if (!dataset_id || !dataset_id[0])
		return -1;

	def = hyle_source_find(dataset_id);
	if (def && def->key_field && def->key_field[0] && strcmp(def->key_field, "id") != 0) {
		snprintf(out, sz, "%s", def->key_field);
		return 0;
	}

	if (def && def->fields && def->field_count > 0) {
		for (size_t i = 0; i < def->field_count; i++) {
			const hyle_source_field_t *f = &def->fields[i];
			if (strcmp(f->name, "id") == 0 || f->type == HYLE_SOURCE_FIELD_INVERSE)
				continue;
			if (f->type == HYLE_SOURCE_FIELD_STRING || f->type == HYLE_SOURCE_FIELD_NULLABLE_STRING) {
				snprintf(out, sz, "%s", f->name);
				return 0;
			}
		}
	}

	schema_hd = hyle_source_get_schema_hd(dataset_id);
	if (!schema_hd)
		return -1;

	cur = qmap_iter(schema_hd, NULL, 0);
	while (qmap_next(&key, &val, cur)) {
		if (strcmp((const char *)key, "id") == 0)
			continue;
		if (val && strstr((const char *)val, "\"i\":"))
			continue;
		snprintf(out, sz, "%s", (const char *)key);
		break;
	}
	qmap_fin(cur);
	return out[0] ? 0 : -1;
}

const char *hyle_source_get_item_label(
        const char *dataset_id, const char *row_id, const char *display_field,
        char *out, size_t sz)
{
	unsigned fields_hd;
	const char *name = NULL;
	char df_buf[64] = { 0 };

	if (!row_id)
		return "";

	fields_hd = hyle_source_get_fields_hd(dataset_id);
	if (!fields_hd) {
		if (out && sz)
			snprintf(out, sz, "%s", row_id);
		return row_id;
	}

	if (!display_field || !display_field[0]) {
		hyle_source_get_display_field(dataset_id, df_buf, sizeof(df_buf));
		display_field = df_buf;
	}

	if (display_field && display_field[0]) {
		char name_key[320];
		snprintf(name_key, sizeof(name_key), "%s:%s", row_id, display_field);
		name = (const char *)qmap_get(fields_hd, name_key);
	}

	const char *res = name ? name : row_id;
	if (out && sz)
		snprintf(out, sz, "%s", res);
	return res;
}

int hyle_source_resolve_options(
        const char *dataset_id, const char *q, int page0, int per_page,
        hyle_option_t *opts, int max, int *total_out,
        char (*id_buf)[64], char (*label_buf)[256])
{
	char qs[1024];
	char display_field[64];
	unsigned result_hd;
	const char *total_str;
	uint32_t cur;
	const void *rkey;
	const void *rval;
	int n = 0;

	if (total_out)
		*total_out = 0;
	if (!dataset_id || !dataset_id[0] || max <= 0 || !opts || !id_buf || !label_buf)
		return 0;

	if (q && q[0]) {
		snprintf(
		        qs, sizeof(qs), "q=%s&page=%d&per_page=%d",
		        q, page0 + 1, per_page > 0 ? per_page : 15);
	} else {
		snprintf(
		        qs, sizeof(qs), "page=%d&per_page=%d",
		        page0 + 1, per_page > 0 ? per_page : 15);
	}

	result_hd = hyle_source_query_dataset(dataset_id, qs);
	if (!result_hd)
		return 0;

	total_str = (const char *)qmap_get(result_hd, "__total__");
	if (total_out)
		*total_out = total_str ? atoi(total_str) : 0;

	hyle_source_get_display_field(dataset_id, display_field, sizeof(display_field));

	cur = qmap_iter(result_hd, NULL, 0);
	while (n < max && qmap_next(&rkey, &rval, cur)) {
		const char *row_id;
		const char *name;

		if (strcmp((const char *)rkey, "__total__") == 0)
			continue;
		row_id = (const char *)rkey;
		snprintf(id_buf[n], sizeof(id_buf[n]), "%s", row_id);
		name = hyle_source_get_item_label(
		        dataset_id, row_id, display_field, label_buf[n], sizeof(label_buf[n]));
		opts[n].id = id_buf[n];
		opts[n].label = name;
		n++;
	}
	qmap_fin(cur);
	qmap_close(result_hd);
	return n;
}

int hyle_source_resolve_tokens(
        const char *dataset_id, const char *comma_slugs, hyle_option_t *out,
        int max, char (*id_buf)[64], char (*label_buf)[256])
{
	char display_field[64];
	unsigned fields_hd;
	const char *p = comma_slugs;
	int n = 0;

	if (!dataset_id || !dataset_id[0] || !p || !p[0] || max <= 0 || !out || !id_buf || !label_buf)
		return 0;

	fields_hd = hyle_source_get_fields_hd(dataset_id);
	if (!fields_hd)
		return 0;

	hyle_source_get_display_field(dataset_id, display_field, sizeof(display_field));

	while (*p && n < max) {
		const char *comma = strpbrk(p, ",\n\r");
		size_t len = comma ? (size_t)(comma - p) : strlen(p);
		char token[128];
		const char *slug = NULL;

		if (len >= sizeof(token))
			len = sizeof(token) - 1;
		memcpy(token, p, len);
		token[len] = '\0';
		char *ttrim = token;
		while (*ttrim == ' ' || *ttrim == '\t')
			ttrim++;
		size_t tlen = strlen(ttrim);
		while (tlen > 0 &&
		       (ttrim[tlen - 1] == ' ' || ttrim[tlen - 1] == '\t' ||
		        ttrim[tlen - 1] == '\r'))
			ttrim[--tlen] = '\0';

		if (ttrim[0]) {
			if (strspn(ttrim, "0123456789") == strlen(ttrim)) {
				slug = qmap_get_key(
				        fields_hd, (uint32_t)atoi(ttrim));
				if (slug && !slug[0])
					slug = NULL;
			}
			if (!slug) {
				char slug_buf[128];
				source_util_slugify(
				        ttrim, strlen(ttrim), slug_buf,
				        sizeof(slug_buf));
				if (slug_buf[0] &&
				    qmap_pos(fields_hd, slug_buf) != UINT32_MAX)
					slug = qmap_get_key(
					        fields_hd,
					        qmap_pos(fields_hd, slug_buf));
			}
			if (!slug) {
				if (qmap_pos(fields_hd, ttrim) != UINT32_MAX)
					slug = qmap_get_key(
					        fields_hd,
					        qmap_pos(fields_hd, ttrim));
			}
			if (!slug) {
				char slug_buf[128];
				source_util_slugify(
				        ttrim, strlen(ttrim), slug_buf,
				        sizeof(slug_buf));
				if (slug_buf[0])
					slug = slug_buf;
			}
			if (!slug)
				slug = ttrim;

			snprintf(id_buf[n], sizeof(id_buf[n]), "%.60s", slug);
			const char *name = hyle_source_get_item_label(
			        dataset_id, slug, display_field, label_buf[n], sizeof(label_buf[n]));
			out[n].id = id_buf[n];
			out[n].label = name;
			n++;
		}
		if (!comma)
			break;
		while (*comma && (*comma == ',' || *comma == '\n' ||
		                  *comma == '\r' || *comma == ' '))
			comma++;
		p = comma;
	}
	return n;
}

int hyle_source_normalize_tokens_to_slugs(
        const char *dataset_id, const char *raw, char *out, size_t out_sz)
{
	char display_field[64];
	unsigned fields_hd;
	const char *p = raw;
	size_t off = 0;

	if (!out || out_sz == 0)
		return -1;
	out[0] = '\0';
	if (!dataset_id || !raw || !raw[0])
		return 0;

	fields_hd = hyle_source_get_fields_hd(dataset_id);
	if (!fields_hd)
		return -1;

	hyle_source_get_display_field(dataset_id, display_field, sizeof(display_field));

	while (*p && off + 2 < out_sz) {
		const char *end = strpbrk(p, "\r\n,");
		size_t len = end ? (size_t)(end - p) : strlen(p);
		char token[128];
		const char *slug = NULL;

		if (len >= sizeof(token))
			len = sizeof(token) - 1;
		memcpy(token, p, len);
		token[len] = '\0';
		char *ttrim = token;
		while (*ttrim == ' ' || *ttrim == '\t')
			ttrim++;
		size_t tlen = strlen(ttrim);
		while (tlen > 0 &&
		       (ttrim[tlen - 1] == ' ' || ttrim[tlen - 1] == '\t' ||
		        ttrim[tlen - 1] == '\r'))
			ttrim[--tlen] = '\0';

		if (ttrim[0]) {
			if (strspn(ttrim, "0123456789") == strlen(ttrim)) {
				slug = qmap_get_key(
				        fields_hd, (uint32_t)atoi(ttrim));
				if (slug && !slug[0])
					slug = NULL;
			}
			/* Fallback reverse-lookup by display name */
			if (!slug && display_field[0]) {
				char slug_buf[128];
				source_util_slugify(
				        ttrim, strlen(ttrim), slug_buf,
				        sizeof(slug_buf));
				if (slug_buf[0] &&
				    qmap_pos(fields_hd, slug_buf) != UINT32_MAX)
				{
					slug = qmap_get_key(
					        fields_hd,
					        qmap_pos(fields_hd, slug_buf));
				} else {
					unsigned result_hd;
					char qs[512];
					snprintf(
					        qs, sizeof(qs), "%s=%s",
					        display_field, ttrim);
					result_hd = hyle_source_query_dataset(dataset_id, qs);
					if (result_hd) {
						uint32_t cur = qmap_iter(result_hd, NULL, 0);
						const void *rk, *rv;
						while (qmap_next(&rk, &rv, cur)) {
							if (strcmp((const char *)rk, "__total__") != 0) {
								slug = (const char *)rk;
								break;
							}
						}
						qmap_fin(cur);
						qmap_close(result_hd);
					}
				}
			}
			if (!slug)
				slug = ttrim;
			if (off > 0)
				out[off++] = ',';
			snprintf(out + off, out_sz - off, "%.60s", slug);
			off = strlen(out);
		}
		if (!end)
			break;
		p = end + 1;
	}
	return 0;
}

int hyle_source_get_enum_options(
        const char *dataset_id, hyle_option_t *pool, int pool_avail,
        char (*id_buf)[64], char (*label_buf)[256])
{
	unsigned row_hd;
	char display_field[64] = "";
	int nopts = 0;
	uint32_t cur;
	const void *key;
	const void *val;

	if (!dataset_id || !dataset_id[0] || !pool || pool_avail <= 0 || !id_buf || !label_buf)
		return 0;

	row_hd = hyle_source_get_data_hd(dataset_id);
	if (!row_hd)
		return 0;

	hyle_source_get_display_field(dataset_id, display_field, sizeof(display_field));

	cur = qmap_iter(row_hd, NULL, 0);
	while (qmap_next(&key, &val, cur) && nopts < pool_avail) {
		const char *row_id = (const char *)key;
		snprintf(id_buf[nopts], sizeof(id_buf[nopts]), "%s", row_id);
		const char *name = hyle_source_get_item_label(
		        dataset_id, row_id, display_field, label_buf[nopts], sizeof(label_buf[nopts]));
		pool[nopts].id = id_buf[nopts];
		pool[nopts].label = name;
		nopts++;
	}
	qmap_fin(cur);

	return nopts;
}
