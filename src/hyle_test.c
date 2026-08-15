#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>
#include <locale.h>
#include <stddef.h>
#include "hyle/hyle.h"
#include "hyle/source.h"
#include <ttypt/qmap.h>

static int failures = 0;
static int total = 0;

#define CHECK(cond, msg) do { \
	total++; \
	if (!(cond)) { \
		fprintf(stderr, "FAIL (%s:%d): %s\n", __FILE__, __LINE__, msg); \
		failures++; \
	} else { \
		printf("  ok  %s\n", msg); \
	} \
} while(0)

#define CHECK_JSON(ctx, val, expected) do { \
	char *_j = hyle_val_to_json(ctx, val); \
	CHECK(_j && strcmp(_j, expected) == 0, "json == " expected); \
	free(_j); \
} while(0)

static void test_ctx(void)
{
	printf("=== hyle_ctx_new / free ===\n");
	hyle_ctx_t *ctx = hyle_ctx_new();
	CHECK(ctx != NULL, "ctx created");
	CHECK(ctx->string_pool > 0, "string_pool open");
	CHECK(ctx->array_pool > 0, "array_pool open");
	CHECK(ctx->map_pool > 0, "map_pool open");
	hyle_ctx_free(ctx);
	CHECK(1, "ctx freed without crash");
}

static void test_null(void)
{
	printf("\n=== hyle_val_null ===\n");
	hyle_ctx_t *ctx = hyle_ctx_new();
	hyle_val_t v = hyle_val_null();
	CHECK(v.type == HYLE_NULL, "type is NULL");
	CHECK_JSON(ctx, v, "null");
	hyle_ctx_free(ctx);
}

static void test_bool(void)
{
	printf("\n=== hyle_val_bool ===\n");
	hyle_ctx_t *ctx = hyle_ctx_new();

	hyle_val_t t = hyle_val_bool(true);
	CHECK(t.type == HYLE_BOOL, "type is BOOL");
	CHECK(t.b == true, "value is true");
	CHECK_JSON(ctx, t, "true");

	hyle_val_t f = hyle_val_bool(false);
	CHECK(f.type == HYLE_BOOL, "type is BOOL");
	CHECK(f.b == false, "value is false");
	CHECK_JSON(ctx, f, "false");

	hyle_ctx_free(ctx);
}

static void test_int(void)
{
	printf("\n=== hyle_val_int ===\n");
	hyle_ctx_t *ctx = hyle_ctx_new();

	hyle_val_t v = hyle_val_int(42);
	CHECK(v.type == HYLE_INT, "type is INT");
	CHECK(v.i == 42, "value is 42");
	CHECK_JSON(ctx, v, "42");

	hyle_val_t neg = hyle_val_int(-1);
	CHECK_JSON(ctx, neg, "-1");

	hyle_val_t big = hyle_val_int(9223372036854775807LL);
	CHECK_JSON(ctx, big, "9223372036854775807");

	hyle_ctx_free(ctx);
}

static void test_float(void)
{
	printf("\n=== hyle_val_float ===\n");
	hyle_ctx_t *ctx = hyle_ctx_new();

	hyle_val_t v = hyle_val_float(3.14);
	CHECK(v.type == HYLE_FLOAT, "type is FLOAT");
	char *j = hyle_val_to_json(ctx, v);
	CHECK(j != NULL, "float json not null");
	free(j);

	hyle_ctx_free(ctx);
}

static void test_string(void)
{
	printf("\n=== hyle_val_string ===\n");
	hyle_ctx_t *ctx = hyle_ctx_new();

	hyle_val_t v = hyle_val_string(ctx, "hello world");
	CHECK(v.type == HYLE_STRING, "type is STRING");

	const char *got = hyle_val_string_get(ctx, v);
	CHECK(got != NULL, "string get not null");
	CHECK(strcmp(got, "hello world") == 0, "string get correct");

	CHECK_JSON(ctx, v, "\"hello world\"");

	hyle_val_t esc = hyle_val_string(ctx, "say \"hi\"\nbye");
	CHECK_JSON(ctx, esc, "\"say \\\"hi\\\"\\nbye\"");

	hyle_ctx_free(ctx);
}

static void test_array(void)
{
	printf("\n=== hyle_val_array ===\n");
	hyle_ctx_t *ctx = hyle_ctx_new();

	hyle_val_t arr = hyle_val_array(ctx);
	CHECK(arr.type == HYLE_ARRAY, "type is ARRAY");
	CHECK(hyle_val_array_len(ctx, arr) == 0, "empty array len 0");

	hyle_val_array_push(ctx, arr, hyle_val_int(1));
	hyle_val_array_push(ctx, arr, hyle_val_int(2));
	hyle_val_array_push(ctx, arr, hyle_val_int(3));

	CHECK(hyle_val_array_len(ctx, arr) == 3, "len 3 after pushes");

	char *json = hyle_val_to_json(ctx, arr);
	CHECK(json != NULL, "array json not null");
	CHECK(strcmp(json, "[1,2,3]") == 0, "array json correct");
	free(json);

	hyle_val_array_push(ctx, arr, hyle_val_string(ctx, "four"));

	json = hyle_val_to_json(ctx, arr);
	CHECK(strcmp(json, "[1,2,3,\"four\"]") == 0, "array with string");
	free(json);

	hyle_val_array_push(ctx, arr, hyle_val_bool(true));
	json = hyle_val_to_json(ctx, arr);
	CHECK(strcmp(json, "[1,2,3,\"four\",true]") == 0, "array with bool");
	free(json);

	hyle_ctx_free(ctx);
}

static void test_map(void)
{
	printf("\n=== hyle_val_map ===\n");
	hyle_ctx_t *ctx = hyle_ctx_new();

	hyle_val_t m = hyle_val_map(ctx);
	CHECK(m.type == HYLE_MAP, "type is MAP");

	hyle_val_map_set(ctx, m, "name", hyle_val_string(ctx, "Alice"));
	hyle_val_map_set(ctx, m, "age", hyle_val_int(30));
	hyle_val_map_set(ctx, m, "active", hyle_val_bool(true));

	hyle_val_t name = hyle_val_map_get(ctx, m, "name");
	CHECK(name.type == HYLE_STRING, "got name as STRING");
	const char *ns = hyle_val_string_get(ctx, name);
	CHECK(ns && strcmp(ns, "\"Alice\"") == 0, "name value correct");

	char *json = hyle_val_to_json(ctx, m);
	CHECK(json != NULL, "map json not null");
	CHECK(strstr(json, "\"name\":\"Alice\"") != NULL, "map has name");
	CHECK(strstr(json, "\"age\":30") != NULL, "map has age");
	CHECK(strstr(json, "\"active\":true") != NULL, "map has active");
	CHECK(json[0] == '{', "map json starts with {");
	free(json);

	hyle_ctx_free(ctx);
}

/* ================================================================
 * Phase 2 helpers
 * ================================================================ */

static unsigned make_row_hd(void)
{
	return qmap_open(NULL, NULL, QM_STR, QM_STR, 0xFF, 0);
}

static void row_set_field(hyle_row_set_t *rs,
	const char *id, const char *field, const char *val)
{
	char key[1024];
	snprintf(key, sizeof(key), "%s:%s", id, field);
	qmap_put(rs->fields_hd, key, val);
}

static void row_set_add(hyle_row_set_t *rs, const char *id)
{
	qmap_put(rs->row_hd, id, "");
}

