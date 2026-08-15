#ifndef HYLE_FIELD_H
#define HYLE_FIELD_H

#include <stddef.h>
#include <stdint.h>

typedef enum {
	HYLE_FIELD_STRING = 0,
	HYLE_FIELD_INT,
	HYLE_FIELD_BOOL,
	HYLE_FIELD_NULLABLE_STRING,
	HYLE_FIELD_REFERENCE,
	HYLE_FIELD_MULTI_REFERENCE,
	HYLE_FIELD_INVERSE,
} hyle_field_type_t;

typedef struct {
	const char *name;
	hyle_field_type_t type;
	int writable;
	const char *target_source;
	const char *inverse_name;
	int required;
	int64_t min;
	int64_t max;
	size_t min_length;
	size_t max_length;
	const char *pattern;
	int searchable;
} hyle_field_t;

const hyle_field_t *hyle_field_by_name(
	const hyle_field_t *fields,
	size_t field_count,
	const char *name);

int hyle_field_is_reference(hyle_field_type_t type);

#endif
