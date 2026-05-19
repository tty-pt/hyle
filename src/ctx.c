#include <stdlib.h>
#include "hyle/ctx.h"
#include <ttypt/qmap.h>

hyle_ctx_t *hyle_ctx_new(void)
{
	hyle_ctx_t *ctx = (hyle_ctx_t *)calloc(1, sizeof(hyle_ctx_t));
	if (!ctx)
		return NULL;

	ctx->string_pool = qmap_open(NULL, "hyle.strings",
		QM_U32, QM_STR, 0xFF, QM_SORTED);
	ctx->array_pool = qmap_open(NULL, "hyle.arrays",
		QM_STR, QM_STR, 0xFF, QM_SORTED | QM_MULTIVALUE);
	ctx->map_pool = qmap_open(NULL, "hyle.maps",
		QM_STR, QM_STR, 0xFF, QM_SORTED);
	ctx->scratch = qmap_open(NULL, "hyle.scratch",
		QM_STR, QM_STR, 0xFF, 0);
	ctx->next_id = 0;

	if (!ctx->string_pool || !ctx->array_pool ||
	    !ctx->map_pool || !ctx->scratch) {
		hyle_ctx_free(ctx);
		return NULL;
	}

	return ctx;
}

void hyle_ctx_free(hyle_ctx_t *ctx)
{
	if (!ctx)
		return;
	if (ctx->scratch)    qmap_close(ctx->scratch);
	if (ctx->map_pool)   qmap_close(ctx->map_pool);
	if (ctx->array_pool) qmap_close(ctx->array_pool);
	if (ctx->string_pool) qmap_close(ctx->string_pool);
	free(ctx);
}