static void build_4rows(hyle_row_set_t *rs)
{
	rs->row_hd = make_row_hd();
	rs->fields_hd = make_row_hd();

	row_set_add(rs, "song1");
	row_set_field(rs, "song1", "title", "Hello World");
	row_set_field(rs, "song1", "author", "Alice");
	row_set_field(rs, "song1", "year", "2020");

	row_set_add(rs, "song2");
	row_set_field(rs, "song2", "title", "Goodbye World");
	row_set_field(rs, "song2", "author", "Bob");
	row_set_field(rs, "song2", "year", "2021");

	row_set_add(rs, "song3");
	row_set_field(rs, "song3", "title", "Hello Again");
	row_set_field(rs, "song3", "author", "Alice");
	row_set_field(rs, "song3", "year", "1999");

	row_set_add(rs, "song4");
	row_set_field(rs, "song4", "title", "Zebra Song");
	row_set_field(rs, "song4", "author", "Charlie");
	row_set_field(rs, "song4", "year", "2005");
}

static void build_25rows(hyle_row_set_t *rs)
{
	rs->row_hd = make_row_hd();
	rs->fields_hd = make_row_hd();

	char id[64];
	for (int i = 1; i <= 25; i++) {
		snprintf(id, sizeof(id), "song%d", i);
		row_set_add(rs, id);
		row_set_field(rs, id, "title", id);
		row_set_field(rs, id, "val", id + 4); /* "1", "2", ... "25" */
	}
}

static int ids_match(const hyle_row_set_t *rs, const char **expected,
	uint32_t expected_count)
{
	uint32_t count = qmap_count(rs->row_hd, NULL);
	if (count != expected_count)
		return 0;

	uint32_t cur = qmap_iter(rs->row_hd, NULL, 0);
	const void *k;
	const void *v;
	uint32_t found = 0;

	while (qmap_next(&k, &v, cur)) {
		found++;
		int ok = 0;
		for (uint32_t i = 0; i < expected_count; i++) {
			if (strcmp((const char *)k, expected[i]) == 0) {
				ok = 1;
				break;
			}
		}
		if (!ok) {
			qmap_fin(cur);
			return 0;
		}
	}
	qmap_fin(cur);
	return found == expected_count;
}

static int ids_in_order(const hyle_row_set_t *rs, const char **expected,
	uint32_t expected_count)
{
	uint32_t count = qmap_count(rs->row_hd, NULL);
	if (count != expected_count)
		return 0;

	uint32_t cur = qmap_iter(rs->row_hd, NULL, 0);
	const void *k;
	const void *v;
	uint32_t pos = 0;
	int ok = 1;

	while (qmap_next(&k, &v, cur)) {
		if (pos >= expected_count) {
			ok = 0;
			break;
		}
		if (strcmp((const char *)k, expected[pos]) != 0) {
			ok = 0;
			break;
		}
		pos++;
	}
	qmap_fin(cur);
	return ok;
}

static void destroy_rows(hyle_row_set_t *rs)
{
	if (rs->row_hd)    qmap_close(rs->row_hd);
	if (rs->fields_hd) qmap_close(rs->fields_hd);
	rs->row_hd = 0;
	rs->fields_hd = 0;
}

#define CHECK_IDS(rs, ...) do { \
	const char *__exp[] = { __VA_ARGS__ }; \
	uint32_t __n = sizeof(__exp) / sizeof(__exp[0]); \
	CHECK(ids_match(&(rs), __exp, __n), "ids match (" #__VA_ARGS__ ")"); \
} while (0)

#define CHECK_ORDER(rs, ...) do { \
	const char *__exp[] = { __VA_ARGS__ }; \
	uint32_t __n = sizeof(__exp) / sizeof(__exp[0]); \
	CHECK(ids_in_order(&(rs), __exp, __n), "ids in order (" #__VA_ARGS__ ")"); \
} while (0)

/* ================================================================
 * Phase 2: hyle_parse_query tests
 * ================================================================ */

static void test_parse_empty(void)
{
	printf("\n=== parse: empty ===\n");
	char buf[] = "";
	hyle_query_t q;
	hyle_parse_query(buf, &q);
	CHECK(q.sort_field == NULL, "sort_field NULL");
	CHECK(q.sort_asc == 1, "sort_asc default true");
	CHECK(q.page == 0, "page default 0");
	CHECK(q.per_page == 0, "per_page default 0");
	CHECK(q.q == NULL, "q NULL");
	CHECK(q.filter_count == 0, "filter_count 0");
	CHECK(q.include_count == 0, "include_count 0");
	hyle_query_clear(&q);
}

static void test_parse_sort(void)
{
	printf("\n=== parse: sort ===\n");
	char buf1[] = "sort=title";
	hyle_query_t q;
	hyle_parse_query(buf1, &q);
	CHECK(q.sort_field != NULL, "sort_field not null");
	CHECK(strcmp(q.sort_field, "title") == 0, "sort_field title");
	CHECK(q.sort_asc == 1, "sort_asc true (default)");
	hyle_query_clear(&q);

	char buf2[] = "sort=title:desc";
	hyle_parse_query(buf2, &q);
	CHECK(strcmp(q.sort_field, "title") == 0, "desc sort_field");
	CHECK(q.sort_asc == 0, "sort_asc false for desc");
	hyle_query_clear(&q);

	char buf3[] = "sort=year:asc";
	hyle_parse_query(buf3, &q);
	CHECK(strcmp(q.sort_field, "year") == 0, "asc sort_field");
	CHECK(q.sort_asc == 1, "sort_asc true for asc");
	hyle_query_clear(&q);
}

static void test_parse_pagination(void)
{
	printf("\n=== parse: pagination ===\n");
	char buf[] = "page=2&per_page=10";
	hyle_query_t q;
	hyle_parse_query(buf, &q);
	CHECK(q.page == 2, "page 2");
	CHECK(q.per_page == 10, "per_page 10");
	hyle_query_clear(&q);
}

static void test_parse_q(void)
{
	printf("\n=== parse: q ===\n");
	char buf[] = "q=love";
	hyle_query_t q;
	hyle_parse_query(buf, &q);
	CHECK(q.q != NULL, "q not null");
	CHECK(strcmp(q.q, "love") == 0, "q = love");
	hyle_query_clear(&q);
}

static void test_parse_include(void)
{
	printf("\n=== parse: include ===\n");
	char buf[] = "include=id,title,author";
	hyle_query_t q;
	hyle_parse_query(buf, &q);
	CHECK(q.include_count == 3, "include 3 fields");
	CHECK(strcmp(q.include[0], "id") == 0, "include[0] id");
	CHECK(strcmp(q.include[1], "title") == 0, "include[1] title");
	CHECK(strcmp(q.include[2], "author") == 0, "include[2] author");
	hyle_query_clear(&q);
}

static void test_parse_field_filter(void)
{
	printf("\n=== parse: field filter ===\n");
	char buf[] = "author=Beatles";
	hyle_query_t q;
	hyle_parse_query(buf, &q);
	CHECK(q.filter_count == 1, "filter_count 1");
	CHECK(strcmp(q.filters[0].field, "author") == 0, "filter field author");
	CHECK(strcmp(q.filters[0].value, "Beatles") == 0, "filter value Beatles");
	hyle_query_clear(&q);
}

static void test_parse_multiple_filters(void)
{
	printf("\n=== parse: multiple filters ===\n");
	char buf[] = "author=Alice&year=2020";
	hyle_query_t q;
	hyle_parse_query(buf, &q);
	CHECK(q.filter_count == 2, "filter_count 2");
	CHECK(strcmp(q.filters[0].field, "author") == 0, "filter0 author");
	CHECK(strcmp(q.filters[0].value, "Alice") == 0, "filter0 Alice");
	CHECK(strcmp(q.filters[1].field, "year") == 0, "filter1 year");
	CHECK(strcmp(q.filters[1].value, "2020") == 0, "filter1 2020");
	hyle_query_clear(&q);
}

