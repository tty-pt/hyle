#ifndef HYLE_BLUEPRINT_H
#define HYLE_BLUEPRINT_H

#include <stddef.h>
#include <stdint.h>
#include "field.h"
#include "query.h"

typedef struct {
	const char *id;
	const char *key_field;
	const hyle_field_t *fields;
	size_t field_count;
	uint32_t fields_hd;
} hyle_source_schema_t;

typedef struct {
	hyle_source_schema_t *sources;
	size_t source_count;
} hyle_blueprint_t;

typedef struct {
	const char *field;
	const char *value;
	int is_reference;
} hyle_manifest_filter_t;

typedef struct {
	const char *base_source;
	const char **select;
	size_t select_count;
	hyle_manifest_filter_t *filter_list;
	size_t filter_count;
	const char *query;
	const char *sort_field;
	int sort_asc;
	uint32_t page;
	uint32_t per_page;
	const char **lookups;
	size_t lookup_count;
	const char **inlines;
	size_t inline_count;
} hyle_manifest_t;

int hyle_blueprint_manifest(
	const hyle_blueprint_t *bp,
	const char *source_id,
	const hyle_query_t *query,
	hyle_manifest_t *out);

void hyle_manifest_clear(hyle_manifest_t *m);

#endif
