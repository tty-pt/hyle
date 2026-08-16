#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <bud/bud_app.h>
#include <hyle-bud/hyle-bud.h>

/* ── Shared multi-value selection helpers ─────────────────────── */

static int hyle_bud_comma_split(const char *cur, const char **out, int max)
{
	int n = 0;

	if (!cur || !cur[0])
		return 0;
	while (*cur && n < max) {
		const char *comma = strchr(cur, ',');
		size_t len = comma ? (size_t)(comma - cur) : strlen(cur);
		if (len > 0) {
			out[n] = cur;
			n++;
		}
		if (!comma)
			break;
		cur = comma + 1;
	}
	return n;
}

static int hyle_bud_is_selected(
	const char *const *sel, int nsel, const char *id)
{
	size_t olen;
	int j;

	if (!id)
		return 0;
	olen = strlen(id);
	for (j = 0; j < nsel; j++) {
		if (strncmp(sel[j], id, olen) == 0 &&
		    (sel[j][olen] == '\0' || sel[j][olen] == ','))
			return 1;
	}
	return 0;
}

static bud_node *hyle_bud_boolean_checkbox(
	const char *key,
	const char *label,
	const char *current_value)
{
	bud_node *legend = bud_element("legend");
	bud_node *cb = lx_el("input",
		lx_attr("type", "checkbox"),
		lx_attr("name", key),
		current_value && strcmp(current_value, "true") == 0
			? lx_attr("checked", "")
			: lx_none()).data.node;
	bud_node *lbl = lx_el("label",
		lx_node(cb),
		lx_text(label)).data.node;

	return lx_el("fieldset",
		legend ? lx_node(legend) : lx_none(),
		lx_node(lbl)).data.node;
}

static bud_node *hyle_bud_checkbox_fieldset(
	const char *key,
	const char *label,
	const char *current_value,
	const hyle_bud_option_t *options,
	int noptions)
{
	const char *selected[1024];
	int nselected;
	bud_node *legend;
	bud_node *fs;
	int i;

	nselected = hyle_bud_comma_split(current_value, selected, 1024);

	legend = lx_el("legend", lx_text(label)).data.node;
	fs = lx_el("fieldset",
		lx_attr("class", "hyle-checkbox-filter"),
		lx_node(legend)).data.node;

	for (i = 0; i < noptions; i++) {
		int checked =
		        hyle_bud_is_selected(selected, nselected, options[i].id);
		bud_node *cb = lx_el("input",
			lx_attr("type", "checkbox"),
			lx_attr("name", key),
			lx_attr("value", options[i].id),
			checked ? lx_attr("checked", "") : lx_none()).data.node;
		bud_node *lbl = lx_el("label",
			lx_node(cb),
			lx_text(options[i].label)).data.node;
		bud_append(fs, lbl);
	}

	return fs;
}

/* ── Multi-select dropdown widget (SSR-first, WASM-enhanced) ──── */

#define HYLE_BUD_MS_MAX 8
#define HYLE_BUD_MS_MAX_OPTS 256

typedef struct {
	const char *key;                    /* field key (name=key) */
	const char *label;                  /* field label ("All {label}s") */
	bud_node *search;                   /* data-hyle-ms-search input */
	bud_node *summary_values;           /* data-hyle-ms-values span */
	bud_node *options_container;        /* data-hyle-ms-options div */
	hyle_bud_option_t opts_copy[HYLE_BUD_MS_MAX_OPTS]; /* persistent copy */
	const hyle_bud_option_t *opts;
	int noptions;
	bud_node *option_rows[HYLE_BUD_MS_MAX_OPTS];
	bud_node *checkboxes[HYLE_BUD_MS_MAX_OPTS];
	char checked[HYLE_BUD_MS_MAX_OPTS]; /* 0/1 bitmap, initialized from cur */
} hyle_bud_ms_t;

static hyle_bud_ms_t g_ms[HYLE_BUD_MS_MAX];
static int g_ms_count = 0;

void hyle_bud_ms_reset(void)
{
	g_ms_count = 0;
}

static const char *ms_ci_substr(const char *haystack, const char *needle)
{
	size_t i;

	if (!needle || !needle[0])
		return haystack;
	if (!haystack)
		return NULL;
	for (; *haystack; haystack++) {
		for (i = 0; needle[i] && haystack[i]; i++) {
			char a = haystack[i];
			char b = needle[i];
			if (a >= 'A' && a <= 'Z')
				a += 32;
			if (b >= 'A' && b <= 'Z')
				b += 32;
			if (a != b)
				break;
		}
		if (!needle[i])
			return haystack;
		if (!haystack[i])
			break;
	}
	return NULL;
}