static void test_parse_all_params(void)
{
	printf("\n=== parse: all params ===\n");
	char buf[] = "sort=title:desc&page=1&per_page=20&q=love"
		"&include=id,title&author=Alice";
	hyle_query_t q;
	hyle_parse_query(buf, &q);
	CHECK(q.sort_field != NULL, "sort_field set");
	CHECK(strcmp(q.sort_field, "title") == 0, "sort title");
	CHECK(q.sort_asc == 0, "sort desc");
	CHECK(q.page == 1, "page 1");
	CHECK(q.per_page == 20, "per_page 20");
	CHECK(q.q != NULL && strcmp(q.q, "love") == 0, "q love");
	CHECK(q.include_count == 2, "include 2 fields");
	CHECK(q.filter_count == 1, "filter 1");
	CHECK(strcmp(q.filters[0].field, "author") == 0, "author filter");
	hyle_query_clear(&q);
}

static void test_parse_unknown_becomes_filter(void)
{
	printf("\n=== parse: unknown param becomes filter ===\n");
	char buf[] = "unknown=value&also=test";
	hyle_query_t q;
	hyle_parse_query(buf, &q);
	CHECK(q.filter_count == 2, "2 unknown params become filters");
	hyle_query_clear(&q);
}

/* ================================================================
 * Phase 2: hyle_filter_rows tests
 * ================================================================ */

static void test_filter_single_match(void)
{
	printf("\n=== filter: single field match ===\n");
	hyle_ctx_t *ctx = hyle_ctx_new();
	hyle_row_set_t input;
	build_4rows(&input);

	hyle_field_filter_t filters[] = {
		{ "author", "Alice" }
	};
	hyle_row_set_t output = { make_row_hd(), 0 };
	hyle_filter_rows(ctx, &input, NULL, filters, 1, NULL, 0, &output);

	CHECK_IDS(output, "song1", "song3");

	destroy_rows(&input);
	qmap_close(output.row_hd);
	hyle_ctx_free(ctx);
}

static void test_filter_no_match(void)
{
	printf("\n=== filter: no match ===\n");
	hyle_ctx_t *ctx = hyle_ctx_new();
	hyle_row_set_t input;
	build_4rows(&input);

	hyle_field_filter_t filters[] = {
		{ "author", "Nobody" }
	};
	hyle_row_set_t output = { make_row_hd(), 0 };
	hyle_filter_rows(ctx, &input, NULL, filters, 1, NULL, 0, &output);

	CHECK(qmap_count(output.row_hd, NULL) == 0, "empty result");

	destroy_rows(&input);
	qmap_close(output.row_hd);
	hyle_ctx_free(ctx);
}

static void test_filter_multi_and(void)
{
	printf("\n=== filter: multiple AND ===\n");
	hyle_ctx_t *ctx = hyle_ctx_new();
	hyle_row_set_t input;
	build_4rows(&input);

	hyle_field_filter_t filters[] = {
		{ "author", "Alice" },
		{ "year", "2020" }
	};
	hyle_row_set_t output = { make_row_hd(), 0 };
	hyle_filter_rows(ctx, &input, NULL, filters, 2, NULL, 0, &output);

	CHECK_IDS(output, "song1");

	destroy_rows(&input);
	qmap_close(output.row_hd);
	hyle_ctx_free(ctx);
}

static void test_filter_fulltext(void)
{
	printf("\n=== filter: full-text q ===\n");
	hyle_ctx_t *ctx = hyle_ctx_new();
	hyle_row_set_t input;
	build_4rows(&input);

	hyle_row_set_t output = { make_row_hd(), 0 };
	hyle_filter_rows(ctx, &input, "hello", NULL, 0, NULL, 0, &output);

	/* song1 title=Hello World, song3 title=Hello Again */
	CHECK_IDS(output, "song1", "song3");

	destroy_rows(&input);
	qmap_close(output.row_hd);
	hyle_ctx_free(ctx);
}

static void test_filter_fulltext_no_match(void)
{
	printf("\n=== filter: full-text no match ===\n");
	hyle_ctx_t *ctx = hyle_ctx_new();
	hyle_row_set_t input;
	build_4rows(&input);

	hyle_row_set_t output = { make_row_hd(), 0 };
	hyle_filter_rows(ctx, &input, "zzzzz", NULL, 0, NULL, 0, &output);

	CHECK(qmap_count(output.row_hd, NULL) == 0, "empty result");

	destroy_rows(&input);
	qmap_close(output.row_hd);
	hyle_ctx_free(ctx);
}

static void test_filter_q_and_field(void)
{
	printf("\n=== filter: q + field filter ===\n");
	hyle_ctx_t *ctx = hyle_ctx_new();
	hyle_row_set_t input;
	build_4rows(&input);

	hyle_field_filter_t filters[] = {
		{ "author", "Alice" }
	};
	hyle_row_set_t output = { make_row_hd(), 0 };
	hyle_filter_rows(ctx, &input, "hello", filters, 1, NULL, 0, &output);

	/* Alice songs: song1 (Hello World), song3 (Hello Again)
	   q=hello matches both titles */
	CHECK_IDS(output, "song1", "song3");

	destroy_rows(&input);
	qmap_close(output.row_hd);
	hyle_ctx_free(ctx);
}

static void test_filter_accent_folding(void)
{
	printf("\n=== filter: accent folding ===\n");
	hyle_ctx_t *ctx = hyle_ctx_new();
	hyle_row_set_t input;

	input.row_hd = make_row_hd();
	input.fields_hd = make_row_hd();

	row_set_add(&input, "songA");
	row_set_field(&input, "songA", "title", "Coração Adorador");
	row_set_add(&input, "songB");
	row_set_field(&input, "songB", "title", "Maçã Verde");

	hyle_field_filter_t f1[] = { { "title", "Coração" } };
	hyle_field_filter_t f2[] = { { "title", "coracao" } };
	hyle_field_filter_t f3[] = { { "title", "maca" } };
	hyle_field_filter_t f4[] = { { "title", "Verde Verde" } };

	hyle_row_set_t o1 = { make_row_hd(), 0 };
	hyle_filter_rows(ctx, &input, NULL, f1, 1, NULL, 0, &o1);
	CHECK_IDS(o1, "songA");

	hyle_row_set_t o2 = { make_row_hd(), 0 };
	hyle_filter_rows(ctx, &input, NULL, f2, 1, NULL, 0, &o2);
	CHECK_IDS(o2, "songA");

	hyle_row_set_t o3 = { make_row_hd(), 0 };
	hyle_filter_rows(ctx, &input, NULL, f3, 1, NULL, 0, &o3);
	CHECK_IDS(o3, "songB");

	hyle_row_set_t o4 = { make_row_hd(), 0 };
	hyle_filter_rows(ctx, &input, NULL, f4, 1, NULL, 0, &o4);
	CHECK(qmap_count(o4.row_hd, NULL) == 0, "no match");

	hyle_row_set_t o5 = { make_row_hd(), 0 };
	hyle_filter_rows(ctx, &input, "Maçã", NULL, 0, NULL, 0, &o5);
	CHECK_IDS(o5, "songB");

	hyle_row_set_t o6 = { make_row_hd(), 0 };
	hyle_filter_rows(ctx, &input, "maca", NULL, 0, NULL, 0, &o6);
	CHECK_IDS(o6, "songB");

	qmap_close(o1.row_hd);
	qmap_close(o2.row_hd);
	qmap_close(o3.row_hd);
	qmap_close(o4.row_hd);
	qmap_close(o5.row_hd);
	qmap_close(o6.row_hd);
	destroy_rows(&input);
	hyle_ctx_free(ctx);
}

/* ================================================================
 * Phase 2: hyle_sort_rows tests
 * ================================================================ */

