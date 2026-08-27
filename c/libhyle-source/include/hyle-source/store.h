#ifndef HYLE_SOURCE_STORE_H
#define HYLE_SOURCE_STORE_H

#include <stddef.h>
#include <stdint.h>

struct hyle_source_def_s;
typedef struct hyle_source_store_s hyle_source_store_t;

typedef struct {
	int (*scan)(hyle_source_store_t *store, const struct hyle_source_def_s *def);
	int (*load)(hyle_source_store_t *store, const struct hyle_source_def_s *def,
	            const char *id, unsigned *row_out);
	int (*put)(hyle_source_store_t *store, const struct hyle_source_def_s *def,
	           const char *id, unsigned data_handle);
	int (*put_field)(hyle_source_store_t *store,
	                 const struct hyle_source_def_s *def, const char *id,
	                 const char *field, const char *value);
	int (*del)(hyle_source_store_t *store, const struct hyle_source_def_s *def,
	           const char *id);
} hyle_source_store_ops_t;

struct hyle_source_store_s {
	const hyle_source_store_ops_t *ops;
	void *user;
};

const hyle_source_store_ops_t *hyle_source_store_fs_ops(void);
hyle_source_store_t hyle_source_store_fs(const char *items_path);

const hyle_source_store_ops_t *hyle_source_store_mem_ops(void);
hyle_source_store_t hyle_source_store_mem(void);

#endif