static void ms_summary_build(
	char *out, size_t out_sz, const hyle_bud_option_t *opts, int noptions,
	const char *checked, const char *label)
{
	size_t pos = 0;
	int shown = 0;
	int i;

	out[0] = '\0';
	for (i = 0; i < noptions; i++) {
		int n;
		if (!checked[i])
			continue;
		if (pos + 1 >= out_sz)
			break;
		n = snprintf(out + pos, out_sz - pos, "%s%s",
		             shown ? "; " : "", opts[i].label);
		if (n < 0 || (size_t)n >= out_sz - pos)
			break;
		pos += n;
		shown = 1;
	}
	if (!shown)
		snprintf(out, out_sz, "All %ss", label);
}

static int hyle_bud_ms_find_widget(bud_node *target)
{
	int i;

	if (!target)
		return -1;
	for (i = 0; i < g_ms_count; i++) {
		int j;
		if (g_ms[i].search == target)
			return i;
		for (j = 0; j < g_ms[i].noptions; j++) {
			if (g_ms[i].checkboxes[j] == target)
				return i;
		}
	}
	return -1;
}

static int hyle_bud_ms_on_search(bud_event *event)
{
	int w = hyle_bud_ms_find_widget(event->target);
	const char *needle;
	int i;

	if (w < 0)
		return 0;
	needle = (const char *)event->user;
	if (!needle)
		needle = "";
	for (i = 0; i < g_ms[w].noptions; i++) {
		int visible = ms_ci_substr(g_ms[w].opts[i].label, needle) != NULL;
		bud_patch_attr(g_ms[w].option_rows[i], "class",
		               visible ? "hyle-ms-option"
		                       : "hyle-ms-option hyle-ms-hidden");
	}
	return 0;
}

static int hyle_bud_ms_on_change(bud_event *event)
{
	int w = hyle_bud_ms_find_widget(event->target);
	int now;
	int k;
	char summary[4096];

	if (w < 0)
		return 0;
	for (k = 0; k < g_ms[w].noptions; k++) {
		if (g_ms[w].checkboxes[k] == event->target)
			break;
	}
	if (k >= g_ms[w].noptions)
		return 0;
	now = event->user && ((const char *)event->user)[0] == '1';
	g_ms[w].checked[k] = (char)now;
	ms_summary_build(summary, sizeof(summary), g_ms[w].opts,
	                 g_ms[w].noptions, g_ms[w].checked, g_ms[w].label);
	bud_patch_text(g_ms[w].summary_values, summary);
	return 0;
}