static void test_sort_string_asc(void)
{
	printf("\n=== sort: string asc ===\n");
	hyle_ctx_t *ctx = hyle_ctx_new();
	hyle_row_set_t input;
	build_4rows(&input);

	hyle_row_set_t output = { make_row_hd(), 0 };
	hyle_sort_rows(ctx, &input, "title", 1, &output);

	/* titles: "Goodbye World", "Hello Again", "Hello World", "Zebra Song" */
	CHECK_ORDER(output, "song2", "song3", "song1", "song4");

	destroy_rows(&input);
	qmap_close(output.row_hd);
	hyle_ctx_free(ctx);
}

static void test_sort_string_desc(void)
{
	printf("\n=== sort: string desc ===\n");
	hyle_ctx_t *ctx = hyle_ctx_new();
	hyle_row_set_t input;
	build_4rows(&input);

	hyle_row_set_t output = { make_row_hd(), 0 };
	hyle_sort_rows(ctx, &input, "title", 0, &output);

	CHECK_ORDER(output, "song4", "song1", "song3", "song2");

	destroy_rows(&input);
	qmap_close(output.row_hd);
	hyle_ctx_free(ctx);
}

static void test_sort_numeric_asc(void)
{
	printf("\n=== sort: numeric asc ===\n");
	hyle_ctx_t *ctx = hyle_ctx_new();
	hyle_row_set_t input;
	build_4rows(&input);

	hyle_row_set_t output = { make_row_hd(), 0 };
	hyle_sort_rows(ctx, &input, "year", 1, &output);

	/* years: 1999, 2005, 2020, 2021 */
	CHECK_ORDER(output, "song3", "song4", "song1", "song2");

	destroy_rows(&input);
	qmap_close(output.row_hd);
	hyle_ctx_free(ctx);
}

static void test_sort_numeric_desc(void)
{
	printf("\n=== sort: numeric desc ===\n");
	hyle_ctx_t *ctx = hyle_ctx_new();
	hyle_row_set_t input;
	build_4rows(&input);

	hyle_row_set_t output = { make_row_hd(), 0 };
	hyle_sort_rows(ctx, &input, "year", 0, &output);

	CHECK_ORDER(output, "song2", "song1", "song4", "song3");

	destroy_rows(&input);
	qmap_close(output.row_hd);
	hyle_ctx_free(ctx);
}

static void test_sort_noop(void)
{
	printf("\n=== sort: NULL field (passthrough) ===\n");
	hyle_ctx_t *ctx = hyle_ctx_new();
	hyle_row_set_t input;
	build_4rows(&input);

	hyle_row_set_t output = { make_row_hd(), 0 };
	hyle_sort_rows(ctx, &input, NULL, 1, &output);

	CHECK(qmap_count(output.row_hd, NULL) == 4, "all 4 rows passthrough");

	destroy_rows(&input);
	qmap_close(output.row_hd);
	hyle_ctx_free(ctx);
}

/* ================================================================
 * Phase 2: hyle_paginate tests
 * ================================================================ */

static void test_paginate_page1(void)
{
	printf("\n=== paginate: page 1 ===\n");
	hyle_ctx_t *ctx = hyle_ctx_new();
	hyle_row_set_t input;
	build_25rows(&input);

	hyle_row_set_t output = { make_row_hd(), 0 };
	uint32_t got_total = 0;
	hyle_paginate(ctx, &input, 1, 10, &output, &got_total);

	CHECK(got_total == 25, "total 25");
	CHECK(qmap_count(output.row_hd, NULL) == 10, "page 1 has 10 rows");

	destroy_rows(&input);
	qmap_close(output.row_hd);
	hyle_ctx_free(ctx);
}

static void test_paginate_page3(void)
{
	printf("\n=== paginate: page 3 of 10 ===\n");
	hyle_ctx_t *ctx = hyle_ctx_new();
	hyle_row_set_t input;
	build_25rows(&input);

	hyle_row_set_t output = { make_row_hd(), 0 };
	uint32_t got_total = 0;
	hyle_paginate(ctx, &input, 3, 10, &output, &got_total);

	CHECK(got_total == 25, "total 25");
	CHECK(qmap_count(output.row_hd, NULL) == 5, "page 3 has 5 rows");

	destroy_rows(&input);
	qmap_close(output.row_hd);
	hyle_ctx_free(ctx);
}

static void test_paginate_beyond(void)
{
	printf("\n=== paginate: page beyond total ===\n");
	hyle_ctx_t *ctx = hyle_ctx_new();
	hyle_row_set_t input;
	build_25rows(&input);

	hyle_row_set_t output = { make_row_hd(), 0 };
	uint32_t got_total = 0;
	hyle_paginate(ctx, &input, 10, 10, &output, &got_total);

	CHECK(got_total == 25, "total 25");
	CHECK(qmap_count(output.row_hd, NULL) == 0, "empty result");

	destroy_rows(&input);
	qmap_close(output.row_hd);
	hyle_ctx_free(ctx);
}

static void test_paginate_all(void)
{
	printf("\n=== paginate: per_page=0 (all) ===\n");
	hyle_ctx_t *ctx = hyle_ctx_new();
	hyle_row_set_t input;
	build_25rows(&input);

	hyle_row_set_t output = { make_row_hd(), 0 };
	uint32_t got_total = 0;
	hyle_paginate(ctx, &input, 1, 0, &output, &got_total);

	CHECK(got_total == 25, "total 25");
	CHECK(qmap_count(output.row_hd, NULL) == 25, "all 25 rows");

	destroy_rows(&input);
	qmap_close(output.row_hd);
	hyle_ctx_free(ctx);
}

/* ================================================================
 * Phase 2: hyle_apply_view tests
 * ================================================================ */

static void test_apply_view_full(void)
{
	printf("\n=== apply_view: filter+sort+paginate ===\n");
	hyle_ctx_t *ctx = hyle_ctx_new();
	hyle_row_set_t input;
	build_25rows(&input);

	char qstr[] = "sort=val&page=1&per_page=5";
	hyle_query_t q;
	hyle_parse_query(qstr, &q);

	hyle_row_set_t output = { 0, 0 };
	uint32_t got_total = 0;
	hyle_apply_view(ctx, &input, &q, NULL, 0, &output, &got_total);

	CHECK(got_total == 25, "total 25");
	CHECK(qmap_count(output.row_hd, NULL) == 5, "5 rows page 1");
	/* sorted by val numeric asc: song1(1), song2(2), song3(3), ... */
	CHECK_ORDER(output, "song1", "song2", "song3", "song4", "song5");
	CHECK(output.fields_hd == input.fields_hd, "fields_hd shared");

	qmap_close(output.row_hd);
	hyle_query_clear(&q);
	destroy_rows(&input);
	hyle_ctx_free(ctx);
}

static void test_apply_view_noop(void)
{
	printf("\n=== apply_view: no params (passthrough) ===\n");
	hyle_ctx_t *ctx = hyle_ctx_new();
	hyle_row_set_t input;
	build_4rows(&input);

	char qstr[] = "";
	hyle_query_t q;
	hyle_parse_query(qstr, &q);

	hyle_row_set_t output = { 0, 0 };
	uint32_t got_total = 0;
	hyle_apply_view(ctx, &input, &q, NULL, 0, &output, &got_total);

	CHECK(got_total == 4, "total 4");
	CHECK(qmap_count(output.row_hd, NULL) == 4, "all 4 rows");
	CHECK_IDS(output, "song1", "song2", "song3", "song4");

	qmap_close(output.row_hd);
	hyle_query_clear(&q);
	destroy_rows(&input);
	hyle_ctx_free(ctx);
}

