#include "hyle-source/store.h"
#include "hyle-source/hyle_source.h"
#include "source_utils.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <limits.h>
#include <sys/stat.h>
#include <dirent.h>
#include <unistd.h>
#include <errno.h>
#include <ttypt/qmap.h>
#include <hyle/hyle.h>
#include <hyle/source.h>

static int fs_load(hyle_source_store_t *store, const hyle_source_def_t *def,
                   const char *id, unsigned *row_out)
{
	(void)store;
	if (!def || !id)
		return -1;
	if (!source_util_is_safe_id(id))
		return -1;
	const char *items_path = def->items_path
	        ? def->items_path
	        : (const char *)store->user;
	if (!items_path || !items_path[0])
		return -1;
	char doc_root[256] = { 0 };
	const char *root = source_util_resolve_doc_root(doc_root, sizeof(doc_root));
	char item_path[PATH_MAX];
	snprintf(item_path, sizeof(item_path), "%s/%s/%s", root,
	        items_path, id);
	struct stat st;
	if (lstat(item_path, &st) != 0 || !S_ISDIR(st.st_mode))
		return -1;
	const char *names[64];
	const char *values[64];
	size_t k = 0;
	char *bufs[64];
	size_t nb = 0;
	char id_norm[256];
	source_util_slugify(id, strnlen(id, sizeof(id_norm)), id_norm, sizeof(id_norm));
	names[k] = "id";
	values[k] = strcmp(id, id_norm) != 0 ? id_norm : id;
	k++;
	for (size_t i = 0; i < def->field_count && k < 64; i++) {
		const hyle_source_field_t *f = &def->fields[i];
		if (strcmp(f->name, "id") == 0)
			continue;
		if (!f->file)
			continue;
		char file_path[PATH_MAX + 256];
		snprintf(file_path, sizeof(file_path), "%s/%s", item_path,
		        f->file);
		char *data = source_util_slurp_file(file_path);
		if (data) {
			hyle_source_internal_process_multi_ref(f, def->id, &data);
			names[k] = f->name;
			values[k] = data;
			k++;
			bufs[nb++] = data;
		}
	}
	int rc = hyle_source_put(def->id, id, names, values, k);
	for (size_t i = 0; i < nb; i++)
		free(bufs[i]);
	if (row_out)
		*row_out = 0;
	return rc;
}

static int fs_scan(hyle_source_store_t *store, const hyle_source_def_t *def)
{
	const char *items_path = def->items_path
	        ? def->items_path
	        : (const char *)store->user;
	if (!items_path || !items_path[0])
		return 0;
	char doc_root[256] = { 0 };
	const char *root = source_util_resolve_doc_root(doc_root, sizeof(doc_root));
	char full[PATH_MAX];
	snprintf(full, sizeof(full), "%s/%s", root, items_path);
	DIR *dir = opendir(full);
	if (!dir)
		return 0;
	struct dirent *entry;
	while ((entry = readdir(dir))) {
		if (entry->d_name[0] == '.')
			continue;
		unsigned row_unused = 0;
		fs_load(store, def, entry->d_name, &row_unused);
	}
	closedir(dir);
	return 0;
}

static int fs_put(hyle_source_store_t *store, const hyle_source_def_t *def,
                  const char *id, unsigned data_handle)
{
	(void)store;
	if (!def || !id)
		return -1;
	if (!source_util_is_safe_id(id))
		return -1;
	const char *items_path = def->items_path
	        ? def->items_path
	        : (const char *)store->user;
	if (!items_path || !items_path[0])
		return -1;
	char doc_root[256] = { 0 };
	const char *root = source_util_resolve_doc_root(doc_root, sizeof(doc_root));
	char item_path[PATH_MAX];
	snprintf(item_path, sizeof(item_path), "%s/%s/%s", root,
	        items_path, id);
	if (mkdir(item_path, 0755) != 0 && errno != EEXIST)
		return -1;
	for (size_t i = 0; i < def->field_count; i++) {
		const hyle_source_field_t *f = &def->fields[i];
		if (strcmp(f->name, "owner") == 0)
			continue;
		if (!f->file)
			continue;
		const char *val = qmap_get(data_handle, f->name);
		char file_path[PATH_MAX + 256];
		snprintf(file_path, sizeof(file_path), "%s/%s",
		        item_path, f->file);
		if (val) {
			if (source_util_write_file(file_path, val, strlen(val)) != 0)
				return -1;
		} else {
			char *content = source_util_slurp_file(file_path);
			if (content) {
				free(content);
			} else if (f->type !=
			           HYLE_SOURCE_FIELD_MULTI_REFERENCE) {
				FILE *fp = fopen(file_path, "w");
				if (fp)
					fclose(fp);
			}
		}
	}
	return 0;
}

static int fs_put_field(hyle_source_store_t *store, const hyle_source_def_t *def,
                        const char *id, const char *field,
                        const char *value)
{
	(void)def;
	const char *items_path = def->items_path
	        ? def->items_path
	        : (const char *)store->user;
	if (!items_path || !items_path[0] || !id || !field)
		return -1;
	if (!source_util_is_safe_id(id))
		return -1;
	char doc_root[256] = { 0 };
	const char *root = source_util_resolve_doc_root(doc_root, sizeof(doc_root));
	char file_path[PATH_MAX + 256];
	snprintf(file_path, sizeof(file_path), "%s/%s/%s/%s", root, items_path, id, field);
	size_t vlen = value ? strlen(value) : 0;
	return source_util_write_file(file_path, value ? value : "", vlen);
}

static int fs_del(hyle_source_store_t *store, const hyle_source_def_t *def,
                  const char *id)
{
	(void)store;
	const char *items_path = def->items_path
	        ? def->items_path
	        : (const char *)store->user;
	if (!items_path || !items_path[0] || !id)
		return -1;
	if (!source_util_is_safe_id(id))
		return -1;
	char doc_root[256] = { 0 };
	const char *root = source_util_resolve_doc_root(doc_root, sizeof(doc_root));
	char item_path[PATH_MAX];
	snprintf(item_path, sizeof(item_path), "%s/%s/%s", root,
	        items_path, id);
	source_util_remove_path_recursive(item_path);
	return 0;
}

static const hyle_source_store_ops_t fs_ops = {
	.scan = fs_scan,
	.load = fs_load,
	.put = fs_put,
	.put_field = fs_put_field,
	.del = fs_del,
};

const hyle_source_store_ops_t *hyle_source_store_fs_ops(void) { return &fs_ops; }

hyle_source_store_t hyle_source_store_fs(const char *items_path)
{
	hyle_source_store_t s = { &fs_ops, (void *)items_path };
	return s;
}
