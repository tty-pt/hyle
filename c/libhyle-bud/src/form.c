#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <limits.h>

#include <bud/bud.h>
#include <bud/bud_app.h>
#include <hyle-bud/hyle-bud.h>

static const char *hyle_bud_field_label(const char *key, char *buf, size_t sz)
{
	if (!key || !key[0] || sz < 2)
		return "";

	size_t bi = 0;
	int cap_next = 1;

	for (size_t i = 0; key[i] && bi + 2 < sz; i++) {
		char c = key[i];
		if (c == '_' || c == '-') {
			if (bi > 0 && buf[bi - 1] != ' ')
				buf[bi++] = ' ';
			cap_next = 1;
		} else if (cap_next && c >= 'a' && c <= 'z') {
			buf[bi++] = c - 32;
			cap_next = 0;
		} else {
			buf[bi++] = c;
			cap_next = 0;
		}
	}
	if (bi + 1 < sz)
		buf[bi++] = ':';
	buf[bi] = '\0';
	return buf;
}

static bud_node *hyle_bud_textarea_value(const char *value)
{
	const char *src = value ? value : "";
	size_t len = strlen(src);
	char *escaped;
	char *dst;

	if (len > (SIZE_MAX - 1) / 6)
		return bud_raw("");
	escaped = malloc(len * 6 + 1);
	if (!escaped)
		return bud_raw("");
	dst = escaped;
	while (*src) {
		const char *entity = NULL;
		size_t entity_len = 0;

		switch (*src) {
		case '&':
			entity = "&amp;";
			entity_len = 5;
			break;
		case '<':
			entity = "&lt;";
			entity_len = 4;
			break;
		case '>':
			entity = "&gt;";
			entity_len = 4;
			break;
		default:
			*dst++ = *src++;
			continue;
		}
		memcpy(dst, entity, entity_len);
		dst += entity_len;
		src++;
	}
	*dst = '\0';

	bud_node *node = bud_raw(escaped);
	free(escaped);
	return node;
}