static void test_apply_view_filter_only(void)
{
	printf("\n=== apply_view: filter only ===\n");
	hyle_ctx_t *ctx = hyle_ctx_new();
	hyle_row_set_t input;
	build_4rows(&input);

	char qstr[] = "author=Alice";
	hyle_query_t q;
	hyle_parse_query(qstr, &q);

	hyle_row_set_t output = { 0, 0 };
	uint32_t got_total = 0;
	hyle_apply_view(ctx, &input, &q, NULL, 0, &output, &got_total);

	CHECK(got_total == 2, "total 2");
	CHECK_IDS(output, "song1", "song3");

	qmap_close(output.row_hd);
	hyle_query_clear(&q);
	destroy_rows(&input);
	hyle_ctx_free(ctx);
}

static void test_apply_view_sort_only(void)
{
	printf("\n=== apply_view: sort only ===\n");
	hyle_ctx_t *ctx = hyle_ctx_new();
	hyle_row_set_t input;
	build_4rows(&input);

	char qstr[] = "sort=title";
	hyle_query_t q;
	hyle_parse_query(qstr, &q);

	hyle_row_set_t output = { 0, 0 };
	uint32_t got_total = 0;
	hyle_apply_view(ctx, &input, &q, NULL, 0, &output, &got_total);

	CHECK(got_total == 4, "total 4");
	CHECK_ORDER(output, "song2", "song3", "song1", "song4");

	qmap_close(output.row_hd);
	hyle_query_clear(&q);
	destroy_rows(&input);
	hyle_ctx_free(ctx);
}

/* ================================================================
 * Phase 3: blueprint + manifest tests
 * ================================================================ */

static const hyle_field_t test_fields[] = {
	{ "id",         HYLE_FIELD_STRING,  1, NULL,             NULL,       1, 0,    0,    0, 0, NULL, 0 },
	{ "title",      HYLE_FIELD_STRING,  1, NULL,             NULL,       1, 0,    0,    0, 0, NULL, 0 },
	{ "author",     HYLE_FIELD_STRING,  1, NULL,             NULL,       0, 0,    0,    0, 0, NULL, 0 },
	{ "year",       HYLE_FIELD_INT,     1, NULL,             NULL,       0, 1900, 2100, 0, 0, NULL, 0 },
	{ "album",      HYLE_FIELD_STRING,  1, NULL,             NULL,       0, 0,    0,    0, 0, NULL, 0 },
	{ "artist_id",  HYLE_FIELD_REFERENCE,        1, "artist.items",  NULL,       0, 0,    0,    0, 0, NULL, 0 },
	{ "tags",       HYLE_FIELD_MULTI_REFERENCE,  1, "tag.items",    NULL,       0, 0,    0,    0, 0, NULL, 0 },
	{ "songs",      HYLE_FIELD_INVERSE, 0, "test.songs",   "artist_id", 0, 0,    0,    0, 0, NULL, 0 },
};

static const hyle_source_schema_t test_schema = {
	"test.songs",
	"id",
	test_fields,
	8,
	0,
};

static const hyle_blueprint_t test_bp = {
	(hyle_source_schema_t *)&test_schema,
	1,
};

static void test_field_lookup(void)
{
	printf("\n=== field: hyle_field_by_name ===\n");
	const hyle_field_t *f = hyle_field_by_name(test_fields, 8, "title");
	CHECK(f != NULL, "find title");
	CHECK(strcmp(f->name, "title") == 0, "name is title");
	CHECK(f->type == HYLE_FIELD_STRING, "type STRING");

	f = hyle_field_by_name(test_fields, 8, "year");
	CHECK(f != NULL, "find year");
	CHECK(f->type == HYLE_FIELD_INT, "type INT");

	f = hyle_field_by_name(test_fields, 8, "artist_id");
	CHECK(f != NULL, "find artist_id");
	CHECK(f->type == HYLE_FIELD_REFERENCE, "type REFERENCE");

	f = hyle_field_by_name(test_fields, 8, "nonexistent");
	CHECK(f == NULL, "not found returns NULL");
}

static void test_field_is_reference(void)
{
	printf("\n=== field: hyle_field_is_reference ===\n");
	CHECK(hyle_field_is_reference(HYLE_FIELD_REFERENCE), "REFERENCE is ref");
	CHECK(hyle_field_is_reference(HYLE_FIELD_MULTI_REFERENCE), "MULTI_REFERENCE is ref");
	CHECK(hyle_field_is_reference(HYLE_FIELD_INVERSE), "INVERSE is ref");
	CHECK(!hyle_field_is_reference(HYLE_FIELD_STRING), "STRING not ref");
	CHECK(!hyle_field_is_reference(HYLE_FIELD_INT), "INT not ref");
	CHECK(!hyle_field_is_reference(HYLE_FIELD_BOOL), "BOOL not ref");
}

static void test_manifest_valid(void)
{
	printf("\n=== manifest: valid all params ===\n");
	char qstr[] = "sort=title:asc&page=2&per_page=10&q=love&include=title,author";
	hyle_query_t q;
	hyle_parse_query(qstr, &q);

	hyle_manifest_t m;
	int rc = hyle_blueprint_manifest(&test_bp, "test.songs", &q, &m);
	CHECK(rc == 0, "manifest ok");
	CHECK(m.base_source != NULL, "base_source set");
	CHECK(strcmp(m.base_source, "test.songs") == 0, "base_source correct");
	CHECK(m.sort_field != NULL, "sort_field set");
	CHECK(strcmp(m.sort_field, "title") == 0, "sort_field title");
	CHECK(m.sort_asc == 1, "sort asc");
	CHECK(m.page == 2, "page 2");
	CHECK(m.per_page == 10, "per_page 10");
	CHECK(m.query != NULL, "query set");
	CHECK(strcmp(m.query, "love") == 0, "q love");
	CHECK(m.select_count == 2, "select_count 2");
	CHECK(strcmp(m.select[0], "title") == 0, "select[0] title");
	CHECK(strcmp(m.select[1], "author") == 0, "select[1] author");
	hyle_manifest_clear(&m);
	hyle_query_clear(&q);
}

static void test_manifest_unknown_filter_field(void)
{
	printf("\n=== manifest: unknown filter field ===\n");
	char qstr[] = "nonexistent=value";
	hyle_query_t q;
	hyle_parse_query(qstr, &q);
	hyle_manifest_t m;
	int rc = hyle_blueprint_manifest(&test_bp, "test.songs", &q, &m);
	CHECK(rc == -1, "unknown filter field returns -1");
	hyle_query_clear(&q);
}

static void test_manifest_unknown_sort_field(void)
{
	printf("\n=== manifest: unknown sort field ===\n");
	char qstr[] = "sort=nonexistent";
	hyle_query_t q;
	hyle_parse_query(qstr, &q);
	hyle_manifest_t m;
	int rc = hyle_blueprint_manifest(&test_bp, "test.songs", &q, &m);
	CHECK(rc == -1, "unknown sort field returns -1");
	hyle_query_clear(&q);
}

static void test_manifest_unknown_include_field(void)
{
	printf("\n=== manifest: unknown include field ===\n");
	char qstr[] = "include=nonexistent";
	hyle_query_t q;
	hyle_parse_query(qstr, &q);
	hyle_manifest_t m;
	int rc = hyle_blueprint_manifest(&test_bp, "test.songs", &q, &m);
	CHECK(rc == -1, "unknown include field returns -1");
	hyle_query_clear(&q);
}

static void test_manifest_source_not_found(void)
{
	printf("\n=== manifest: source not found ===\n");
	hyle_query_t q;
	memset(&q, 0, sizeof(q));
	hyle_manifest_t m;
	int rc = hyle_blueprint_manifest(&test_bp, "nonexistent.source", &q, &m);
	CHECK(rc == -1, "source not found returns -1");
}

