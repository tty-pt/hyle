#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "hyle/value.h"
#include <ttypt/qmap.h>

/* ---- String helpers ---- */

static char *id_key(char *buf, size_t sz, uint32_t id)
{
	snprintf(buf, sz, "%u", id);
	return buf;
}

static char *map_key(char *buf, size_t sz, uint32_t id, const char *field)
{
	snprintf(buf, sz, "%u.%s", id, field);
	return buf;
}

/* ---- Constructors ---- */

hyle_val_t hyle_val_null(void)
{
	return (hyle_val_t){ .type = HYLE_NULL };
}

hyle_val_t hyle_val_bool(bool b)
{
	return (hyle_val_t){ .type = HYLE_BOOL, .b = b };
}

hyle_val_t hyle_val_int(int64_t i)
{
	return (hyle_val_t){ .type = HYLE_INT, .i = i };
}

hyle_val_t hyle_val_float(double f)
{
	return (hyle_val_t){ .type = HYLE_FLOAT, .f = f };
}

hyle_val_t hyle_val_string(hyle_ctx_t *ctx, const char *s)
{
	uint32_t id = ctx->next_id++;
	qmap_put(ctx->string_pool, &id, s);
	return (hyle_val_t){ .type = HYLE_STRING, .hd = id };
}

hyle_val_t hyle_val_array(hyle_ctx_t *ctx)
{
	uint32_t id = ctx->next_id++;
	return (hyle_val_t){ .type = HYLE_ARRAY, .hd = id };
}

hyle_val_t hyle_val_map(hyle_ctx_t *ctx)
{
	uint32_t id = ctx->next_id++;
	return (hyle_val_t){ .type = HYLE_MAP, .hd = id };
}

/* ---- Accessors ---- */

const char *hyle_val_string_get(hyle_ctx_t *ctx, hyle_val_t v)
{
	if (v.type != HYLE_STRING)
		return NULL;
	return (const char *)qmap_get(ctx->string_pool, &v.hd);
}

/* ---- Array operations ---- */

void hyle_val_array_push(hyle_ctx_t *ctx, hyle_val_t arr, hyle_val_t elem)
{
	char *json = hyle_val_to_json(ctx, elem);
	if (!json)
		return;
	char key[32];
	id_key(key, sizeof(key), arr.hd);
	qmap_put(ctx->array_pool, key, json);
	free(json);
}

hyle_val_t hyle_val_array_get(hyle_ctx_t *ctx, hyle_val_t arr, size_t idx)
{
	char key[32];
	id_key(key, sizeof(key), arr.hd);

	uint32_t cur = qmap_get_multi(ctx->array_pool, key);
	if (cur == QM_MISS)
		return hyle_val_null();

	const void *k, *val;
	size_t i = 0;
	while (qmap_next(&k, &val, cur)) {
		if (i == idx) {
			const char *s = (const char *)val;
			qmap_fin(cur);
			return hyle_val_string(ctx, s);
		}
		i++;
	}
	qmap_fin(cur);
	return hyle_val_null();
}

size_t hyle_val_array_len(hyle_ctx_t *ctx, hyle_val_t arr)
{
	char key[32];
	id_key(key, sizeof(key), arr.hd);
	return qmap_count(ctx->array_pool, key);
}

/* ---- Map operations ---- */

void hyle_val_map_set(hyle_ctx_t *ctx, hyle_val_t map,
	const char *field, hyle_val_t val)
{
	char *json = hyle_val_to_json(ctx, val);
	if (!json)
		return;
	char key[256];
	map_key(key, sizeof(key), map.hd, field);
	qmap_put(ctx->map_pool, key, json);
	free(json);
}

hyle_val_t hyle_val_map_get(hyle_ctx_t *ctx, hyle_val_t map,
	const char *field)
{
	char key[256];
	map_key(key, sizeof(key), map.hd, field);
	const char *val = (const char *)qmap_get(ctx->map_pool, key);
	if (!val)
		return hyle_val_null();
	return hyle_val_string(ctx, val);
}

/* ---- JSON serialization ---- */

