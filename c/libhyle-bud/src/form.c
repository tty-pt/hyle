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
	if (!key || !key[0])
		return "";
	if (strcmp(key, "title") == 0)
		return "Title:";
	if (strcmp(key, "type") == 0)
		return "Type:";
	if (strcmp(key, "author") == 0)
		return "Author:";
	if (strcmp(key, "yt") == 0)
		return "Youtube ID:";
	if (strcmp(key, "audio") == 0)
		return "Audio URL:";
	if (strcmp(key, "pdf") == 0)
		return "PDF URL:";
	if (strcmp(key, "data") == 0)
		return "Chords/Lyrics:";
	if (strcmp(key, "format") == 0)
		return "Format:";
	if (strcmp(key, "content") == 0)
		return "Content:";

	/* Capitalize first char */
	snprintf(
	        buf, sz, "%c%s:",
	        (key[0] >= 'a' && key[0] <= 'z') ? key[0] - 32 : key[0],
	        key + 1);
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

		if (is_ref && (!val || strlen(val) < HYLE_BUD_PICK_QS_BUDGET)) {
			ctl = hyle_bud_filter(schema, d->key, val, pv);
			has_ref = 1;
			if (!first_ref)
				first_ref = d->key;
		} else if (d->qm_type == BUD_QM_VSTR || strcmp(d->key, "format") == 0) {
			ctl = bud_tpl(
				"<textarea name='%s' class='font-mono w-full'>%node</textarea>",
				d->key,
				hyle_bud_textarea_value(val)
			);
		} else if (d->file && !strstr(d->file, ".txt") && !strstr(d->file, ".html")) {
			ctl = bud_tpl("<input type='file' name='%s'/>", d->key);
		} else {
			ctl = bud_tpl(
				"<input type='text' name='%s' value='%s'/>",
				d->key,
				(val && val[0]) ? val : ""
			);
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
