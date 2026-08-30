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

static bud_node *picker_row(
	const hyle_bud_picker_desc_t *d, const char *id, const char *label)
{
	return bud_tpl(
		"<label class='hyle-picker-option'>"
		"  <input type='%s' name='%s' value='%s' %b/> %s"
		"</label>",
		d->multi ? "checkbox" : "radio",
		d->key ? d->key : "",
		id ? id : "",
		picker_is_selected(d, id) ? "checked" : NULL,
		(label && label[0]) ? label : (id ? id : "")
	);
}

static int picker_populate_rows(
	const hyle_bud_picker_desc_t *d, bud_node *parent, int include_pinned)
{
	int count = 0;
	int i;

	if (!parent)
		return 0;

	if (include_pinned) {
		/* Pinned selections are ALWAYS emitted regardless of
		 * q/page so values survive navigation. */
		for (i = 0; i < d->nsel; i++) {
			if (!d->sel[i].id)
				continue;
			bud_node *r = picker_row(d, d->sel[i].id,
			        (d->sel[i].label && d->sel[i].label[0]) ? d->sel[i].label
			                                                : d->sel[i].id);
			if (r) {
				bud_append(parent, r);
				count++;
			}
		}
	}

	for (i = 0; i < d->npage; i++) {
		if (!d->page_opts[i].id)
			continue;
		/* Skip ids already covered by the pinned block. */
		if (include_pinned && picker_is_selected(d,
		            d->page_opts[i].id))
			continue;
		bud_node *r = picker_row(d, d->page_opts[i].id,
		        (d->page_opts[i].label && d->page_opts[i].label[0]) ? d->page_opts[i].label
		                                                            : d->page_opts[i].id);
		if (r) {
			bud_append(parent, r);
			count++;
		}
	}

	return count;
}

static bud_node *picker_rows_node(
	const hyle_bud_picker_desc_t *d, int include_pinned)
{
	bud_node *rows = bud_tpl("<div class='hyle-picker-rows'></div>");
	int count = picker_populate_rows(d, rows, include_pinned);

	if (!count && rows) {
		bud_node *empty = bud_tpl("<div class='hyle-picker-empty'>No matches</div>");
		if (empty)
			bud_append(rows, empty);
	}
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

	return bud_tpl(
		"<span class='hyle-picker-values' data-hyle-slot='values'>%s</span>",
		summary
	);
}

static bud_node *picker_search_node(const hyle_bud_picker_desc_t *d)
{
	char name[192];
	char aria[256];

	if (d->search_param && d->search_param[0])
		snprintf(name, sizeof(name), "%s", d->search_param);
	else
		picker_field_name(name, sizeof(name), "pick_q_", d->key);
	snprintf(aria, sizeof(aria), "%s %s",
	        hyle_bud_tr("Search"),
	        d->label ? hyle_bud_tr(d->label) : hyle_bud_tr("options"));

	bud_node *inp = bud_tpl(
		"<input type='search' name='%s' class='hyle-picker-search' value='%s' placeholder='%s' aria-label='%s'/>",
		name,
		(d->q && d->q[0]) ? d->q : "",
		hyle_bud_tr("Search…"),
		aria
	);
	if (inp && d->get_form_id)
		bud_set_attr(inp, "form", d->get_form_id);
	return inp;
}

static bud_node *picker_paging_node(const hyle_bud_picker_desc_t *d)
{
	char name[192];
	char value[32];
	bud_node *paging;

	/* Dual-mode pagination: these buttons are the no-JS engine
	 * (GET round-trips through the sibling form); enhanced containers
	 * hide this block via CSS (.hyle-frag-active). */
	paging = bud_tpl("<div class='hyle-picker-paging'></div>");
	if (!d->get_form_id)
		return paging;

	if (d->page_param && d->page_param[0])
		snprintf(name, sizeof(name), "%s", d->page_param);
	else
		picker_field_name(name, sizeof(name), "pick_page_", d->key);

	if (d->page > 0) {
		snprintf(value, sizeof(value), "%d", d->page - 1);
		bud_node *btn = bud_tpl(
			"<button type='button' form='%s' name='%s' value='%s' class='hyle-picker-page-btn' aria-label='Previous page'>‹</button>",
			d->get_form_id, name, value
		);
		if (btn)
			bud_append(paging, btn);
	}
	if (d->per_page > 0 && (d->page + 1) * d->per_page < d->total) {
		snprintf(value, sizeof(value), "%d", d->page + 1);
		bud_node *btn = bud_tpl(
			"<button type='button' form='%s' name='%s' value='%s' class='hyle-picker-page-btn' aria-label='Next page'>›</button>",
			d->get_form_id, name, value
		);
		if (btn)
			bud_append(paging, btn);
	}
	return paging;
}

