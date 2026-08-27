#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <bud/bud_app.h>
#include <hyle-bud/hyle-bud.h>

/* ── Paginated omnisearch picker (form REFERENCE/MULTI_REFERENCE) ──
 *
 * SSR-first dropdown widget for item forms: searchable, paginated,
 * no-JS working mode (Prev/Next GET round-trips via a sibling form),
 * progressive infinite-scroll enhancement driven by neutral
 * data-hyle-* hooks. One internal builder feeds all three entry
 * points, so the SSR page, fragment reset, and fragment append shapes
 * are structurally identical. Precedent: filter.c. */

#define PICKER_SUMMARY_SZ 4096

static void picker_field_name(
	char *out, size_t out_sz, const char *prefix, const char *key)
{
	snprintf(out, out_sz, "%s%s", prefix, key ? key : "");
}

static int picker_is_selected(
	const hyle_bud_picker_desc_t *d, const char *id)
{
	int j;

	if (!id)
		return 0;
	for (j = 0; j < d->nsel; j++) {
		if (d->sel[j].id && strcmp(d->sel[j].id, id) == 0)
			return 1;
	}
	return 0;
}

static bud_node *picker_row_input(
	const hyle_bud_picker_desc_t *d, const char *id)
{
	return lx_el("input",
		lx_attr("type", d->multi ? "checkbox" : "radio"),
		lx_attr("name", d->key),
		lx_attr("value", id),
		picker_is_selected(d, id) ? lx_attr("checked", "")
					  : lx_none()).data.node;
}

static bud_node *picker_row(
	const hyle_bud_picker_desc_t *d, const char *id, const char *label)
{
	return lx_el("label",
		lx_attr("class", "hyle-picker-option"),
		lx_node(picker_row_input(d, id)),
		lx_text((label && label[0]) ? label : (id ? id : ""))).data.node;
}

static bud_node *picker_rows_node(
	const hyle_bud_picker_desc_t *d, int include_pinned)
{
	bud_node *rows = lx_el("div",
		lx_attr("class", "hyle-picker-rows")).data.node;
	int count = 0;
	int i;

	if (include_pinned) {
		/* Pinned selections are ALWAYS emitted regardless of
		 * q/page so values survive navigation. */
		for (i = 0; i < d->nsel; i++) {
			if (!d->sel[i].id)
				continue;
			bud_append(rows,
			        picker_row(d, d->sel[i].id,
			                (d->sel[i].label && d->sel[i].label[0]) ? d->sel[i].label
			                                                        : d->sel[i].id));
			count++;
		}
	}

	for (i = 0; i < d->npage; i++) {
		if (!d->page_opts[i].id)
			continue;
		/* Skip ids already covered by the pinned block. */
		if (include_pinned && picker_is_selected(d,
		            d->page_opts[i].id))
			continue;
		bud_append(rows,
		        picker_row(d, d->page_opts[i].id,
		                (d->page_opts[i].label && d->page_opts[i].label[0]) ? d->page_opts[i].label
		                                                                    : d->page_opts[i].id));
		count++;
	}

	if (!count)
		bud_append(rows,
		        lx_el("div",
		                lx_attr("class", "hyle-picker-empty"),
		                lx_text("No matches"))
		                .data.node);
	return rows;
}

static bud_node *picker_values_node(const hyle_bud_picker_desc_t *d)
{
	char summary[PICKER_SUMMARY_SZ];
	size_t pos = 0;
	int shown = 0;
	int i;

	summary[0] = '\0';
	for (i = 0; i < d->nsel; i++) {
		int n;
		if (!d->sel[i].id)
			continue;
		n = snprintf(summary + pos, sizeof(summary) - pos, "%s%s",
		        shown ? "; " : "",
		        d->sel[i].label ? d->sel[i].label : d->sel[i].id);
		if (n < 0 || (size_t)n >= sizeof(summary) - pos)
			break;
		pos += (size_t)n;
		shown++;
	}

	return lx_el("span",
		lx_attr("class", "hyle-picker-values"),
		lx_attr("data-hyle-slot", "values"),
		lx_text(summary)).data.node;
}

static bud_node *picker_search_node(const hyle_bud_picker_desc_t *d)
{
	char name[192];
	char aria[256];

	if (d->search_param && d->search_param[0])
		snprintf(name, sizeof(name), "%s", d->search_param);
	else
		picker_field_name(name, sizeof(name), "pick_q_", d->key);
	snprintf(aria, sizeof(aria), "Search %s",
	        d->label ? d->label : "options");

	return lx_el("input",
		lx_attr("type", "search"),
		lx_attr("name", name),
		lx_attr("class", "hyle-picker-search"),
		d->get_form_id ? lx_attr("form", d->get_form_id) : lx_none(),
		d->q && d->q[0] ? lx_attr("value", d->q) : lx_none(),
		lx_attr("placeholder", "Search\xe2\x80\xa6"),
		lx_attr("aria-label", aria)).data.node;
}