static void test_manifest_null_params(void)
{
	printf("\n=== manifest: NULL params ===\n");
	hyle_manifest_t m;
	int rc = hyle_blueprint_manifest(NULL, NULL, NULL, &m);
	CHECK(rc == -1, "NULL bp/query returns -1");
}

static void test_manifest_empty_query(void)
{
	printf("\n=== manifest: empty query ===\n");
	char qstr[] = "";
	hyle_query_t q;
	hyle_parse_query(qstr, &q);
	hyle_manifest_t m;
	int rc = hyle_blueprint_manifest(&test_bp, "test.songs", &q, &m);
	CHECK(rc == 0, "empty query ok");
	CHECK(m.base_source != NULL, "base_source set");
	CHECK(m.filter_count == 0, "no filters");
	CHECK(m.sort_field == NULL, "no sort");
	CHECK(m.page == 0, "page 0");
	CHECK(m.per_page == 0, "per_page 0");
	hyle_manifest_clear(&m);
	hyle_query_clear(&q);
}

static void test_manifest_reference_lookup(void)
{
	printf("\n=== manifest: ref filter classified as lookup ===\n");
	char qstr[] = "artist_id=abc123";
	hyle_query_t q;
	hyle_parse_query(qstr, &q);
	hyle_manifest_t m;
	int rc = hyle_blueprint_manifest(&test_bp, "test.songs", &q, &m);
	CHECK(rc == 0, "manifest ok");
	CHECK(m.filter_count == 1, "one filter");
	CHECK(m.filter_list[0].is_reference == 1, "filter is reference");
	CHECK(m.lookup_count == 1, "one lookup");
	CHECK(strcmp(m.lookups[0], "artist.items") == 0, "lookup artist.items");
	CHECK(m.inline_count == 0, "no inlines");
	hyle_manifest_clear(&m);
	hyle_query_clear(&q);
}

static void test_manifest_reference_inline(void)
{
	printf("\n=== manifest: ref include classified as inline ===\n");
	char qstr[] = "include=artist_id";
	hyle_query_t q;
	hyle_parse_query(qstr, &q);
	hyle_manifest_t m;
	int rc = hyle_blueprint_manifest(&test_bp, "test.songs", &q, &m);
	CHECK(rc == 0, "manifest ok");
	CHECK(m.select_count == 1, "select 1");
	CHECK(strcmp(m.select[0], "artist_id") == 0, "select artist_id");
	CHECK(m.inline_count == 1, "one inline");
	CHECK(strcmp(m.inlines[0], "artist.items") == 0, "inline artist.items");
	CHECK(m.lookup_count == 0, "no lookups");
	hyle_manifest_clear(&m);
	hyle_query_clear(&q);
}

static void test_manifest_dedup_ref_targets(void)
{
	printf("\n=== manifest: dedup ref targets ===\n");
	char qstr[] = "artist_id=abc&include=artist_id";
	hyle_query_t q;
	hyle_parse_query(qstr, &q);
	hyle_manifest_t m;
	int rc = hyle_blueprint_manifest(&test_bp, "test.songs", &q, &m);
	CHECK(rc == 0, "manifest ok");
	CHECK(m.lookup_count == 1, "one lookup");
	CHECK(m.inline_count == 1, "one inline");
	CHECK(strcmp(m.lookups[0], "artist.items") == 0, "lookup artist.items");
	CHECK(strcmp(m.inlines[0], "artist.items") == 0, "inline artist.items");
	hyle_manifest_clear(&m);
	hyle_query_clear(&q);
}

static void test_manifest_multi_reference(void)
{
	printf("\n=== manifest: MULTI_REFERENCE field ===\n");
	char qstr[] = "tags=jazz";
	hyle_query_t q;
	hyle_parse_query(qstr, &q);
	hyle_manifest_t m;
	int rc = hyle_blueprint_manifest(&test_bp, "test.songs", &q, &m);
	CHECK(rc == 0, "manifest ok");
	CHECK(m.filter_count == 1, "one filter");
	CHECK(m.filter_list[0].is_reference == 1, "filter is reference");
	CHECK(m.lookup_count == 1, "one lookup");
	CHECK(strcmp(m.lookups[0], "tag.items") == 0, "lookup tag.items");
	hyle_manifest_clear(&m);
	hyle_query_clear(&q);
}

static void test_manifest_inverse_field(void)
{
	printf("\n=== manifest: INVERSE field ===\n");
	char qstr[] = "include=songs";
	hyle_query_t q;
	hyle_parse_query(qstr, &q);
	hyle_manifest_t m;
	int rc = hyle_blueprint_manifest(&test_bp, "test.songs", &q, &m);
	CHECK(rc == 0, "manifest ok");
	CHECK(m.select_count == 1, "select 1");
	CHECK(m.inline_count == 1, "one inline");
	CHECK(strcmp(m.inlines[0], "test.songs") == 0, "inline test.songs (self)");
	hyle_manifest_clear(&m);
	hyle_query_clear(&q);
}

static void test_manifest_empty_blueprint(void)
{
	printf("\n=== manifest: empty blueprint ===\n");
	hyle_blueprint_t empty = { NULL, 0 };
	hyle_query_t q;
	memset(&q, 0, sizeof(q));
	hyle_manifest_t m;
	int rc = hyle_blueprint_manifest(&empty, "test.songs", &q, &m);
	CHECK(rc == -1, "empty blueprint returns -1");
}

/* ================================================================
 * Phase 6 — purify
 * ================================================================ */

static void test_purify_required(void)
{
	printf("\n=== hyle_purify_row — required ===\n");
	hyle_field_t fields[] = {
		{"name", HYLE_FIELD_STRING, 1, NULL, NULL, 1, 0, 0, 0, 0, NULL, 0},
	};
	const char *values[] = {NULL};
	hyle_purify_error_t *errs;
	size_t nerr;
	int rc = hyle_purify_row(fields, 1, values, &errs, &nerr);
	CHECK(rc != 0, "required field missing → error");
	CHECK(nerr == 1, "one error");
	CHECK(strcmp(errs[0].field, "name") == 0, "field is 'name'");
	CHECK(strcmp(errs[0].rule, "required") == 0, "rule is 'required'");
	hyle_purify_errors_free(errs, nerr);

	/* With value present */
	const char *values2[] = {"Alice"};
	rc = hyle_purify_row(fields, 1, values2, &errs, &nerr);
	CHECK(rc == 0, "required field present → ok");
}

static void test_purify_min(void)
{
	printf("\n=== hyle_purify_row — min ===\n");
	hyle_field_t fields[] = {
		{"age", HYLE_FIELD_INT, 1, NULL, NULL, 0, 18, 0, 0, 0, NULL, 0},
	};
	const char *values[] = {"15"};
	hyle_purify_error_t *errs;
	size_t nerr;
	int rc = hyle_purify_row(fields, 1, values, &errs, &nerr);
	CHECK(rc != 0, "value below min → error");
	CHECK(strcmp(errs[0].rule, "min") == 0, "rule is 'min'");
	hyle_purify_errors_free(errs, nerr);

	const char *values2[] = {"18"};
	rc = hyle_purify_row(fields, 1, values2, &errs, &nerr);
	CHECK(rc == 0, "value equal to min → ok");

	const char *values3[] = {"25"};
	rc = hyle_purify_row(fields, 1, values3, &errs, &nerr);
	CHECK(rc == 0, "value above min → ok");
}