static bud_node *picker_add_node(const hyle_bud_picker_desc_t *d)
{
	if (!d || !d->allow_add || !d->source || !d->source[0] || !d->q || !d->q[0])
		return NULL;

	return bud_tpl(
		"<div class='hyle-picker-add'>"
		"  <button type='button' class='hyle-picker-add-btn' data-hyle-picker-add='' "
		"data-hyle-picker-key='%s' data-hyle-picker-source='%s' data-hyle-picker-name='%s'>"
		"+ Add “%s”"
		"  </button>"
		"</div>",
		d->key ? d->key : "",
		d->source ? d->source : "",
		d->q,
		d->q
	);
}

static bud_node *picker_panel_node(const hyle_bud_picker_desc_t *d)
{
	return bud_tpl(
		"<div class='hyle-picker-panel' data-hyle-slot='panel'>"
		"  %node"
		"  %node"
		"  %node"
		"  <div class='hyle-picker-more' data-hyle-frag-sentinel=''></div>"
		"  %node"
		"</div>",
		picker_search_node(d),
		picker_add_node(d),
		picker_rows_node(d, 1),
		picker_paging_node(d)
	);
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
		        "/pick/%s/options?key=%s&multi=%d&add=%d&label=&sel={sel}&%s={q}&%s={page}",
		        d->source, d->key, d->multi ? 1 : 0, d->allow_add ? 1 : 0, sp, pp);
		url_tmpl = auto_url_tmpl;
	}

	bud_node *picker = bud_tpl(
		"<div class='hyle-picker' data-hyle-picker='' data-hyle-picker-key='%s' data-hyle-picker-multi='%d'>"
		"  <details class='hyle-picker-details' %b>"
		"    <summary class='hyle-picker-trigger'>"
		"      %node"
		"      <span class='hyle-picker-caret' aria-hidden='true'>▾</span>"
		"    </summary>"
		"    %node"
		"  </details>"
		"</div>",
		d->key,
		d->multi ? 1 : 0,
		(d->q && d->q[0]) ? "open" : NULL,
		picker_values_node(d),
		picker_panel_node(d)
	);

	if (picker) {
		if (d->allow_add)
			bud_set_attr(picker, "data-hyle-picker-addable", "1");
		if (d->source && d->source[0])
			bud_set_attr(picker, "data-hyle-picker-source", d->source);
		if (url_tmpl && url_tmpl[0])
			bud_set_attr(picker, "data-hyle-frag-url", url_tmpl);
	}

	return picker;
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
	bud_node *frag;
	int count;

	if (!rows || !rows_sz)
		return;
	rows[0] = '\0';
	frag = bud_fragment();
	if (!frag)
		return;
	count = picker_populate_rows(d, frag, 0);
	if (count > 0)
		picker_sprint(frag, rows, rows_sz);
	bud_free(frag);
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