static bud_node *picker_paging_node(const hyle_bud_picker_desc_t *d)
{
	char name[192];
	char value[32];
	bud_node *paging;

	/* Dual-mode pagination: these buttons are the no-JS engine
	 * (GET round-trips through the sibling form); enhanced containers
	 * hide this block via CSS (.hyle-frag-active). */
	paging = lx_el("div",
		lx_attr("class", "hyle-picker-paging")).data.node;
	if (!d->get_form_id)
		return paging;

	if (d->page_param && d->page_param[0])
		snprintf(name, sizeof(name), "%s", d->page_param);
	else
		picker_field_name(name, sizeof(name), "pick_page_", d->key);

	if (d->page > 0) {
		snprintf(value, sizeof(value), "%d", d->page - 1);
		bud_append(paging,
		        lx_el("button",
		                lx_attr("type", "button"),
		                lx_attr("form", d->get_form_id),
		                lx_attr("name", name),
		                lx_attr("value", value),
		                lx_attr("class", "hyle-picker-page-btn"),
		                lx_attr("aria-label", "Previous page"),
		                lx_text("\xe2\x80\xb9"))
		                .data.node);
	}
	if (d->per_page > 0 && (d->page + 1) * d->per_page < d->total) {
		snprintf(value, sizeof(value), "%d", d->page + 1);
		bud_append(paging,
		        lx_el("button",
		                lx_attr("type", "button"),
		                lx_attr("form", d->get_form_id),
		                lx_attr("name", name),
		                lx_attr("value", value),
		                lx_attr("class", "hyle-picker-page-btn"),
		                lx_attr("aria-label", "Next page"),
		                lx_text("\xe2\x80\xba"))
		                .data.node);
	}
	return paging;
}

static bud_node *picker_panel_node(const hyle_bud_picker_desc_t *d)
{
	return lx_el("div",
		lx_attr("class", "hyle-picker-panel"),
		lx_attr("data-hyle-slot", "panel"),
		lx_node(picker_search_node(d)),
		lx_node(picker_rows_node(d, 1)),
		lx_el("div",
			lx_attr("class", "hyle-picker-more"),
			lx_attr("data-hyle-frag-sentinel", "")),
		lx_node(picker_paging_node(d))).data.node;
}

bud_node *hyle_bud_picker_field(const hyle_bud_picker_desc_t *d)
{
	char auto_url_tmpl[512];
	const char *url_tmpl;

	if (!d || !d->key || !d->key[0])
		return NULL;

	url_tmpl = d->url_tmpl;
	if ((!url_tmpl || !url_tmpl[0]) && d->source && d->source[0]) {
		char sp[64], pp[64];
		if (d->search_param && d->search_param[0])
			snprintf(sp, sizeof(sp), "%s", d->search_param);
		else
			snprintf(sp, sizeof(sp), "pick_q_%s", d->key);
		if (d->page_param && d->page_param[0])
			snprintf(pp, sizeof(pp), "%s", d->page_param);
		else
			snprintf(pp, sizeof(pp), "pick_page_%s", d->key);

		snprintf(auto_url_tmpl, sizeof(auto_url_tmpl),
		        "/pick/%s/options?key=%s&multi=%d&label=&sel={sel}&%s={q}&%s={page}",
		        d->source, d->key, d->multi ? 1 : 0, sp, pp);
		url_tmpl = auto_url_tmpl;
	}

	return lx_el("div",
		lx_attr("class", "hyle-picker"),
		lx_attr("data-hyle-picker", ""),
		lx_attr("data-hyle-picker-key", d->key),
		d->source ? lx_attr("data-hyle-picker-source", d->source)
		          : lx_none(),
		lx_attr("data-hyle-picker-multi", d->multi ? "1" : "0"),
		url_tmpl ? lx_attr("data-hyle-frag-url", url_tmpl)
			 : lx_none(),
		lx_el("details",
			lx_attr("class", "hyle-picker-details"),
			(d->q && d->q[0]) ? lx_attr("open", "") : lx_none(),
			lx_el("summary",
				lx_attr("class", "hyle-picker-trigger"),
				lx_node(picker_values_node(d)),
				lx_el("span",
					lx_attr("class", "hyle-picker-caret"),
					lx_attr("aria-hidden", "true"),
					lx_text("\xe2\x96\xbe"))),
			lx_node(picker_panel_node(d)))).data.node;
}

/* Serialize a tree to a caller buffer as HTML. bud_render_html
 * allocates; bud_sprint_tree is bud's debug dumper, not HTML. */
