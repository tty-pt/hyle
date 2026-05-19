#ifndef HYLE_CTX_H
#define HYLE_CTX_H

#include <stdint.h>

typedef struct hyle_ctx_t {
	unsigned string_pool;
	unsigned array_pool;
	unsigned map_pool;
	unsigned scratch;
	uint32_t next_id;
} hyle_ctx_t;

hyle_ctx_t *hyle_ctx_new(void);
void hyle_ctx_free(hyle_ctx_t *ctx);

#endif