bud_node *hyle_bud_filter_scoped(
	const hyle_schema_desc_t *desc,
	const char *field_name,
	int scope,
	const char *current_value,
	const char *current_label,
	const char *get_action,
	const hyle_bud_picker_view_t *pv,
	int is_active,
	const char *extra_class,
	bud_node **sibling_forms_out)
{
	const hyle_schema_desc_t *d = NULL;
	char dyn_key[64];
	char form_id[64];
	char search_param[64], page_param[64];
	const char *target = NULL;
	int multi = 0;

	if (!field_name || !field_name[0])
		return NULL;

	if (pv != NULL && pv->n == 0) {
		return hyle_bud_text_input(field_name, field_name, current_value);
	}

	if (desc) {
		for (const hyle_schema_desc_t *scan = desc; scan && scan->key; scan++) {
			if (strcmp(scan->key, field_name) == 0) {
				d = scan;
				break;
			}
		}
	}

	const hyle_bud_picker_entry_t *e = NULL;
	if (pv && pv->n > 0) {
		for (int pi = 0; pi < pv->n; pi++) {
			if (pv->entries[pi].key && strcmp(pv->entries[pi].key, field_name) == 0) {
				e = &pv->entries[pi];
				break;
			}
		}
		if (!e && is_active)
			e = &pv->entries[0];
	}
	hyle_bud_option_t sel_opt = {
		.id = current_value,
		.label = (current_label && current_label[0]) ? current_label : current_value
	};

	if (d) {
		target = d->ref_source ? d->ref_source : d->key;
		multi = (d->type == HYLE_FIELD_MULTI_REFERENCE || d->is_array != 0);
	} else if (e && e->target) {
		target = e->target;
		multi = e->multi;
	} else {
		target = field_name;
	}

	if (scope >= 0) {
		snprintf(dyn_key, sizeof(dyn_key), "%s_%d", field_name, scope);
		snprintf(form_id, sizeof(form_id), "pickq-%s_%d", field_name, scope);
		snprintf(search_param, sizeof(search_param), "pick_q_%s_%d", field_name, scope);
		snprintf(page_param, sizeof(page_param), "pick_page_%s_%d", field_name, scope);
	} else {
		snprintf(dyn_key, sizeof(dyn_key), "%s", field_name);
		snprintf(form_id, sizeof(form_id), "pickq-%s", field_name);
		snprintf(search_param, sizeof(search_param), "pick_q_%s", field_name);
		snprintf(page_param, sizeof(page_param), "pick_page_%s", field_name);
	}

	int allow_add = e ? e->allow_add : (d ? d->allow_add : 0);
	char url_tmpl_buf[512];
	snprintf(
	        url_tmpl_buf, sizeof(url_tmpl_buf),
	        "/pick/%s/options?key=%s&multi=%d&add=%d&label=&sel={sel}&%s={q}&%s={page}",
	        (e && e->target) ? e->target : target, dyn_key, multi ? 1 : 0,
	        allow_add ? 1 : 0,
	        search_param, page_param);

	hyle_bud_picker_desc_t pd = {
		.key = dyn_key,
		.label = field_name,
		.source = (e && e->target) ? e->target : target,
		.multi = multi,
		.get_form_id = form_id,
		.url_tmpl = url_tmpl_buf,
		.page_opts = e ? e->page_opts : NULL,
		.npage = e ? e->npage : 0,
		.sel = (e && e->nsel > 0 && e->sel) ? e->sel : &sel_opt,
		.nsel = (e && e->nsel > 0 && e->sel)
		                ? e->nsel
		                : ((current_value && current_value[0]) ? 1 : 0),
		.q = (e && e->q) ? e->q : "",
		.page = e ? e->page : 0,
		.per_page = (e && e->per_page > 0) ? e->per_page : 15,
		.total = e ? e->total : 0,
		.search_param = search_param,
		.page_param = page_param,
		.allow_add = allow_add
	};

	bud_node *picker = hyle_bud_picker_field(&pd);
	if (picker && extra_class && extra_class[0]) {
		bud_add_class(picker, extra_class);
	}

	if (sibling_forms_out && get_action && get_action[0]) {
		bud_node *sib = bud_tpl(
			"<form id='%s' action='%s' method='GET' class='pick-sibling-form'></form>",
			form_id,
			get_action
		);
		if (!*sibling_forms_out)
			*sibling_forms_out = bud_fragment();
		bud_append(*sibling_forms_out, sib);
	}

	return picker;
}

#ifndef __wasm__
#include <hyle-source/hyle_source.h>

#define HYLE_BUD_PICK_MAX_POOLS 16
#define HYLE_BUD_PICK_MAX_OPTS 128
#define HYLE_BUD_PICK_MAX_SEL 64

static __thread hyle_bud_option_t hyle_bud_v_opts[HYLE_BUD_PICK_MAX_POOLS][HYLE_BUD_PICK_MAX_OPTS];
static __thread char hyle_bud_v_ids[HYLE_BUD_PICK_MAX_POOLS][HYLE_BUD_PICK_MAX_OPTS][64];
static __thread char hyle_bud_v_labels[HYLE_BUD_PICK_MAX_POOLS][HYLE_BUD_PICK_MAX_OPTS][256];
static __thread hyle_bud_option_t hyle_bud_v_sel[HYLE_BUD_PICK_MAX_POOLS][HYLE_BUD_PICK_MAX_SEL];
static __thread char hyle_bud_v_sel_ids[HYLE_BUD_PICK_MAX_POOLS][HYLE_BUD_PICK_MAX_SEL][64];
static __thread char hyle_bud_v_sel_labels[HYLE_BUD_PICK_MAX_POOLS][HYLE_BUD_PICK_MAX_SEL][256];
static __thread char hyle_bud_v_q[HYLE_BUD_PICK_MAX_POOLS][256];
static __thread int hyle_bud_pool_cursor = 0;