static void picker_sprint(bud_node *n, char *buf, size_t sz)
{
	char *html;

	if (!buf || !sz)
		return;
	buf[0] = '\0';
	if (!n)
		return;
	html = bud_render_html(n);
	if (!html)
		return;
	snprintf(buf, sz, "%s", html);
	bud_free_string(html);
}

void hyle_bud_picker_slots(const hyle_bud_picker_desc_t *d, char *panel,
        size_t panel_sz, char *values, size_t values_sz)
{
	bud_node *n;

	if (values && values_sz) {
		values[0] = '\0';
		n = picker_values_node(d);
		picker_sprint(n, values, values_sz);
	}
	if (panel && panel_sz) {
		panel[0] = '\0';
		n = picker_panel_node(d);
		picker_sprint(n, panel, panel_sz);
	}
}

void hyle_bud_picker_rows(
	const hyle_bud_picker_desc_t *d, char *rows, size_t rows_sz)
{
	bud_node *n;

	if (!rows || !rows_sz)
		return;
	rows[0] = '\0';
	n = picker_rows_node(d, 0);
	picker_sprint(n, rows, rows_sz);
}

struct parse_user {
	hyle_bud_option_t *opts;
	char (*ids)[128];
	char (*labels)[256];
	int max;
	int count;
};

static void picker_opt_item_cb(const char *elem, size_t len, void *user)
{
	struct parse_user *u = (struct parse_user *)user;
	int i = u->count;

	if (i >= u->max)
		return;
	u->ids[i][0] = '\0';
	u->labels[i][0] = '\0';
	bud_json_str_len(elem, len, "id", u->ids[i], sizeof(u->ids[i]));
	bud_json_str_len(elem, len, "label", u->labels[i], sizeof(u->labels[i]));
	u->opts[i].id = u->ids[i];
	u->opts[i].label = u->labels[i][0] ? u->labels[i] : u->ids[i];
	u->count = i + 1;
}

void hyle_bud_picker_state_from_json(
        const char *json, size_t jlen, const char *key, const char *target,
        int multi, const char *q, int page,
        hyle_bud_picker_buffer_t *buf, hyle_bud_picker_view_t *pv_out)
{
	struct parse_user u_opts, u_sel;

	if (!pv_out || !buf)
		return;

	memset(pv_out, 0, sizeof(*pv_out));
	memset(buf, 0, sizeof(*buf));

	pv_out->n = 1;
	hyle_bud_picker_entry_t *e = &pv_out->entries[0];
	e->key = key;
	e->target = target;
	e->multi = multi;
	e->q = q;
	e->page = page;

	u_opts.opts = buf->opts;
	u_opts.ids = buf->opt_ids;
	u_opts.labels = buf->opt_labels;
	u_opts.max = HYLE_BUD_PICKER_MAX_OPTS;
	u_opts.count = 0;

	bud_json_array_for_each_key_len(
	        json, jlen, "pick_opts", picker_opt_item_cb, &u_opts);
	e->page_opts = buf->opts;
	e->npage = u_opts.count;

	u_sel.opts = buf->sel;
	u_sel.ids = buf->sel_ids;
	u_sel.labels = buf->sel_labels;
	u_sel.max = HYLE_BUD_PICKER_MAX_SEL;
	u_sel.count = 0;

	bud_json_array_for_each_key_len(
	        json, jlen, "pick_sel", picker_opt_item_cb, &u_sel);
	e->sel = buf->sel;
	e->nsel = u_sel.count;

	e->per_page = bud_json_int_len(json, jlen, "pick_per_page", 25);
	e->total = bud_json_int_len(json, jlen, "pick_total", 0);
}

