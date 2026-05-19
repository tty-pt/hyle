#include "hyle/blueprint.h"
#include <string.h>
#include <stdlib.h>

void hyle_manifest_clear(hyle_manifest_t *m)
{
	if (!m)
		return;
	free((void *)m->select);
	free(m->filter_list);
	free((void *)m->lookups);
	free((void *)m->inlines);
	memset(m, 0, sizeof(*m));
}

int hyle_blueprint_manifest(
	const hyle_blueprint_t *bp,
	const char *source_id,
	const hyle_query_t *query,
	hyle_manifest_t *out)
{
	memset(out, 0, sizeof(*out));

	if (!bp || !source_id || !query)
		return -1;

	const hyle_source_schema_t *schema = NULL;
	for (size_t i = 0; i < bp->source_count; i++) {
		if (strcmp(bp->sources[i].id, source_id) == 0) {
			schema = &bp->sources[i];
			break;
		}
	}
	if (!schema)
		return -1;

	out->base_source = schema->id;

	if (query->filter_count > 0) {
		out->filter_list = (hyle_manifest_filter_t *)calloc(
			query->filter_count, sizeof(hyle_manifest_filter_t));
		if (!out->filter_list)
			return -1;

		for (unsigned i = 0; i < query->filter_count; i++) {
			const hyle_field_t *f = hyle_field_by_name(
				schema->fields, schema->field_count,
				query->filters[i].field);
			if (!f) {
				hyle_manifest_clear(out);
				return -1;
			}
			out->filter_list[i].field = query->filters[i].field;
			out->filter_list[i].value = query->filters[i].value;
			out->filter_list[i].is_reference =
				hyle_field_is_reference(f->type);
			out->filter_count++;
		}
	}

	if (query->include_count > 0) {
		out->select = (const char **)calloc(
			query->include_count, sizeof(char *));
		if (!out->select) {
			hyle_manifest_clear(out);
			return -1;
		}
		for (unsigned i = 0; i < query->include_count; i++) {
			const hyle_field_t *f = hyle_field_by_name(
				schema->fields, schema->field_count,
				query->include[i]);
			if (!f) {
				hyle_manifest_clear(out);
				return -1;
			}
			out->select[i] = query->include[i];
			out->select_count++;

			if (hyle_field_is_reference(f->type)
			    && f->target_source) {
				int found = 0;
				for (size_t j = 0; j < out->inline_count; j++) {
					if (strcmp(out->inlines[j],
						f->target_source) == 0) {
						found = 1;
						break;
					}
				}
				if (!found) {
					const char **tmp = (const char **)realloc(
						(void *)out->inlines,
						(out->inline_count + 1)
						    * sizeof(char *));
					if (!tmp) {
						hyle_manifest_clear(out);
						return -1;
					}
					out->inlines = tmp;
					out->inlines[out->inline_count++]
						= f->target_source;
				}
			}
		}
	}

	if (query->sort_field) {
		const hyle_field_t *f = hyle_field_by_name(
			schema->fields, schema->field_count,
			query->sort_field);
		if (!f) {
			hyle_manifest_clear(out);
			return -1;
		}
		out->sort_field = query->sort_field;
		out->sort_asc = query->sort_asc;
	}

	out->query = query->q;
	out->page = query->page;
	out->per_page = query->per_page;

	for (unsigned i = 0; i < out->filter_count; i++) {
		if (!out->filter_list[i].is_reference)
			continue;

		const hyle_field_t *f = hyle_field_by_name(
			schema->fields, schema->field_count,
			out->filter_list[i].field);
		if (!f || !f->target_source)
			continue;

		int found = 0;
		for (size_t j = 0; j < out->lookup_count; j++) {
			if (strcmp(out->lookups[j], f->target_source) == 0) {
				found = 1;
				break;
			}
		}
		if (!found) {
			const char **tmp = (const char **)realloc(
				(void *)out->lookups,
				(out->lookup_count + 1) * sizeof(char *));
			if (!tmp) {
				hyle_manifest_clear(out);
				return -1;
			}
			out->lookups = tmp;
			out->lookups[out->lookup_count++] = f->target_source;
		}
	}

	return 0;
}
