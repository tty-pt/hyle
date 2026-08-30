#if __has_include(<json-c/json.h>) && !defined(__wasm__)
#include <json-c/json.h>
#endif
#include <bud/bud.h>
#include <hyle-bud/hyle-bud.h>

static hyle_bud_translate_fn g_hyle_bud_tr_fn = NULL;

void hyle_bud_set_translator(hyle_bud_translate_fn fn)
{
	g_hyle_bud_tr_fn = fn;
}

const char *hyle_bud_tr(const char *msgid)
{
	if (g_hyle_bud_tr_fn && msgid)
		return g_hyle_bud_tr_fn(msgid);
	return msgid ? msgid : "";
}

#if __has_include(<json-c/json.h>) && !defined(__wasm__)
int hyle_bud_state_overlay_from_desc(
        struct json_object *jo,
        const void *state,
        const bud_field_desc_t *fields,
        int int_kind,
        int str_kind)
{
	if (!jo || !state || !fields)
		return -1;
	for (const bud_field_desc_t *f = fields; f->key; f++) {
		if (f->kind == int_kind) {
			int val = *(const int *)((const char *)state + f->offset);
			json_object_object_add(
			        jo, f->key, json_object_new_int(val));
		} else if (f->kind == str_kind) {
			const char *val = (const char *)state + f->offset;
			json_object_object_add(
			        jo, f->key,
			        json_object_new_string(val ? val : ""));
		}
	}
	return 0;
}

struct json_object *hyle_bud_state_overlay_array(
        const void *items,
        int count,
        size_t elem_size,
        const bud_field_desc_t *fields,
        int int_kind,
        int str_kind)
{
	struct json_object *ja = json_object_new_array();
	if (!ja)
		return NULL;
	for (int i = 0; i < count; i++) {
		const void *item = (const char *)items + (size_t)i * elem_size;
		struct json_object *jo = json_object_new_object();
		if (!jo)
			continue;
		hyle_bud_state_overlay_from_desc(
		        jo, item, fields, int_kind, str_kind);
		json_object_array_add(ja, jo);
	}
	return ja;
}
#endif
