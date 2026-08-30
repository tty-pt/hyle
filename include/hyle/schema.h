#ifndef HYLE_SCHEMA_H
#define HYLE_SCHEMA_H

#include <stddef.h>
#include "field.h"

/* ── Hyle Schema Descriptor Kinds (UI / Serialization Mode) ───── */
#define HYLE_KIND_RECORD      0 /* standard record field, include in state JSON */
#define HYLE_KIND_EXCLUDE     1 /* record field, exclude from state JSON */
#define HYLE_KIND_REF_DISPLAY 2 /* reference field, resolve IDs to display names */
#define HYLE_KIND_OVERLAY_INT 3 /* computed int overlay */
#define HYLE_KIND_OVERLAY_STR 4 /* computed string overlay */
#define HYLE_KIND_INVERSE     5 /* inverse virtual relation */

/* ── Hyle Storage / QMap Types ────────────────────────────────── */
#define HYLE_QM_STR           2
#define HYLE_QM_REFERENCE     6
#define HYLE_QM_MULTI_REF     7
#define HYLE_QM_VSTR          8

/* Backward compatibility aliases */
#ifndef BUD_RECORD
#define BUD_RECORD HYLE_KIND_RECORD
#define BUD_EXCLUDE HYLE_KIND_EXCLUDE
#define BUD_REF_DISPLAY HYLE_KIND_REF_DISPLAY
#define BUD_OVERLAY_INT HYLE_KIND_OVERLAY_INT
#define BUD_OVERLAY_STR HYLE_KIND_OVERLAY_STR
#define BUD_INVERSE HYLE_KIND_INVERSE
#endif
#ifndef BUD_QM_STR
#define BUD_QM_STR HYLE_QM_STR
#define BUD_QM_VSTR HYLE_QM_VSTR
#define BUD_QM_REFERENCE HYLE_QM_REFERENCE
#define BUD_QM_MULTI_REF HYLE_QM_MULTI_REF
#endif

/* ── Canonical Hyle Schema Descriptor ─────────────────────────── */
/* Framework-neutral, pure C data descriptor.
 * Layout contract: first 5 fields match bud_field_desc_t for zero-copy stride casting.
 */
typedef struct hyle_schema_desc {
	const char *key;
	size_t offset;
	size_t size;
	int is_int;
	int kind;        /* HYLE_KIND_* (first 5 fields mirror bud_field_desc_t) */
	int is_array;    /* 0 = scalar, 1 = collection / array */
	int qm_type;
	union {
		hyle_field_type_t type;
		int source_type; /* backwards compatibility alias */
	};
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
	int allow_add;
} hyle_schema_desc_t;

typedef struct hyle_schema_desc source_desc_t;

/* ── Member Size and Offset Helpers ───────────────────────────── */
#ifndef FIELD_SIZE
#define FIELD_SIZE(st, mb) sizeof(((st *)0)->mb)
#endif
#ifndef FIELD_OFFSET
#define FIELD_OFFSET(st, mb) offsetof(st, mb)
#endif

/* ── Base Field Property Generator Helpers ─────────────────────── */

#define _FIELD_TEXT(name, st, ...)                                             \
	.key = #name,                                                          \
	.offset = offsetof(st, name),                                          \
	.size = sizeof(((st *)0)->name),                                       \
	.is_int = 0,                                                           \
	.kind = HYLE_KIND_RECORD,                                              \
	.qm_type = HYLE_QM_STR,                                                \
	.type = HYLE_FIELD_STRING,                                              \
	.writable = 1,                                                         \
	##__VA_ARGS__

#define _FIELD_INT(name, st, ...)                                              \
	.key = #name,                                                          \
	.offset = offsetof(st, name),                                          \
	.size = sizeof(int),                                                   \
	.is_int = 1,                                                           \
	.kind = HYLE_KIND_RECORD,                                              \
	.type = HYLE_FIELD_INT,                                                 \
	.writable = 1,                                                         \
	##__VA_ARGS__

#define _FIELD_BOOL(name, st, ...)                                             \
	.key = #name,                                                          \
	.offset = offsetof(st, name),                                          \
	.size = sizeof(int),                                                   \
	.is_int = 0,                                                           \
	.kind = HYLE_KIND_RECORD,                                              \
	.type = HYLE_FIELD_BOOL,                                                \
	.writable = 1,                                                         \
	##__VA_ARGS__

