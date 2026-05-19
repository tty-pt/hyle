#ifndef HYLE_VALUE_H
#define HYLE_VALUE_H

#include <stdbool.h>
#include <stdint.h>
#include <stddef.h>
#include "ctx.h"

typedef enum {
	HYLE_NULL = 0,
	HYLE_BOOL,
	HYLE_INT,
	HYLE_FLOAT,
	HYLE_STRING,
	HYLE_ARRAY,
	HYLE_MAP,
} hyle_val_type_t;

typedef struct {
	hyle_val_type_t type;
	uint8_t flags;
	uint8_t _pad[3];
	union {
		bool b;
		int64_t i;
		double f;
		uint32_t hd;
	};
} hyle_val_t;

_Static_assert(sizeof(hyle_val_t) == 16, "hyle_val_t must be 16 bytes");

typedef struct {
	hyle_ctx_t *ctx;
	uint32_t hd;
} hyle_row_t;

/* Constructors */
hyle_val_t hyle_val_null(void);
hyle_val_t hyle_val_bool(bool b);
hyle_val_t hyle_val_int(int64_t i);
hyle_val_t hyle_val_float(double f);
hyle_val_t hyle_val_string(hyle_ctx_t *ctx, const char *s);
hyle_val_t hyle_val_array(hyle_ctx_t *ctx);
hyle_val_t hyle_val_map(hyle_ctx_t *ctx);

/* Accessors */
const char *hyle_val_string_get(hyle_ctx_t *ctx, hyle_val_t v);

/* Array operations */
void hyle_val_array_push(hyle_ctx_t *ctx, hyle_val_t arr, hyle_val_t elem);
hyle_val_t hyle_val_array_get(hyle_ctx_t *ctx, hyle_val_t arr, size_t idx);
size_t hyle_val_array_len(hyle_ctx_t *ctx, hyle_val_t arr);

/* Map operations */
void hyle_val_map_set(hyle_ctx_t *ctx, hyle_val_t map, const char *key, hyle_val_t val);
hyle_val_t hyle_val_map_get(hyle_ctx_t *ctx, hyle_val_t map, const char *key);

/* JSON serialization — caller frees via free() */
char *hyle_val_to_json(hyle_ctx_t *ctx, hyle_val_t v);

#endif