bud_node *hyle_bud_multiselect_field(
	const char *key,
	const char *label,
	const char *current_value,
	const hyle_bud_option_t *options,
	int noptions)
{
	const char *selected[1024];
	int nselected;
	hyle_bud_ms_t *w;
	bud_node *summary;
	bud_node *summary_text_node;
	bud_node *search;
	bud_node *container;
	bud_node *caption;
	char summary_text[4096];
	int i;

	if (g_ms_count >= HYLE_BUD_MS_MAX)
		return hyle_bud_checkbox_fieldset(
		        key, label, current_value, options, noptions);

	w = &g_ms[g_ms_count];
	g_ms_count++;

	w->key = key;
	w->label = label;
	w->opts = w->opts_copy;
	w->noptions = noptions > HYLE_BUD_MS_MAX_OPTS
	                      ? HYLE_BUD_MS_MAX_OPTS
	                      : noptions;

	nselected = hyle_bud_comma_split(current_value, selected, 1024);
	for (i = 0; i < w->noptions; i++) {
		w->opts_copy[i].id = options[i].id;
		w->opts_copy[i].label = options[i].label;
		w->checked[i] =
		        (char)hyle_bud_is_selected(selected, nselected, options[i].id);
		w->option_rows[i] = NULL;
		w->checkboxes[i] = NULL;
	}

	ms_summary_build(summary_text, sizeof(summary_text), w->opts,
	                 w->noptions, w->checked, label);

	summary_text_node = bud_text(summary_text);
	summary = lx_el("span",
		lx_attr("class", "hyle-ms-values"),
		lx_attr("data-hyle-ms-values", "1"),
		lx_node(summary_text_node)).data.node;
	w->summary_values = summary_text_node;

	caption = lx_el("span",
		lx_attr("class", "hyle-ms-caption"),
		lx_text(label)).data.node;

	container = lx_el("div",
		lx_attr("class", "hyle-ms-options"),
		lx_attr("data-hyle-ms-options", "1")).data.node;
	w->options_container = container;

	for (i = 0; i < w->noptions; i++) {
		bud_node *cb = lx_el("input",
			lx_attr("type", "checkbox"),
			lx_attr("name", key),
			lx_attr("value", w->opts[i].id),
			w->checked[i] ? lx_attr("checked", "") : lx_none(),
			lx_bind("change", 0, hyle_bud_ms_on_change)).data.node;
		bud_node *row = lx_el("label",
			lx_attr("class", "hyle-ms-option"),
			lx_node(cb),
			lx_text(w->opts[i].label)).data.node;
		w->checkboxes[i] = cb;
		w->option_rows[i] = row;
		bud_append(container, row);
	}

	search = lx_el("input",
		lx_attr("type", "search"),
		lx_attr("class", "hyle-ms-search"),
		lx_attr("data-hyle-ms-search", "1"),
		lx_attr("placeholder", "Search\xe2\x80\xa6"),
		lx_attr("aria-label", "Search options"),
		lx_bind("input", 0, hyle_bud_ms_on_search)).data.node;
	w->search = search;

	return lx_el("div",
		lx_attr("class", "hyle-ms-field"),
		lx_node(caption),
		lx_el("details",
			lx_attr("class", "hyle-multiselect"),
			lx_attr("data-hyle-ms", key),
			lx_attr("data-hyle-ms-label", label),
			lx_el("summary", lx_attr("class", "hyle-ms-trigger"),
				lx_node(summary),
				lx_el("span", lx_attr("class", "hyle-ms-caret"),
					lx_attr("aria-hidden", "true"),
					lx_text("\xe2\x96\xbe"))),
			lx_el("div", lx_attr("class", "hyle-ms-panel"),
				lx_attr("data-hyle-ms-panel", "1"),
				lx_node(search),
				lx_node(container)))).data.node;
}

static bud_node *hyle_bud_reference_select(
	const char *key,
	const char *label,
	const char *current_value,
	const hyle_bud_option_t *options,
	int noptions)
{
	char all_label[256];
	int i;

	snprintf(all_label, sizeof(all_label), "All %ss", label);

	bud_node *select = lx_el("select",
		lx_attr("name", key),
		lx_el("option",
			lx_attr("value", ""),
			(!current_value || !current_value[0])
				? lx_attr("selected", "")
				: lx_none(),
			lx_text(all_label))).data.node;

	for (i = 0; i < noptions; i++) {
		int sel = current_value && strcmp(current_value, options[i].id) == 0;
		bud_append(select,
			lx_el("option",
				lx_attr("value", options[i].id),
				sel ? lx_attr("selected", "") : lx_none(),
				lx_text(options[i].label)).data.node);
	}

	return lx_el("label",
		lx_text(label),
		lx_node(select)).data.node;
}

static bud_node *hyle_bud_text_input(
	const char *key,
	const char *label,
	const char *current_value)
{
	return lx_el("label",
		lx_attr("class", "filter-field"),
		lx_text(label),
		lx_text(":"),
		lx_el("input",
			lx_attr("type", "text"),
			lx_attr("name", key),
			lx_attr("placeholder", label),
			current_value && current_value[0]
				? lx_attr("value", current_value)
				: lx_none())).data.node;
}

bud_node *hyle_bud_filter_field(
	const char *key,
	const char *label,
	int type,
	const char *current_value,
	const hyle_bud_option_t *options,
	int noptions,
	const char *filter_style)
{
	switch (type) {
	case HYLE_BUD_BOOL:
		return hyle_bud_boolean_checkbox(key, label, current_value);
	case HYLE_BUD_MULTI_REFERENCE:
		if (options && noptions > 0) {
			if (filter_style && strcmp(filter_style, "dropdown") == 0)
				return hyle_bud_multiselect_field(
				        key, label, current_value, options,
				        noptions);
			return hyle_bud_checkbox_fieldset(
			        key, label, current_value, options, noptions);
		}
		return hyle_bud_text_input(key, label, current_value);
	case HYLE_BUD_REFERENCE:
		if (options && noptions > 0)
			return hyle_bud_reference_select(
			        key, label, current_value, options, noptions);
		return hyle_bud_text_input(key, label, current_value);
	default:
		return hyle_bud_text_input(key, label, current_value);
	}
}
