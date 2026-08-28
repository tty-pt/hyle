#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <bud/bud.h>
#include <hyle-bud/hyle-bud.h>

static void qs_without_sort(char *buf, size_t len, const char *qs)
{
	const char *p;
	size_t written = 0;

	buf[0] = '\0';
	if (!qs)
		return;
	p = qs;
	while (*p && written < len - 1) {
		if (strncmp(p, "sort=", 5) == 0) {
			const char *end = strchr(p, '&');
			if (end)
				p = end;
			else
				break;
		} else if (*p == '&' && strncmp(p + 1, "sort=", 5) == 0) {
			const char *end = strchr(p + 1, '&');
			if (end)
				p = end;
			else
				break;
		} else {
			buf[written++] = *p;
			buf[written] = '\0';
			p++;
		}
	}
}

bud_node *hyle_bud_table_header(
	const char **col_keys,
	const char **col_labels,
	int ncols,
	const char *sort_field,
	int sort_asc,
	const char *qs)
{
	bud_node *tr = bud_element("tr");
	int i;

	for (i = 0; i < ncols; i++) {
		char href[2048];
		char label[128];
		int active = sort_field && strcmp(col_keys[i], sort_field) == 0;

		{
			char without[1024];
			qs_without_sort(without, sizeof(without), qs);
			int new_asc = active ? !sort_asc : 1;
			if (without[0])
				snprintf(href, sizeof(href),
					"?%s&sort=%s:%s",
					without, col_keys[i],
					new_asc ? "asc" : "desc");
			else
				snprintf(href, sizeof(href),
					"?sort=%s:%s", col_keys[i],
					new_asc ? "asc" : "desc");
		}

		if (active) {
			const char *up = "▲";
			const char *dn = "▼";
			snprintf(label, sizeof(label), "%s %s",
				col_labels[i], sort_asc ? up : dn);
		} else {
			snprintf(label, sizeof(label), "%s", col_labels[i]);
		}

		bud_node *th = bud_tpl(
			"<th><a href='%s' class='hyle-sort-button'>%s</a></th>",
			href, label
		);
		if (th)
			bud_append(tr, th);
	}

	return bud_tpl("<thead>%node</thead>", tr);
}

static void row_action_append(
	bud_node *td,
	const hyle_bud_row_action_t *action,
	const char *id,
	const char *first_val,
	const char *module)
{
	char href[512];
	char aria[600];
	const char *text;
	bud_node *el = NULL;

	if (!td || !action || action->kind == HYLE_ROW_ACTION_NONE)
		return;
	text = action->label ? action->label : "";
	snprintf(aria, sizeof(aria), "%s%s%s",
		action->aria_base ? action->aria_base : "",
		action->aria_base && first_val && first_val[0] ? " " : "",
		first_val ? first_val : "");
	if (action->kind == HYLE_ROW_ACTION_LINK) {
		if (action->href_base && action->href_base[0])
			snprintf(href, sizeof(href), "%s/%s",
				action->href_base, id);
		else
			snprintf(href, sizeof(href), "/%s/%s",
				module, id);
		el = bud_tpl(
			"<a class='hyle-row-action' href='%s' aria-label='%s'>%s</a>",
			href, aria, text
		);
	} else if (action->kind == HYLE_ROW_ACTION_SUBMIT) {
		el = bud_tpl(
			"<button type='submit' class='hyle-row-action' value='%s' aria-label='%s'>%s</button>",
			id, aria, text
		);
		if (action->form_id)
			bud_set_attr(el, "form", action->form_id);
		if (action->field_name)
			bud_set_attr(el, "name", action->field_name);
	}
	if (!el)
		return;
	if (action->css_class && action->css_class[0])
		bud_add_class(el, action->css_class);
	bud_append(td, el);
}

static bud_node *table_body_impl(
	const char **col_keys,
	const char **col_labels,
	int ncols,
	const char **ids,
	int nids,
	const char **values,
	const char *module,
	const hyle_bud_row_action_t *action)
{
	bud_node *tbody = bud_element("tbody");
	int i, j;

	(void)col_keys;
	if (!tbody)
		return NULL;

	for (i = 0; i < nids; i++) {
		bud_node *tr = bud_element("tr");
		if (!tr)
			continue;
		bud_add_class(tr, "hyle-row-clickable");
		for (j = 0; j < ncols; j++) {
			const char *fval = values[i * ncols + j];
			if (!fval)
				fval = "";

			if (j == 0) {
				char item_href[512];
				snprintf(item_href, sizeof(item_href),
					"/%s/%s", module, ids[i]);
				bud_node *td = bud_tpl(
					"<td data-label='%s'><a href='%s'>%s</a></td>",
					col_labels[j], item_href, fval
				);
				if (!td)
					continue;
				row_action_append(td, action, ids[i],
					fval, module);
				bud_append(tr, td);
			} else {
				bud_node *td = bud_tpl(
					"<td data-label='%s'>%s</td>",
					col_labels[j], fval
				);
				if (td)
					bud_append(tr, td);
			}
		}
		bud_append(tbody, tr);
	}
	return tbody;
}

