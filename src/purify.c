#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <regex.h>

#include "hyle/purify.h"

static int is_string_type(hyle_field_type_t t)
{
	return t == HYLE_FIELD_STRING ||
	       t == HYLE_FIELD_NULLABLE_STRING;
}

static int is_numeric_type(hyle_field_type_t t)
{
	return t == HYLE_FIELD_INT;
}

static int value_is_present(const char *v)
{
	return v && v[0];
}

static int64_t parse_int64(const char *s, int *ok)
{
	char *end;
	long long val;

	if (!s || !s[0]) {
		*ok = 0;
		return 0;
	}
	end = NULL;
	val = strtoll(s, &end, 10);
	if (*end) {
		*ok = 0;
		return 0;
	}
	*ok = 1;
	return (int64_t)val;
}

int hyle_purify_row(
	const hyle_field_t *fields,
	size_t field_count,
	const char **values,
	hyle_purify_error_t **errors_out,
	size_t *error_count_out)
{
	size_t cap;
	size_t nerr;
	hyle_purify_error_t *errs;
	size_t i;

	if (!fields || field_count == 0 || !errors_out || !error_count_out) {
		if (errors_out) *errors_out = NULL;
		if (error_count_out) *error_count_out = 0;
		return 0;
	}

	*errors_out = NULL;
	*error_count_out = 0;

	cap = field_count;
	errs = malloc(cap * sizeof(hyle_purify_error_t));
	if (!errs)
		return 1;
	nerr = 0;

	for (i = 0; i < field_count; i++) {
		const hyle_field_t *f = &fields[i];
		const char *val = values ? values[i] : NULL;

		/* 1. required check */
		if (f->required && !value_is_present(val)) {
			char msg[256];
			snprintf(msg, sizeof(msg),
				"Field '%s' is required", f->name);
			errs[nerr].field = f->name;
			errs[nerr].rule = "required";
			errs[nerr].message = strdup(msg);
			if (!errs[nerr].message) {
				hyle_purify_errors_free(errs, nerr);
				return 1;
			}
			nerr++;
			continue;
		}

		/* If no value, skip remaining checks */
		if (!value_is_present(val))
			continue;

		/* 2. min/max (numeric) */
		if (is_numeric_type(f->type) && (f->min || f->max)) {
			int ok;
			int64_t nv = parse_int64(val, &ok);
			if (ok) {
				if (f->min && nv < f->min) {
					char msg[256];
					snprintf(msg, sizeof(msg),
						"Field '%s' must be >= %lld",
						f->name, (long long)f->min);
					errs[nerr].field = f->name;
					errs[nerr].rule = "min";
					errs[nerr].message = strdup(msg);
					if (!errs[nerr].message) {
						hyle_purify_errors_free(errs, nerr);
						return 1;
					}
					nerr++;
					continue;
				}
				if (f->max && nv > f->max) {
					char msg[256];
					snprintf(msg, sizeof(msg),
						"Field '%s' must be <= %lld",
						f->name, (long long)f->max);
					errs[nerr].field = f->name;
					errs[nerr].rule = "max";
					errs[nerr].message = strdup(msg);
					if (!errs[nerr].message) {
						hyle_purify_errors_free(errs, nerr);
						return 1;
					}
					nerr++;
					continue;
				}
			}
		}

		/* 3. min_length / max_length */
		if (is_string_type(f->type) &&
		    (f->min_length > 0 || f->max_length > 0))
		{
			size_t slen = strlen(val);
			if (f->min_length > 0 && slen < f->min_length) {
				char msg[256];
				snprintf(msg, sizeof(msg),
					"Field '%s' must be at least %zu "
					"characters",
					f->name, f->min_length);
				errs[nerr].field = f->name;
				errs[nerr].rule = "minLength";
				errs[nerr].message = strdup(msg);
				if (!errs[nerr].message) {
					hyle_purify_errors_free(errs, nerr);
					return 1;
				}
				nerr++;
				continue;
			}
			if (f->max_length > 0 && slen > f->max_length) {
				char msg[256];
				snprintf(msg, sizeof(msg),
					"Field '%s' must be at most %zu "
					"characters",
					f->name, f->max_length);
				errs[nerr].field = f->name;
				errs[nerr].rule = "maxLength";
				errs[nerr].message = strdup(msg);
				if (!errs[nerr].message) {
					hyle_purify_errors_free(errs, nerr);
					return 1;
				}
				nerr++;
				continue;
			}
		}

		/* 4. pattern (regex) */
		if (is_string_type(f->type) && f->pattern) {
			regex_t re;
			int rc;

			rc = regcomp(&re, f->pattern, REG_EXTENDED | REG_NOSUB);
			if (rc != 0) {
				/* Invalid pattern — report as error */
				char msg[256];
				snprintf(msg, sizeof(msg),
					"Invalid pattern '%s' for field '%s'",
					f->pattern, f->name);
				errs[nerr].field = f->name;
				errs[nerr].rule = "pattern";
				errs[nerr].message = strdup(msg);
				if (!errs[nerr].message) {
					hyle_purify_errors_free(errs, nerr);
					return 1;
				}
				nerr++;
				continue;
			}

			rc = regexec(&re, val, 0, NULL, 0);
			regfree(&re);
			if (rc != 0) {
				char msg[256];
				snprintf(msg, sizeof(msg),
					"Field '%s' does not match pattern "
					"'%s'",
					f->name, f->pattern);
				errs[nerr].field = f->name;
				errs[nerr].rule = "pattern";
				errs[nerr].message = strdup(msg);
				if (!errs[nerr].message) {
					hyle_purify_errors_free(errs, nerr);
					return 1;
				}
				nerr++;
				continue;
			}
		}
	}

	if (nerr == 0) {
		free(errs);
		*errors_out = NULL;
		*error_count_out = 0;
		return 0;
	}

	*errors_out = errs;
	*error_count_out = nerr;
	return 1;
}

void hyle_purify_errors_free(
	hyle_purify_error_t *errors,
	size_t error_count)
{
	size_t i;

	if (!errors)
		return;

	for (i = 0; i < error_count; i++)
		free((char *)errors[i].message);

	free(errors);
}
