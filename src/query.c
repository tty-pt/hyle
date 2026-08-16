#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include "hyle/query.h"

static void url_decode(char *s)
{
	char *d = s;
	while (*s) {
		if (*s == '%' && s[1] && s[2]) {
			char hex[3] = { s[1], s[2], 0 };
			*d++ = (char)strtol(hex, NULL, 16);
			s += 3;
		} else if (*s == '+') {
			*d++ = ' ';
			s++;
		} else {
			*d++ = *s++;
		}
	}
	*d = '\0';
}

static int count_params(const char *s)
{
	if (!s || !*s)
		return 0;
	int n = 1;
	while (*s) {
		if (*s == '&')
			n++;
		s++;
	}
	return n;
}

int hyle_parse_query(char *query_str, hyle_query_t *out)
{
	out->sort_field = NULL;
	out->sort_asc = 1;
	out->page = 0;
	out->per_page = 0;
	out->q = NULL;
	out->filters = NULL;
	out->filter_count = 0;
	out->include = NULL;
	out->include_count = 0;

	if (!query_str || !*query_str)
		return 0;

	int max = count_params(query_str);
	if (max < 1)
		return 0;

	char **names = (char **)calloc((size_t)max, sizeof(char *));
	char **values = (char **)calloc((size_t)max, sizeof(char *));
	if (!names || !values) {
		free(names);
		free(values);
		return -1;
	}

	int n = 0;
	char *p = query_str;
	while (p && *p && n < max) {
		char *amp = strchr(p, '&');
		char *part = p;
		if (amp) {
			*amp = '\0';
			p = amp + 1;
		} else {
			p = p + strlen(p);
		}

		char *eq = strchr(part, '=');
		if (eq) {
			*eq = '\0';
			names[n] = part;
			values[n] = eq + 1;
		} else {
			names[n] = part;
			values[n] = NULL;
		}
		n++;
	}

	out->filter_count = 0;
	out->include_count = 0;

	/* Collect _op overrides before dispatching filters */
	char op_field[64][64];
	const char *op_val[64];
	int n_ops = 0;

	for (int i = 0; i < n; i++) {
		char *name = names[i];
		char *value = values[i];

		if (!value)
			continue;

		url_decode(value);

		/* Intercept <field>_op — always consumed (never a filter).
		 * Only valid values ("and"/"or") are recorded as overrides. */
		{
			size_t nlen = strlen(name);
			if (nlen > 3 && strcmp(name + nlen - 3, "_op") == 0) {
				if ((strcmp(value, "and") == 0 ||
				     strcmp(value, "or") == 0) &&
				    n_ops < 64) {
					size_t flen = nlen - 3;
					if (flen >= 64)
						flen = 63;
					memcpy(op_field[n_ops], name, flen);
					op_field[n_ops][flen] = '\0';
					op_val[n_ops] = value;
					n_ops++;
				}
				continue; /* NOT a filter regardless of value */
			}
		}

		if (strcmp(name, "sort") == 0) {
			char *colon = strchr(value, ':');
			if (colon) {
				*colon = '\0';
				out->sort_field = value;
				out->sort_asc = (strcmp(colon + 1, "desc") != 0);
			} else {
				out->sort_field = value;
			}
		} else if (strcmp(name, "page") == 0) {
			out->page = (uint32_t)atol(value);
		} else if (strcmp(name, "per_page") == 0) {
			out->per_page = (uint32_t)atol(value);
		} else if (strcmp(name, "q") == 0) {
			out->q = value;
		} else if (strcmp(name, "include") == 0) {
			char *field = value;
			while (field) {
				char *comma = strchr(field, ',');
				if (comma)
					*comma = '\0';
				url_decode(field);

				const char **tmp = (const char **)realloc(
					(void *)out->include,
					(out->include_count + 1) * sizeof(char *));
				if (!tmp)
					goto nomem;
				out->include = tmp;
				out->include[out->include_count++] = field;

				if (comma)
					field = comma + 1;
				else
					break;
			}
		} else {
			hyle_field_filter_t *tmp = (hyle_field_filter_t *)realloc(
				(void *)out->filters,
				(out->filter_count + 1) * sizeof(hyle_field_filter_t));
			if (!tmp)
				goto nomem;
			out->filters = tmp;
			out->filters[out->filter_count].field = name;
			out->filters[out->filter_count].value = value;
			out->filters[out->filter_count].op = NULL;
			out->filter_count++;
		}
	}

	/* Apply _op overrides to matching filters (last-wins, order-independent) */
	for (unsigned i = 0; i < out->filter_count; i++) {
		for (int j = 0; j < n_ops; j++) {
			if (strcmp(out->filters[i].field, op_field[j]) == 0)
				out->filters[i].op = op_val[j];
		}
	}

	free(names);
	free(values);
	return 0;

nomem:
	hyle_query_clear(out);
	free(names);
	free(values);
	return -1;
}

void hyle_query_clear(hyle_query_t *q)
{
	if (!q)
		return;
	free((void *)q->include);
	free((void *)q->filters);
	q->include = NULL;
	q->filters = NULL;
	q->include_count = 0;
	q->filter_count = 0;
}