int hyle_bud_picker_view_collect_scoped(
        const char *qs,
        const hyle_schema_desc_t *schema,
        const void *record,
        hyle_bud_picker_view_t *pv_out,
        const char *scope)
{
	int ri = 0;
	char buf[64];
	char skey[128];
	char pname[256];

	if (!pv_out)
		return 0;
	memset(pv_out, 0, sizeof(*pv_out));
	if (!schema || (qs && strlen(qs) >= HYLE_BUD_PICK_QS_BUDGET))
		return 0;

	for (const hyle_schema_desc_t *d = schema; d && d->key && ri < HYLE_BUD_PICKER_MAX_FIELDS; d++) {
		const char *target = d->ref_source;
		int is_ref = (d->type == HYLE_FIELD_REFERENCE || d->type == HYLE_FIELD_MULTI_REFERENCE || target != NULL);
		if (!is_ref || !target || !target[0])
			continue;
		if (d->writable == 0 && d->kind >= 3)
			continue;

		int slot = (hyle_bud_pool_cursor++) % HYLE_BUD_PICK_MAX_POOLS;
		hyle_bud_picker_entry_t *e = &pv_out->entries[ri];
		memset(e, 0, sizeof(*e));
		e->key = d->key;
		e->target = target;
		e->multi = (d->type == HYLE_FIELD_MULTI_REFERENCE || d->is_array != 0);
		e->allow_add = (d->allow_add && target) ? hyle_source_is_creatable(target) : 0;
		e->per_page = 15;

		if (scope && scope[0])
			snprintf(skey, sizeof(skey), "%s__%s", d->key, scope);
		else
			snprintf(skey, sizeof(skey), "%s", d->key);

		hyle_bud_v_q[slot][0] = '\0';
		if (qs && qs[0]) {
			snprintf(pname, sizeof(pname), "pick_q_%s", skey);
			hyle_bud_query_param(qs, pname, hyle_bud_v_q[slot], sizeof(hyle_bud_v_q[slot]));

			buf[0] = '\0';
			snprintf(pname, sizeof(pname), "pick_page_%s", skey);
			hyle_bud_query_param(qs, pname, buf, sizeof(buf));
			if (buf[0]) {
				e->page = atoi(buf);
				if (e->page < 0)
					e->page = 0;
				if (e->page > 10000)
					e->page = 10000;
			}
		}
		e->q = hyle_bud_v_q[slot];

		int total = 0;
		int nopts = hyle_source_resolve_options(
		        target, e->q, e->page, e->per_page,
		        hyle_bud_v_opts[slot], HYLE_BUD_PICK_MAX_OPTS, &total,
		        hyle_bud_v_ids[slot], hyle_bud_v_labels[slot]);
		e->total = total;
		e->page_opts = hyle_bud_v_opts[slot];
		e->npage = nopts;

		char draft_val[1024] = { 0 };
		const char *current_val = "";
		if (qs && qs[0]) {
			hyle_bud_query_param(qs, d->key, draft_val, sizeof(draft_val));
		}
		if (draft_val[0]) {
			current_val = draft_val;
		} else if (record && d->size > 0) {
			current_val = (const char *)record + d->offset;
		}

		if (current_val && current_val[0]) {
			e->nsel = hyle_source_resolve_tokens(
			        target, current_val, hyle_bud_v_sel[slot], HYLE_BUD_PICK_MAX_SEL,
			        hyle_bud_v_sel_ids[slot], hyle_bud_v_sel_labels[slot]);
			e->sel = hyle_bud_v_sel[slot];
		} else {
			e->nsel = 0;
			e->sel = NULL;
		}

		ri++;
	}

	pv_out->n = ri;
	return ri;
}

int hyle_bud_picker_view_collect_schema(
        const char *qs,
        const hyle_schema_desc_t *schema,
        const void *record,
        hyle_bud_picker_view_t *pv_out,
        int *active_scope_out)
{
	char scope_str[32] = { 0 };
	int scope = hyle_bud_pick_find_active_scope(qs, scope_str, sizeof(scope_str));
	if (active_scope_out)
		*active_scope_out = scope;

	if (scope >= 0 && scope_str[0]) {
		return hyle_bud_picker_view_collect_scoped(
		        qs, schema, record, pv_out, scope_str);
	}
	return hyle_bud_picker_view_collect_scoped(
	        qs, schema, record, pv_out, NULL);
}