static char *json_escape(const char *s)
{
	size_t len = strlen(s);
	size_t cap = len * 2 + 3;
	char *out = (char *)malloc(cap);
	if (!out)
		return NULL;

	out[0] = '"';
	size_t j = 1;
	for (size_t i = 0; s[i]; i++) {
		unsigned char c = (unsigned char)s[i];
		switch (c) {
		case '"':  out[j++] = '\\'; out[j++] = '"';  break;
		case '\\': out[j++] = '\\'; out[j++] = '\\'; break;
		case '\n': out[j++] = '\\'; out[j++] = 'n';  break;
		case '\t': out[j++] = '\\'; out[j++] = 't';  break;
		case '\r': out[j++] = '\\'; out[j++] = 'r';  break;
		default:
			if (c < 0x20) {
				char buf[8];
				snprintf(buf, sizeof(buf), "\\u%04x", c);
				for (int k = 0; buf[k]; k++)
					out[j++] = buf[k];
			} else {
				out[j++] = c;
			}
			break;
		}
	}
	out[j++] = '"';
	out[j] = '\0';
	return out;
}

char *hyle_val_to_json(hyle_ctx_t *ctx, hyle_val_t v)
{
	char buf[128];

	switch (v.type) {
	case HYLE_NULL:
		return strdup("null");

	case HYLE_BOOL:
		return strdup(v.b ? "true" : "false");

	case HYLE_INT:
		snprintf(buf, sizeof(buf), "%ld", (long)v.i);
		return strdup(buf);

	case HYLE_FLOAT:
		snprintf(buf, sizeof(buf), "%g", v.f);
		return strdup(buf);

	case HYLE_STRING: {
		const char *raw = hyle_val_string_get(ctx, v);
		if (!raw)
			return strdup("null");
		return json_escape(raw);
	}

	case HYLE_ARRAY: {
		char key[32];
		id_key(key, sizeof(key), v.hd);

		uint32_t cur = qmap_get_multi(ctx->array_pool, key);
		if (cur == QM_MISS)
			return strdup("[]");

		size_t cap = 256;
		size_t len = 0;
		char *json = (char *)malloc(cap);
		if (!json) { qmap_fin(cur); return NULL; }
		json[len++] = '[';

		const void *k, *val;
		int first = 1;
		while (qmap_next(&k, &val, cur)) {
			if (!first) {
				if (len + 1 >= cap) {
					cap *= 2;
					char *tmp = (char *)realloc(json, cap);
					if (!tmp) { free(json); qmap_fin(cur); return NULL; }
					json = tmp;
				}
				json[len++] = ',';
			}
			first = 0;
			const char *elem = (const char *)val;
			size_t elen = strlen(elem);
			while (len + elen + 1 >= cap) {
				cap *= 2;
				char *tmp = (char *)realloc(json, cap);
				if (!tmp) { free(json); qmap_fin(cur); return NULL; }
				json = tmp;
			}
			memcpy(json + len, elem, elen);
			len += elen;
		}
		qmap_fin(cur);
		json[len++] = ']';
		json[len] = '\0';
		return json;
	}

	case HYLE_MAP: {
		size_t cap = 256;
		size_t len = 0;
		char *json = (char *)malloc(cap);
		if (!json) return NULL;
		json[len++] = '{';

		int first = 1;
		uint32_t cur = qmap_iter(ctx->map_pool, NULL, 0);

		const void *k, *vval;
		char prefix[32];
		snprintf(prefix, sizeof(prefix), "%u.", v.hd);
		size_t plen = strlen(prefix);

		while (qmap_next(&k, &vval, cur)) {
			const char *key = (const char *)k;
			if (strncmp(key, prefix, plen) != 0)
				continue;

			if (!first) {
				if (len + 1 >= cap) {
					cap *= 2;
					char *tmp = (char *)realloc(json, cap);
					if (!tmp) { free(json); qmap_fin(cur); return NULL; }
					json = tmp;
				}
				json[len++] = ',';
			}
			first = 0;

			const char *field = key + plen;
			char *esc_field = json_escape(field);
			const char *esc_val = (const char *)vval;
			size_t flen = strlen(esc_field);
			size_t vlen = strlen(esc_val);
			size_t need = flen + vlen + 2;
			while (len + need + 1 >= cap) {
				cap *= 2;
				char *tmp = (char *)realloc(json, cap);
				if (!tmp) { free(esc_field); qmap_fin(cur); free(json); return NULL; }
				json = tmp;
			}
			memcpy(json + len, esc_field, flen);
			len += flen;
			json[len++] = ':';
			memcpy(json + len, esc_val, vlen);
			len += vlen;
			free(esc_field);
		}
		qmap_fin(cur);
		json[len++] = '}';
		json[len] = '\0';
		return json;
	}

	default:
		return strdup("null");
	}
}
