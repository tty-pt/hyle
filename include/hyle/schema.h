#ifndef HYLE_SCHEMA_H
#define HYLE_SCHEMA_H

#include <stddef.h>

#define BUD_QM_STR 2
#define BUD_QM_VSTR 8
#define BUD_QM_MULTI_REF 7

/* Canonical hyle schema descriptor — framework-neutral, pure C, no bud dependency */
typedef struct hyle_schema_desc {
	const char *key;
	size_t offset;
	size_t size;
	int is_int;
	int kind; /* 0=include, 1=exclude, 2=virtual/ref-display */
	int qm_type;
	int source_type;
	int writable;
	int required;
	size_t min_length;
	const char *ref_source;
	const char *ref_inverse;
	int in_meta;
	const char *file;
	const char *filter_style;
	const char *filter_mode;
	const char *derive_key;
} hyle_schema_desc_t;

typedef struct hyle_schema_desc source_desc_t;

#endif