bud_node *hyle_bud_table_body(
	const char **col_keys,
	const char **col_labels,
	int ncols,
	const char **ids,
	int nids,
	const char **values,
	const char *module)
{
	return table_body_impl(col_keys, col_labels, ncols, ids, nids,
			values, module, NULL);
}

static bud_node *table_impl(
	const char **col_keys,
	const char **col_labels,
	int ncols,
	const char **ids,
	int nids,
	const char **values,
	const char *module,
	const char *sort_field,
	int sort_asc,
	const char *qs,
	const hyle_bud_row_action_t *action)
{
	bud_node *h = hyle_bud_table_header(
		col_keys, col_labels, ncols, sort_field, sort_asc, qs);
	bud_node *b = table_body_impl(
		col_keys, col_labels, ncols, ids, nids, values, module,
		action);

	return bud_tpl(
		"<div class='hyle-table-wrap'>"
		"  <table>"
		"    %node"
		"    %node"
		"  </table>"
		"</div>",
		h,
		b
	);
}

bud_node *hyle_bud_table(
	const char **col_keys,
	const char **col_labels,
	int ncols,
	const char **ids,
	int nids,
	const char **values,
	const char *module,
	const char *sort_field,
	int sort_asc,
	const char *qs)
{
	return table_impl(col_keys, col_labels, ncols, ids, nids, values,
			module, sort_field, sort_asc, qs, NULL);
}

bud_node *hyle_bud_table_actions(
	const char **col_keys,
	const char **col_labels,
	int ncols,
	const char **ids,
	int nids,
	const char **values,
	const char *module,
	const char *sort_field,
	int sort_asc,
	const char *qs,
	const hyle_bud_row_action_t *action)
{
	return table_impl(col_keys, col_labels, ncols, ids, nids, values,
			module, sort_field, sort_asc, qs, action);
}

bud_node *hyle_bud_pagination(
	int page,
	int per_page,
	int total,
	int row_count,
	const char *qs)
{
	int last_page = per_page > 0
		? (total + per_page - 1) / per_page
		: 1;
	char tmp_prev[64], tmp_next[64], tmp_text[64];
	char row_count_text[64];

	(void)qs;

	snprintf(tmp_prev, sizeof(tmp_prev), "%d",
		page > 1 ? page - 1 : 1);
	snprintf(tmp_next, sizeof(tmp_next), "%d",
		page < last_page ? page + 1 : last_page);
	snprintf(tmp_text, sizeof(tmp_text), "Page %d", page);
	snprintf(row_count_text, sizeof(row_count_text),
		"%d of %d rows", row_count, total);

	bud_node *sel = bud_element("select");
	if (sel) {
		bud_set_attr(sel, "name", "per_page");
		static const int per_page_opts[] = {5, 10, 20, 50, 100};
		size_t k;
		for (k = 0;
		     k < sizeof(per_page_opts) / sizeof(per_page_opts[0]);
		     k++)
		{
			bud_node *opt = bud_tpl(
				"<option value='%d' %b>%d / page</option>",
				per_page_opts[k],
				per_page_opts[k] == per_page ? "selected" : NULL,
				per_page_opts[k]
			);
			if (opt)
				bud_append(sel, opt);
		}
	}

	return bud_tpl(
		"<div class='hyle-table-footer'>"
		"  <div class='hyle-pagination'>"
		"    <button type='submit' name='page' value='%s' %b>← Prev</button>"
		"    <span class='text-sm'>%s</span>"
		"    <button type='submit' name='page' value='%s' %b>Next →</button>"
		"    %node"
		"    <button type='submit'>Apply</button>"
		"  </div>"
		"  <span class='hyle-row-count'>%s</span>"
		"</div>",
		tmp_prev,
		page <= 1 ? "disabled" : NULL,
		tmp_text,
		tmp_next,
		page >= last_page ? "disabled" : NULL,
		sel,
		row_count_text
	);
}