int hyle_bud_picker_view_collect_auto_fields_schema(
        const char *qs,
        const hyle_schema_desc_t *schema,
        hyle_bud_picker_view_t *pv_out,
        int *active_field_idx_out,
        int *active_scope_out)
{
	if (active_field_idx_out)
		*active_field_idx_out = -1;
	if (active_scope_out)
		*active_scope_out = -1;

	if (!schema || !pv_out)
		return 0;

	if (!qs || !qs[0])
		return 0;

	int field_idx = 0;
	for (const hyle_schema_desc_t *d = schema; d && d->key; d++, field_idx++) {
		const char *fname = d->key;
		if (!fname || !fname[0])
			continue;

		/* Scan for pick_q_<fname>_<idx>= or pick_page_<fname>_<idx>= */
		char prefix_q[64], prefix_p[64];
		snprintf(prefix_q, sizeof(prefix_q), "pick_q_%s_", fname);
		snprintf(prefix_p, sizeof(prefix_p), "pick_page_%s_", fname);

		const char *p = strstr(qs, prefix_q);
		if (p && p != qs && p[-1] != '&' && p[-1] != '?')
			p = NULL;
		if (!p) {
			p = strstr(qs, prefix_p);
			if (p && p != qs && p[-1] != '&' && p[-1] != '?')
				p = NULL;
		}

		if (p) {
			const char *matched_prefix = (strstr(qs, prefix_q) == p) ? prefix_q : prefix_p;
			p += strlen(matched_prefix);
			if (*p >= '0' && *p <= '9') {
				int idx = atoi(p);
				if (active_field_idx_out)
					*active_field_idx_out = field_idx;
				if (active_scope_out)
					*active_scope_out = idx;

				char dyn_name[64];
				snprintf(dyn_name, sizeof(dyn_name), "%s_%d", fname, idx);
				hyle_schema_desc_t single_schema[] = {
					{ .key = dyn_name, .source_type = d->source_type,
					  .ref_source = d->ref_source, .kind = d->kind, .writable = d->writable,
					  .allow_add = d->allow_add },
					{ 0 }
				};
				return hyle_bud_picker_view_collect_scoped(qs, single_schema, NULL, pv_out, NULL);
			}
		}

		/* Scan for pick_q_<fname>__<scope>= or pick_page_<fname>__<scope>= */
		char scoped_q[64], scoped_p[64];
		snprintf(scoped_q, sizeof(scoped_q), "pick_q_%s__", fname);
		snprintf(scoped_p, sizeof(scoped_p), "pick_page_%s__", fname);

		p = strstr(qs, scoped_q);
		if (p && p != qs && p[-1] != '&' && p[-1] != '?')
			p = NULL;
		if (!p) {
			p = strstr(qs, scoped_p);
			if (p && p != qs && p[-1] != '&' && p[-1] != '?')
				p = NULL;
		}

		if (p) {
			const char *matched_scoped = (strstr(qs, scoped_q) == p) ? scoped_q : scoped_p;
			p += strlen(matched_scoped);
			int scope = atoi(p);
			if (active_field_idx_out)
				*active_field_idx_out = field_idx;
			if (active_scope_out)
				*active_scope_out = scope;

			char scope_str[32];
			snprintf(scope_str, sizeof(scope_str), "%d", scope);
			hyle_schema_desc_t single_schema[] = {
				*d,
				{ 0 }
			};
			return hyle_bud_picker_view_collect_scoped(qs, single_schema, NULL, pv_out, scope_str);
		}
	}

	/* 2. Check for general ?replace=<scope> */
	char scope_str[32] = { 0 };
	int scope = hyle_bud_pick_find_active_scope(qs, scope_str, sizeof(scope_str));
	if (scope >= 0 && scope_str[0]) {
		if (active_scope_out)
			*active_scope_out = scope;
		if (active_field_idx_out)
			*active_field_idx_out = 0;
		return hyle_bud_picker_view_collect_scoped(qs, schema, NULL, pv_out, scope_str);
	}

	return 0;
}
#endif /* !__wasm__ */

