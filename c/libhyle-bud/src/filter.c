#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <hyle-bud/hyle-bud.h>

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
	int nselected = 0;
	int i;

	if (current_value && current_value[0]) {
		const char *p = current_value;
		while (*p && nselected < 1024) {
			const char *comma = strchr(p, ',');
			size_t len = comma ? (size_t)(comma - p) : strlen(p);
			if (len > 0) {
				selected[nselected] = p;
				nselected++;
			}
			if (!comma)
				break;
			p = comma + 1;
		}
	}

	bud_node *legend = lx_el("legend", lx_text(label)).data.node;
	bud_node *fs = lx_el("fieldset",
		lx_attr("class", "hyle-checkbox-filter"),
		lx_node(legend)).data.node;

	for (i = 0; i < noptions; i++) {
		int checked = 0;
		int j;
		for (j = 0; j < nselected; j++) {
			size_t olen = strlen(options[i].id);
			if (strncmp(selected[j], options[i].id, olen) == 0 &&
			    (selected[j][olen] == '\0' || selected[j][olen] == ','))
			{
				checked = 1;
				break;
			}
		}
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
	int noptions)
{
	switch (type) {
	case HYLE_BUD_BOOL:
		return hyle_bud_boolean_checkbox(key, label, current_value);
	case HYLE_BUD_MULTI_REFERENCE:
		if (options && noptions > 0)
			return hyle_bud_checkbox_fieldset(
				key, label, current_value, options, noptions);
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
