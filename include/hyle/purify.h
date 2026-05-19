#ifndef HYLE_PURIFY_H
#define HYLE_PURIFY_H

#include <stddef.h>
#include <stdint.h>
#include "field.h"

typedef struct {
	const char *field;
	const char *rule;
	const char *message;
} hyle_purify_error_t;

int hyle_purify_row(
	const hyle_field_t *fields,
	size_t field_count,
	const char **values,
	hyle_purify_error_t **errors_out,
	size_t *error_count_out);

void hyle_purify_errors_free(
	hyle_purify_error_t *errors,
	size_t error_count);

#endif