static void test_purify_max(void)
{
	printf("\n=== hyle_purify_row — max ===\n");
	hyle_field_t fields[] = {
		{"score", HYLE_FIELD_INT, 1, NULL, NULL, 0, 0, 100, 0, 0, NULL, 0},
	};
	const char *values[] = {"101"};
	hyle_purify_error_t *errs;
	size_t nerr;
	int rc = hyle_purify_row(fields, 1, values, &errs, &nerr);
	CHECK(rc != 0, "value above max → error");
	CHECK(strcmp(errs[0].rule, "max") == 0, "rule is 'max'");
	hyle_purify_errors_free(errs, nerr);

	const char *values2[] = {"100"};
	rc = hyle_purify_row(fields, 1, values2, &errs, &nerr);
	CHECK(rc == 0, "value equal to max → ok");

	const char *values3[] = {"50"};
	rc = hyle_purify_row(fields, 1, values3, &errs, &nerr);
	CHECK(rc == 0, "value below max → ok");
}

static void test_purify_min_length(void)
{
	printf("\n=== hyle_purify_row — minLength ===\n");
	hyle_field_t fields[] = {
		{"name", HYLE_FIELD_STRING, 1, NULL, NULL, 0, 0, 0, 3, 0, NULL, 0},
	};
	const char *values[] = {"ab"};
	hyle_purify_error_t *errs;
	size_t nerr;
	int rc = hyle_purify_row(fields, 1, values, &errs, &nerr);
	CHECK(rc != 0, "too short → error");
	CHECK(strcmp(errs[0].rule, "minLength") == 0, "rule is 'minLength'");
	hyle_purify_errors_free(errs, nerr);

	const char *values2[] = {"abc"};
	rc = hyle_purify_row(fields, 1, values2, &errs, &nerr);
	CHECK(rc == 0, "exact min length → ok");

	const char *values3[] = {"abcd"};
	rc = hyle_purify_row(fields, 1, values3, &errs, &nerr);
	CHECK(rc == 0, "above min length → ok");
}

static void test_purify_max_length(void)
{
	printf("\n=== hyle_purify_row — maxLength ===\n");
	hyle_field_t fields[] = {
		{"code", HYLE_FIELD_STRING, 1, NULL, NULL, 0, 0, 0, 0, 3, NULL, 0},
	};
	const char *values[] = {"abcd"};
	hyle_purify_error_t *errs;
	size_t nerr;
	int rc = hyle_purify_row(fields, 1, values, &errs, &nerr);
	CHECK(rc != 0, "too long → error");
	CHECK(strcmp(errs[0].rule, "maxLength") == 0, "rule is 'maxLength'");
	hyle_purify_errors_free(errs, nerr);

	const char *values2[] = {"abc"};
	rc = hyle_purify_row(fields, 1, values2, &errs, &nerr);
	CHECK(rc == 0, "exact max length → ok");

	const char *values3[] = {"ab"};
	rc = hyle_purify_row(fields, 1, values3, &errs, &nerr);
	CHECK(rc == 0, "below max length → ok");
}

static void test_purify_pattern(void)
{
	printf("\n=== hyle_purify_row — pattern ===\n");
	hyle_field_t fields[] = {
		{"code", HYLE_FIELD_STRING, 1, NULL, NULL, 0, 0, 0, 0, 0, "^[A-Z]{3}$", 0},
	};
	const char *values[] = {"ABC"};
	hyle_purify_error_t *errs;
	size_t nerr;
	int rc = hyle_purify_row(fields, 1, values, &errs, &nerr);
	CHECK(rc == 0, "matching pattern → ok");
	hyle_purify_errors_free(errs, nerr);

	const char *values2[] = {"abcd"};
	rc = hyle_purify_row(fields, 1, values2, &errs, &nerr);
	CHECK(rc != 0, "non-matching pattern → error");
	CHECK(strcmp(errs[0].rule, "pattern") == 0, "rule is 'pattern'");
	hyle_purify_errors_free(errs, nerr);
}

static void test_purify_valid(void)
{
	printf("\n=== hyle_purify_row — valid row ===\n");
	hyle_field_t fields[] = {
		{"name", HYLE_FIELD_STRING, 1, NULL, NULL, 1, 0, 0, 2, 0, NULL, 0},
		{"age",  HYLE_FIELD_INT,    1, NULL, NULL, 0, 18, 99, 0, 0, NULL, 0},
	};
	const char *values[] = {"Alice", "30"};
	hyle_purify_error_t *errs;
	size_t nerr;
	int rc = hyle_purify_row(fields, 2, values, &errs, &nerr);
	CHECK(rc == 0, "valid row → ok");
	CHECK(errs == NULL, "no errors");
}

static void test_purify_multiple_errors(void)
{
	printf("\n=== hyle_purify_row — multiple errors ===\n");
	hyle_field_t fields[] = {
		{"name", HYLE_FIELD_STRING, 1, NULL, NULL, 1, 0, 0, 0, 0, NULL, 0},
		{"age",  HYLE_FIELD_INT,    1, NULL, NULL, 0, 18, 99, 0, 0, NULL, 0},
	};
	const char *values[] = {NULL, "15"};
	hyle_purify_error_t *errs;
	size_t nerr;
	int rc = hyle_purify_row(fields, 2, values, &errs, &nerr);
	CHECK(rc != 0, "multiple errors");
	CHECK(nerr == 2, "two errors");
	CHECK(strcmp(errs[0].rule, "required") == 0, "first error is required");
	CHECK(strcmp(errs[1].rule, "min") == 0, "second error is min");
	hyle_purify_errors_free(errs, nerr);
}

static void test_purify_null_params(void)
{
	printf("\n=== hyle_purify_row — null params ===\n");
	int rc = hyle_purify_row(NULL, 0, NULL, NULL, NULL);
	CHECK(rc == 0, "null fields → ok");
}

/* ================================================================
 * Phase 6 — full-text index (stoma) via hyle_source_query
 * ================================================================ */

static const hyle_field_t fts_fields[] = {
	{ "title",  HYLE_FIELD_STRING, 1, NULL, NULL, 0, 0, 0, 0, 256, NULL, 1 },
	{ "author", HYLE_FIELD_STRING, 1, NULL, NULL, 0, 0, 0, 0, 256, NULL, 1 },
	{ "year",   HYLE_FIELD_INT,    1, NULL, NULL, 0, 0, 0, 0, 16,  NULL, 0 },
};

static void fts_put_row(const char *id, const char *title,
	const char *author, const char *year)
{
	const char *names[3]  = { "title", "author", "year" };
	const char *values[3] = { title, author, year };
	hyle_source_put("fts.test", id, names, values, 3);
}

static void fts_query_src(const char *src, const char *field,
	const char *value, hyle_row_set_t *out)
{
	hyle_field_filter_t filters[1] = { { field, value } };
	hyle_query_t q;

	memset(&q, 0, sizeof(q));
	q.filters = filters;
	q.filter_count = 1;
	hyle_source_query(src, &q, out, NULL);
}

static void fts_query_filter(const char *field, const char *value,
	hyle_row_set_t *out)
{
	fts_query_src("fts.test", field, value, out);
}

