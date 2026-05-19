#include "hyle/field.h"
#include <string.h>

const hyle_field_t *hyle_field_by_name(
	const hyle_field_t *fields,
	size_t field_count,
	const char *name)
{
	for (size_t i = 0; i < field_count; i++) {
		if (strcmp(fields[i].name, name) == 0)
			return &fields[i];
	}
	return NULL;
}

int hyle_field_is_reference(hyle_field_type_t type)
{
	return type == HYLE_FIELD_REFERENCE
		|| type == HYLE_FIELD_MULTI_REFERENCE
		|| type == HYLE_FIELD_INVERSE;
}