#define _FIELD_REF(name, st, target, ...)                                      \
	.key = #name,                                                          \
	.offset = offsetof(st, name),                                          \
	.size = sizeof(((st *)0)->name),                                       \
	.is_int = 0,                                                           \
	.kind = HYLE_KIND_REF_DISPLAY,                                         \
	.qm_type = HYLE_QM_REFERENCE,                                          \
	.type = HYLE_FIELD_REFERENCE,                                           \
	.writable = 1,                                                         \
	.ref_source = target,                                                  \
	.file = #name,                                                         \
	##__VA_ARGS__

/* ── Modern Ergonomic Schema Macros ───────────────────────────── */

/* String / Text record field */
#define FIELD_TEXT(name, st, ...)                                             \
	{ _FIELD_TEXT(name, st, ##__VA_ARGS__) }

/* Integer record field */
#define FIELD_INT(name, st, ...)                                              \
	{ _FIELD_INT(name, st, ##__VA_ARGS__) }

/* Boolean record field */
#define FIELD_BOOL(name, st, ...)                                             \
	{ _FIELD_BOOL(name, st, ##__VA_ARGS__) }

/* Reference to foreign entity with automatic source lookup and display */
#define FIELD_REF(name, st, target, ...)                                      \
	{ _FIELD_REF(name, st, target, ##__VA_ARGS__) }

/* Attached file (e.g. data.txt, pt_PT.html) */
#define FIELD_FILE(name, file_name, ...)                                      \
	{                                                                      \
		.key = #name,                                                  \
		.offset = 0,                                                   \
		.size = 0,                                                     \
		.is_int = 0,                                                   \
		.kind = HYLE_KIND_EXCLUDE,                                     \
		.qm_type = HYLE_QM_VSTR,                                       \
		.type = HYLE_FIELD_STRING,                                      \
		.writable = 1,                                                 \
		.file = file_name,                                             \
		##__VA_ARGS__                                                  \
	}

/* Derived virtual field (in-memory derivation / search fold) */
#define FIELD_DERIVED(name, derive_func_key, ...)                             \
	{                                                                      \
		.key = #name,                                                  \
		.offset = 0,                                                   \
		.size = 0,                                                     \
		.is_int = 0,                                                   \
		.kind = HYLE_KIND_EXCLUDE,                                     \
		.qm_type = HYLE_QM_STR,                                        \
		.type = HYLE_FIELD_DERIVED,                                     \
		.writable = 0,                                                 \
		.derive_key = derive_func_key,                                 \
		##__VA_ARGS__                                                  \
	}

/* Inverse virtual relation */
#define FIELD_INVERSE(name, target, inv_field, ...)                            \
	{                                                                      \
		.key = #name,                                                  \
		.offset = 0,                                                   \
		.size = 0,                                                     \
		.is_int = 0,                                                   \
		.kind = HYLE_KIND_INVERSE,                                     \
		.qm_type = 0,                                                  \
		.type = HYLE_FIELD_INVERSE,                                     \
		.writable = 0,                                                 \
		.ref_source = target,                                          \
		.ref_inverse = inv_field,                                      \
		##__VA_ARGS__                                                  \
	}

/* Excluded field (backed by struct member, e.g. owner) */
#define FIELD_EXCL(name, st, ...)                                             \
	{                                                                      \
		.key = #name,                                                  \
		.offset = offsetof(st, name),                                  \
		.size = sizeof(((st *)0)->name),                               \
		.is_int = 0,                                                   \
		.kind = HYLE_KIND_EXCLUDE,                                     \
		.qm_type = HYLE_QM_STR,                                        \
		.type = HYLE_FIELD_STRING,                                      \
		##__VA_ARGS__                                                  \
	}

/* Overlay fields (for app state structures) */
#define OVERLAY_INT(name, st, mb)                                              \
	{                                                                      \
		.key = #name,                                                  \
		.offset = offsetof(st, mb),                                    \
		.size = sizeof(int),                                           \
		.is_int = 1,                                                   \
		.kind = HYLE_KIND_OVERLAY_INT                                  \
	}

#define OVERLAY_STR(name, st, mb, sz)                                          \
	{                                                                      \
		.key = #name,                                                  \
		.offset = offsetof(st, mb),                                    \
		.size = sz,                                                    \
		.is_int = 0,                                                   \
		.kind = HYLE_KIND_OVERLAY_STR                                  \
	}

/* End marker */
#define FIELD_END { .key = NULL }

/* ── Higher-Order Array Combinator ─────────────────────────────── */
/* Makes ANY base field type into an array/collection field (.is_array = 1) */
#define FIELD_ARRAY(type_macro, ...) type_macro(__VA_ARGS__, .is_array = 1)

/* Direct Array Convenience Aliases */
#define FIELD_ARRAY_INT(name, st, ...)                                        \
	FIELD_INT(name, st, .is_array = 1, ##__VA_ARGS__)
#define FIELD_ARRAY_TEXT(name, st, ...)                                       \
	FIELD_TEXT(name, st, .is_array = 1, ##__VA_ARGS__)
#define FIELD_ARRAY_BOOL(name, st, ...)                                       \
	FIELD_BOOL(name, st, .is_array = 1, ##__VA_ARGS__)
#define FIELD_ARRAY_REF(name, st, target, ...)                                \
	FIELD_REF(name, st, target, .is_array = 1, ##__VA_ARGS__)
#define FIELD_MULTI_REF FIELD_ARRAY_REF

/* ── Legacy Positional Field Macros (for backward compatibility) ── */

#define REC_FIELD(name, st, mb, sz, wr, rq, ml, im)                            \
	{                                                                      \
		#name, offsetof(st, mb), sz, 0, HYLE_KIND_RECORD,             \
		HYLE_QM_STR, { .type = HYLE_FIELD_STRING }, wr, rq, ml,        \
		NULL, NULL, im, NULL, NULL, NULL, NULL, 0                      \
	}

#define REF_FIELD(name, st, mb, sz, src, inv, im)                              \
	{                                                                      \
		#name, offsetof(st, mb), sz, 0, HYLE_KIND_REF_DISPLAY,         \
		HYLE_QM_REFERENCE, { .type = HYLE_FIELD_REFERENCE }, 1, 0, 0,  \
		src, inv, im, #name, NULL, NULL, NULL, 0                       \
	}

#define REF_FIELD_S(name, st, mb, sz, src, inv, im, style)                     \
	{                                                                      \
		#name, offsetof(st, mb), sz, 0, HYLE_KIND_REF_DISPLAY,         \
		HYLE_QM_REFERENCE, { .type = HYLE_FIELD_REFERENCE }, 1, 0, 0,  \
		src, inv, im, #name, style, NULL, NULL, 0                      \
	}

#define REF_FIELD_SA(name, st, mb, sz, src, inv, im, style, add)              \
	{                                                                      \
		#name, offsetof(st, mb), sz, 0, HYLE_KIND_REF_DISPLAY,         \
		HYLE_QM_REFERENCE, { .type = HYLE_FIELD_REFERENCE }, 1, 0, 0,  \
		src, inv, im, #name, style, NULL, NULL, add                    \
	}

#define MULTI_REF_FIELD(name, st, mb, sz, src, inv, im)                        \
	{                                                                      \
		#name, offsetof(st, mb), sz, 0, HYLE_KIND_REF_DISPLAY,         \
		HYLE_QM_MULTI_REF, { .type = HYLE_FIELD_MULTI_REFERENCE },     \
		1, 0, 0, src, inv, im, #name, NULL, NULL, NULL, 0              \
	}

#define MULTI_REF_FIELD_S(name, st, mb, sz, src, inv, im, style)               \
	{                                                                      \
		#name, offsetof(st, mb), sz, 0, HYLE_KIND_REF_DISPLAY,         \
		HYLE_QM_MULTI_REF, { .type = HYLE_FIELD_MULTI_REFERENCE },     \
		1, 0, 0, src, inv, im, #name, style, NULL, NULL, 0             \
	}

#define MULTI_REF_FIELD_SM(name, st, mb, sz, src, inv, im, style, mode)        \
	{                                                                      \
		#name, offsetof(st, mb), sz, 0, HYLE_KIND_REF_DISPLAY,         \
		HYLE_QM_MULTI_REF, { .type = HYLE_FIELD_MULTI_REFERENCE },     \
		1, 0, 0, src, inv, im, #name, style, mode, NULL, 0             \
	}

#define MULTI_REF_FIELD_SMA(name, st, mb, sz, src, inv, im, style, mode, add)  \
	{                                                                      \
		#name, offsetof(st, mb), sz, 0, HYLE_KIND_REF_DISPLAY,         \
		HYLE_QM_MULTI_REF, { .type = HYLE_FIELD_MULTI_REFERENCE },     \
		1, 0, 0, src, inv, im, #name, style, mode, NULL, add          \
	}

#define INVERSE_FIELD(name, src, inv)                                          \
	{                                                                      \
		#name, 0, 0, 0, HYLE_KIND_INVERSE, 0,                          \
		{ .type = HYLE_FIELD_INVERSE }, 0, 0, 0,                       \
		src, inv, 0, NULL, NULL, NULL, NULL, 0                         \
	}

#define EXCL_FIELD(name, st, mb, sz, ...)                                      \
	{                                                                      \
		#name, offsetof(st, mb), sz, 0, HYLE_KIND_EXCLUDE,             \
		HYLE_QM_STR, { .type = HYLE_FIELD_STRING }, 0, 0, 0,            \
		NULL, NULL, 0, NULL, NULL, NULL, NULL, 0                       \
	}

#define EXCL_FIELD_M(name, st, mb, sz, im)                                     \
	{                                                                      \
		#name, offsetof(st, mb), sz, 0, HYLE_KIND_EXCLUDE,             \
		HYLE_QM_STR, { .type = HYLE_FIELD_STRING }, 0, 0, 0,            \
		NULL, NULL, im, NULL, NULL, NULL, NULL, 0                      \
	}

#define EXCL_FIELD_W(name, st, mb, sz, qt, im)                                 \
	{                                                                      \
		#name, offsetof(st, mb), sz, 0, HYLE_KIND_EXCLUDE, qt,          \
		{ .type = HYLE_FIELD_STRING }, 1, 0, 0, NULL, NULL, im,        \
		NULL, NULL, NULL, NULL, 0                                      \
	}

#define EXCL_FIELD_V(name, qt, wr, im)                                         \
	{                                                                      \
		#name, 0, 0, 0, HYLE_KIND_EXCLUDE, qt,                         \
		{ .type = HYLE_FIELD_STRING }, wr, 0, 0,                        \
		NULL, NULL, im, NULL, NULL, NULL, NULL, 0                      \
	}

#define EXCL_FIELD_VF(name, qt, wr, im, fl)                                    \
	{                                                                      \
		#name, 0, 0, 0, HYLE_KIND_EXCLUDE, qt,                         \
		{ .type = HYLE_FIELD_STRING }, wr, 0, 0,                        \
		NULL, NULL, im, fl, NULL, NULL, NULL, 0                        \
	}

#define DERIVED_FIELD(name, dkey)                                              \
	{                                                                      \
		#name, 0, 0, 0, HYLE_KIND_EXCLUDE, HYLE_QM_STR,                \
		{ .type = HYLE_FIELD_DERIVED }, 0, 0, 0, NULL, NULL, 0, NULL,  \
		NULL, NULL, dkey, 0                                            \
	}

#define INT_FIELD(name, st, mb, wr)                                            \
	{                                                                      \
		#name, offsetof(st, mb), sizeof(int), 1, HYLE_KIND_RECORD,     \
		0, { .type = HYLE_FIELD_INT }, wr, 0, 0, NULL, NULL, 0,        \
		NULL, NULL, NULL, NULL, 0                                      \
	}

#define BOOL_FIELD(name, st, mb, wr)                                           \
	{                                                                      \
		#name, offsetof(st, mb), sizeof(int), 0, HYLE_KIND_RECORD,     \
		0, { .type = HYLE_FIELD_BOOL }, wr, 0, 0, NULL, NULL, 0,       \
		NULL, NULL, NULL, NULL, 0                                      \
	}

#define REQ_FIELD(name, st, mb, sz)                                            \
	{                                                                      \
		#name, offsetof(st, mb), sz, 0, HYLE_KIND_RECORD,              \
		HYLE_QM_STR, { .type = HYLE_FIELD_STRING }, 1, 1, 0,           \
		NULL, NULL, 0, NULL, NULL, NULL, NULL, 0                       \
	}

#define REQ_FIELD_MIN(name, st, mb, sz, ml)                                    \
	{                                                                      \
		#name, offsetof(st, mb), sz, 0, HYLE_KIND_RECORD,              \
		HYLE_QM_STR, { .type = HYLE_FIELD_STRING }, 1, 1, ml,          \
		NULL, NULL, 0, NULL, NULL, NULL, NULL, 0                       \
	}

#define VSTR_FIELD(name, fl)                                                   \
	{                                                                      \
		#name, 0, 0, 0, HYLE_KIND_EXCLUDE, HYLE_QM_VSTR,               \
		{ .type = HYLE_FIELD_STRING }, 1, 0, 0, NULL, NULL, 0,         \
		fl, NULL, NULL, NULL, 0                                        \
	}

#define FILE_FIELD(name, file_name) VSTR_FIELD(name, file_name)

#endif