static void test_fts(void)
{
	hyle_row_set_t out;

	printf("\n=== full-text index (stoma) ===\n");
	hyle_source_register("fts.test", fts_fields, 3, 0, 0, NULL);

	fts_put_row("song1", "Starlight of the night",
		"Alice Smith", "2020");
	fts_put_row("song2", "Station one", "Bob Jones", "2021");
	fts_put_row("song3", "A Dark Night", "Alice", "2020");
	fts_put_row("song4", "Nostalgia", "Carol", "2022");

	/* exact token */
	fts_query_filter("title", "night", &out);
	CHECK_IDS(out, "song1", "song3");
	qmap_close(out.row_hd);

	/* prefix: st → starlight + station, NOT nostalgia */
	fts_query_filter("title", "st", &out);
	CHECK_IDS(out, "song1", "song2");
	qmap_close(out.row_hd);

	/* two-token AND */
	fts_query_filter("title", "dark night", &out);
	CHECK_IDS(out, "song3");
	qmap_close(out.row_hd);

	/* case-insensitive author */
	fts_query_filter("author", "ALICE", &out);
	CHECK_IDS(out, "song1", "song3");
	qmap_close(out.row_hd);

	/* mixed searchable + non-searchable AND */
	{
		hyle_field_filter_t filters[2] = {
			{ "title", "night" },
			{ "year", "2020" },
		};
		hyle_query_t q;

		memset(&q, 0, sizeof(q));
		q.filters = filters;
		q.filter_count = 2;
		hyle_source_query("fts.test", &q, &out, NULL);
		CHECK_IDS(out, "song1", "song3");
		qmap_close(out.row_hd);
	}

	/* two searchable filters AND */
	{
		hyle_field_filter_t filters[2] = {
			{ "title", "night" },
			{ "author", "ALICE" },
		};
		hyle_query_t q;

		memset(&q, 0, sizeof(q));
		q.filters = filters;
		q.filter_count = 2;
		hyle_source_query("fts.test", &q, &out, NULL);
		CHECK_IDS(out, "song1", "song3");
		qmap_close(out.row_hd);
	}

	/* update → lazy rebuild: stale token gone, new token matches */
	fts_put_row("song1", "Brand New Title", "Alice Smith", "2020");
	fts_query_filter("title", "starlight", &out);
	CHECK(qmap_count(out.row_hd, NULL) == 0,
		"stale token gone after update");
	qmap_close(out.row_hd);
	fts_query_filter("title", "brand", &out);
	CHECK_IDS(out, "song1");
	qmap_close(out.row_hd);

	/* delete → gone */
	hyle_source_del("fts.test", "song2");
	fts_query_filter("title", "st", &out);
	CHECK(qmap_count(out.row_hd, NULL) == 0, "deleted row gone");
	qmap_close(out.row_hd);

	/* empty filter value → no-op, matches everything */
	fts_query_filter("title", "", &out);
	CHECK(qmap_count(out.row_hd, NULL) == 3, "empty filter matches all");
	qmap_close(out.row_hd);

	/* non-searchable field still filtered by the old path */
	fts_query_filter("year", "2021", &out);
	CHECK(qmap_count(out.row_hd, NULL) == 0, "year filter via old path");
	qmap_close(out.row_hd);
}

typedef struct {
	char title[256];
	char author[256];
} fts_rec_t;

static void test_fts_record(void)
{
	static const qmap_record_field_t rec_fields[2] = {
		{ "title", QM_STR, offsetof(fts_rec_t, title),
		  sizeof(((fts_rec_t *)0)->title), 0, 0, NULL },
		{ "author", QM_STR, offsetof(fts_rec_t, author),
		  sizeof(((fts_rec_t *)0)->author), 0, 0, NULL },
	};
	static const hyle_field_t fields[2] = {
		{ "title", HYLE_FIELD_STRING, 1, NULL, NULL, 0, 0, 0, 0, 256,
		  NULL, 1 },
		{ "author", HYLE_FIELD_STRING, 1, NULL, NULL, 0, 0, 0, 0, 256,
		  NULL, 1 },
	};
	uint32_t rec;
	unsigned fhd;
	hyle_row_set_t out;
	const char *names[2]  = { "title", "author" };
	const char *values[2] = { "Starlight over record", "Dora" };

	printf("\n=== record-aware hyle_source_put (FTS) ===\n");

	rec = qmap_record_register("fts.rec", sizeof(fts_rec_t),
		rec_fields, 2);
	CHECK(rec != QM_MISS, "record layout registered");

	fhd = hyle_source_register("fts.rec", fields, 2, rec, 0, NULL);
	CHECK(fhd != 0, "record source registered");

	hyle_source_put("fts.rec", "r1", names, values, 2);

	/* record branch wrote via qmap_field_put → struct field set */
	CHECK(strcmp(qmap_field_get(fhd, "r1", "title"),
		"Starlight over record") == 0,
		"record branch: title round-trips via qmap_field_get");

	/* FTS over the record source (first query rebuilds) */
	fts_query_src("fts.rec", "title", "star", &out);
	CHECK(qmap_count(out.row_hd, NULL) == 1, "record FTS query hits r1");
	qmap_close(out.row_hd);

	/* foreign-writer update via put → dirty → lazy rebuild */
	values[0] = "Moonlight now";
	hyle_source_put("fts.rec", "r1", names, values, 2);
	fts_query_src("fts.rec", "title", "starlight", &out);
	CHECK(qmap_count(out.row_hd, NULL) == 0, "stale token gone (record)");
	qmap_close(out.row_hd);
	fts_query_src("fts.rec", "title", "moonlight", &out);
	CHECK(qmap_count(out.row_hd, NULL) == 1, "new token matches (record)");
	qmap_close(out.row_hd);

	/* delete → gone from the record source */
	hyle_source_del("fts.rec", "r1");
	fts_query_src("fts.rec", "title", "moonlight", &out);
	CHECK(qmap_count(out.row_hd, NULL) == 0, "deleted record row gone");
	qmap_close(out.row_hd);
}

/* ================================================================
 * Phase 3: combined main
 * ================================================================ */

int main(void)
{
	setlocale(LC_ALL, "en_US.UTF-8");
	printf("libhyle Phase 1+2+3 tests\n");
	printf("-------------------------\n");

	/* Phase 1 */
	test_ctx();
	test_null();
	test_bool();
	test_int();
	test_float();
	test_string();
	test_array();
	test_map();

	/* Phase 2 — parse */
	test_parse_empty();
	test_parse_sort();
	test_parse_pagination();
	test_parse_q();
	test_parse_include();
	test_parse_field_filter();
	test_parse_multiple_filters();
	test_parse_all_params();
	test_parse_unknown_becomes_filter();

	/* Phase 2 — filter */
	test_filter_single_match();
	test_filter_no_match();
	test_filter_multi_and();
	test_filter_fulltext();
	test_filter_fulltext_no_match();
	test_filter_q_and_field();
	test_filter_accent_folding();

	/* Phase 2 — sort */
	test_sort_string_asc();
	test_sort_string_desc();
	test_sort_numeric_asc();
	test_sort_numeric_desc();
	test_sort_noop();

	/* Phase 2 — paginate */
	test_paginate_page1();
	test_paginate_page3();
	test_paginate_beyond();
	test_paginate_all();

	/* Phase 2 — apply view */
	test_apply_view_full();
	test_apply_view_noop();
	test_apply_view_filter_only();
	test_apply_view_sort_only();

	/* Phase 3 — field */
	test_field_lookup();
	test_field_is_reference();

	/* Phase 3 — manifest */
	test_manifest_valid();
	test_manifest_unknown_filter_field();
	test_manifest_unknown_sort_field();
	test_manifest_unknown_include_field();
	test_manifest_source_not_found();
	test_manifest_null_params();
	test_manifest_empty_query();
	test_manifest_reference_lookup();
	test_manifest_reference_inline();
	test_manifest_dedup_ref_targets();
	test_manifest_multi_reference();
	test_manifest_inverse_field();
	test_manifest_empty_blueprint();

	/* Phase 6 — purify */
	test_purify_null_params();
	test_purify_required();
	test_purify_min();
	test_purify_max();
	test_purify_min_length();
	test_purify_max_length();
	test_purify_pattern();
	test_purify_valid();
	test_purify_multiple_errors();

	/* Phase 6 — full-text index */
	test_fts();
	test_fts_record();

	printf("\n-----------------------\n");
	printf("Results: %d/%d passed", total - failures, total);
	if (failures > 0)
		printf(", %d FAILED", failures);
	printf("\n");

	return failures > 0 ? 1 : 0;
}