#ifndef __wasm__
#if __has_include(<json-c/json.h>)
#include <json-c/json.h>
void hyle_bud_picker_state_to_json(
        const hyle_bud_picker_view_t *pv, struct json_object *j_root)
{
	if (!pv || !j_root || pv->n <= 0)
		return;
	const hyle_bud_picker_entry_t *pe = &pv->entries[0];
	json_object *j_opts = json_object_new_array();
	json_object *j_sel = json_object_new_array();
	int oi;

	for (oi = 0; oi < pe->npage && oi < HYLE_BUD_PICKER_MAX_OPTS; oi++) {
		const hyle_bud_option_t *o = &pe->page_opts[oi];
		json_object *jo;

		if (!o || !o->id)
			continue;
		jo = json_object_new_object();
		json_object_object_add(
		        jo, "id", json_object_new_string(o->id));
		json_object_object_add(
		        jo, "label",
		        json_object_new_string(o->label ? o->label : o->id));
		json_object_array_add(j_opts, jo);
	}
	for (oi = 0; oi < pe->nsel && oi < HYLE_BUD_PICKER_MAX_SEL; oi++) {
		const hyle_bud_option_t *o = &pe->sel[oi];
		json_object *jo;

		if (!o || !o->id)
			continue;
		jo = json_object_new_object();
		json_object_object_add(
		        jo, "id", json_object_new_string(o->id));
		json_object_object_add(
		        jo, "label",
		        json_object_new_string(o->label ? o->label : o->id));
		json_object_array_add(j_sel, jo);
	}
	json_object_object_add(j_root, "pick_opts", j_opts);
	json_object_object_add(j_root, "pick_sel", j_sel);
	json_object_object_add(
	        j_root, "pick_per_page", json_object_new_int(pe->per_page));
	json_object_object_add(
	        j_root, "pick_total", json_object_new_int(pe->total));
}
#else
void hyle_bud_picker_state_to_json(
        const hyle_bud_picker_view_t *pv, struct json_object *j_root)
{
	(void)pv;
	(void)j_root;
}
#endif
#else
void hyle_bud_picker_state_to_json(
        const hyle_bud_picker_view_t *pv, struct json_object *j_root)
{
	(void)pv;
	(void)j_root;
}
#endif

void hyle_bud_state_apply_len(
        void *state, const hyle_schema_desc_t *fields, const char *json,
        size_t len)
{
	bud_state_apply_stride_len(
	        state, fields, sizeof(hyle_schema_desc_t), json, len);
}

void hyle_bud_state_apply(
        void *state, const hyle_schema_desc_t *fields, const char *json)
{
	size_t len = json ? strlen(json) : 0;
	hyle_bud_state_apply_len(state, fields, json, len);
}

static size_t hyle_url_decode(const char *src, size_t src_len, char *out, size_t out_len)
{
	size_t r = 0, w = 0;
	while (r < src_len && w + 1 < out_len) {
		if (src[r] == '%' && r + 2 < src_len) {
			char hex[3] = { src[r + 1], src[r + 2], 0 };
			char *end;
			long val = strtol(hex, &end, 16);
			if (end == hex + 2 && val >= 0) {
				out[w++] = (char)val;
				r += 3;
				continue;
			}
		}
		if (src[r] == '+')
			out[w++] = ' ';
		else
			out[w++] = src[r];
		r++;
	}
	out[w] = '\0';
	return w;
}

size_t hyle_bud_query_param(
        const char *qs, const char *key, char *out, size_t out_sz)
{
	size_t klen;
	const char *p;

	if (out && out_sz)
		out[0] = '\0';
	if (!qs || !key || !out || out_sz == 0)
		return 0;
	klen = strlen(key);
	p = qs;
	while (p && *p) {
		const char *amp = strchr(p, '&');
		size_t part_len = amp ? (size_t)(amp - p) : strlen(p);
		const char *eq = memchr(p, '=', part_len);
		if (eq && (size_t)(eq - p) == klen && strncmp(p, key, klen) == 0) {
			const char *v = eq + 1;
			size_t vlen = part_len - klen - 1;
			return hyle_url_decode(v, vlen, out, out_sz);
		}
		p = amp ? amp + 1 : NULL;
	}
	return 0;
}

int hyle_bud_pick_find_active_scope(const char *qs, char *scope_buf, size_t scope_sz)
{
	const char *p;
	if (scope_buf && scope_sz > 0)
		scope_buf[0] = '\0';
	if (!qs || !qs[0] || !scope_buf || scope_sz == 0)
		return -1;

	/* Check for replace=<scope> */
	p = strstr(qs, "replace=");
	if (p && (p == qs || p[-1] == '&' || p[-1] == '?')) {
		p += 8;
		const char *end = strchr(p, '&');
		size_t len = end ? (size_t)(end - p) : strlen(p);
		if (len > 0 && len < scope_sz) {
			memcpy(scope_buf, p, len);
			scope_buf[len] = '\0';
			return atoi(scope_buf);
		}
	}

	/* Check for pick_q_<key>__<scope>= or pick_page_<key>__<scope>= */
	p = qs;
	while ((p = strstr(p, "__")) != NULL) {
		const char *start = p + 2;
		const char *eq = strchr(start, '=');
		if (eq && eq > start) {
			const char *k = p;
			while (k > qs && k[-1] != '&' && k[-1] != '?')
				k--;
			if (strncmp(k, "pick_q_", 7) == 0 || strncmp(k, "pick_page_", 10) == 0) {
				size_t len = (size_t)(eq - start);
				if (len > 0 && len < scope_sz) {
					memcpy(scope_buf, start, len);
					scope_buf[len] = '\0';
					return atoi(scope_buf);
				}
			}
		}
		p += 2;
	}
	return -1;
}