bud_node *hyle_bud_form(
        const hyle_schema_desc_t *schema,
        const void *record,
        const char *action,
        const char *cancel_href,
        const char *submit_label,
        const char *csrf_token,
        const hyle_bud_picker_view_t *pv,
        const char *vstr_val)
{
	bud_node *fields_frag = bud_fragment();
	const char *first_ref = NULL;
	char label_buf[128];
	int has_ref = 0;

	if (!schema || !fields_frag)
		return NULL;

	if (csrf_token && csrf_token[0]) {
		bud_node *csrf = bud_tpl("<input type='hidden' name='csrf_token' value='%s'/>", csrf_token);
		if (csrf)
			bud_append(fields_frag, csrf);
	}

	for (const hyle_schema_desc_t *d = schema; d && d->key; d++) {
		if (strcmp(d->key, "id") == 0 || strcmp(d->key, "owner") == 0 ||
		    strcmp(d->key, "song_source") == 0)
			continue;
		if (!d->writable)
			continue;
		if (d->kind >= 3 || d->kind == 5) /* exclude computed or inverse */
			continue;

		const char *label = hyle_bud_field_label(d->key, label_buf, sizeof(label_buf));
		const char *val = "";

		if (d->qm_type == BUD_QM_VSTR && vstr_val) {
			val = vstr_val;
		} else if (record && d->size > 0 && d->source_type != HYLE_BUD_DERIVED) {
			val = (const char *)record + d->offset;
		}

		if ((!val || !val[0]) && pv && pv->n > 0) {
			for (int pi = 0; pi < pv->n; pi++) {
				if (pv->entries[pi].key &&
				    strcmp(pv->entries[pi].key, d->key) == 0)
				{
					if (pv->entries[pi].nsel > 0 &&
					    pv->entries[pi].sel &&
					    pv->entries[pi].sel[0].id)
					{
						val = pv->entries[pi].sel[0].id;
					}
					break;
				}
			}
		}

		bud_node *ctl = NULL;
		int is_ref = (d->source_type == HYLE_BUD_REFERENCE ||
		              d->source_type == HYLE_BUD_MULTI_REFERENCE ||
		              d->ref_source != NULL);
		const char *req_attr = d->required ? "required" : NULL;

		if (is_ref && (!val || strlen(val) < HYLE_BUD_PICK_QS_BUDGET)) {
			ctl = hyle_bud_filter(schema, d->key, val, pv);
			has_ref = 1;
			if (!first_ref)
				first_ref = d->key;
		} else if (d->is_int || d->source_type == HYLE_BUD_INT) {
			int int_val = 0;
			if (record && d->size >= sizeof(int))
				int_val = *(const int *)((const char *)record + d->offset);
			ctl = bud_tpl(
				"<input type='number' name='%s' value='%d' %b/>",
				d->key, int_val, req_attr);
		} else if (d->source_type == HYLE_BUD_BOOL) {
			int bool_val = 0;
			if (record && d->size >= sizeof(int))
				bool_val = *(const int *)((const char *)record + d->offset);
			ctl = bud_tpl(
				"<input type='checkbox' name='%s' value='1' %b %b/>",
				d->key,
				bool_val ? "checked" : NULL,
				req_attr);
		} else if (d->qm_type == BUD_QM_VSTR || strcmp(d->key, "format") == 0) {
			if (d->min_length > 0) {
				ctl = bud_tpl(
					"<textarea name='%s' class='font-mono w-full' minlength='%zu' %b>%node</textarea>",
					d->key, d->min_length, req_attr,
					hyle_bud_textarea_value(val)
				);
			} else {
				ctl = bud_tpl(
					"<textarea name='%s' class='font-mono w-full' %b>%node</textarea>",
					d->key, req_attr,
					hyle_bud_textarea_value(val)
				);
			}
		} else if (d->file && !strstr(d->file, ".txt") && !strstr(d->file, ".html")) {
			ctl = bud_tpl("<input type='file' name='%s' %b/>", d->key, req_attr);
		} else {
			size_t max_len = (d->size > 1) ? d->size - 1 : 0;
			if (d->min_length > 0 && max_len > 0) {
				ctl = bud_tpl(
					"<input type='text' name='%s' value='%s' minlength='%zu' maxlength='%zu' %b/>",
					d->key,
					(val && val[0]) ? val : "",
					d->min_length,
					max_len,
					req_attr
				);
			} else if (max_len > 0) {
				ctl = bud_tpl(
					"<input type='text' name='%s' value='%s' maxlength='%zu' %b/>",
					d->key,
					(val && val[0]) ? val : "",
					max_len,
					req_attr
				);
			} else if (d->min_length > 0) {
				ctl = bud_tpl(
					"<input type='text' name='%s' value='%s' minlength='%zu' %b/>",
					d->key,
					(val && val[0]) ? val : "",
					d->min_length,
					req_attr
				);
			} else {
				ctl = bud_tpl(
					"<input type='text' name='%s' value='%s' %b/>",
					d->key,
					(val && val[0]) ? val : "",
					req_attr
				);
			}
		}

		if (ctl) {
			bud_node *field_lbl = bud_tpl(
				"<label>%s %node</label>",
				label,
				ctl
			);
			if (field_lbl)
				bud_append(fields_frag, field_lbl);
		}
	}

	/* Form actions */
	bud_node *actions = bud_tpl(
		"<div class='flex gap-2'>"
		"  <button type='submit' class='btn btn-primary'>%s</button>"
		"  %node"
		"</div>",
		submit_label ? submit_label : "Save",
		(cancel_href && cancel_href[0]) ? bud_tpl("<a href='%s' class='btn btn-secondary'>Cancel</a>", cancel_href) : NULL
	);
	bud_append(fields_frag, actions);

	bud_node *form = bud_tpl(
		"<form action='%s' method='POST' enctype='multipart/form-data' class='flex flex-col gap-4'>"
		"  %node"
		"</form>",
		action ? action : "",
		fields_frag
	);

	if (has_ref && first_ref) {
		char form_id[192];
		snprintf(form_id, sizeof(form_id), "pickq-%s", first_ref);
		bud_node *hiddens = bud_fragment();
		long budget = HYLE_BUD_PICK_QS_BUDGET;

		for (const hyle_schema_desc_t *d = schema; d && d->key; d++) {
			if (strcmp(d->key, "id") == 0 || strcmp(d->key, "owner") == 0)
				continue;
			if (!d->writable || d->kind >= 3 || d->kind == 5)
				continue;
			if (d->file && !strstr(d->file, ".txt") && !strstr(d->file, ".html"))
				continue;

			const char *val = "";
			if (d->qm_type == BUD_QM_VSTR && vstr_val) {
				val = vstr_val;
			} else if (record && d->size > 0 && d->source_type != HYLE_BUD_DERIVED) {
				val = (const char *)record + d->offset;
			}

			if (!val || !val[0])
				continue;

			budget -= (long)(strlen(d->key) + strlen(val) + 16);
			if (budget < 0)
				break;

			bud_node *h = bud_tpl("<input type='hidden' name='%s' value='%s'/>", d->key, val);
			if (h)
				bud_append(hiddens, h);
		}

		bud_node *pp = bud_tpl("<input type='hidden' name='per_page' value='50'/>");
		if (pp)
			bud_append(hiddens, pp);

		bud_node *sib = bud_tpl(
			"<form id='%s' action='%s' method='GET' class='pick-sibling-form'>"
			"  %node"
			"</form>",
			form_id,
			action ? action : "",
			hiddens
		);

		return bud_tpl(
			"%node"
			"%node",
			form,
			sib
		);
	}

	return form;
}
