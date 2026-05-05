import { test, expect, request as apiRequest } from "@playwright/test";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const BASE = "http://localhost:8080";

/** Reset server state to seed data before each test. */
async function reset(req: ReturnType<typeof apiRequest.newContext> extends Promise<infer T> ? T : never) {
  const res = await req.get("/__reset");
  expect(res.status()).toBe(200);
}

// We use the `request` fixture from `test` directly via beforeEach.

// ---------------------------------------------------------------------------
// JS tests
// ---------------------------------------------------------------------------

test.describe("js", () => {
  test.use({ javaScriptEnabled: true });

  test.beforeEach(async ({ request }) => {
    await request.get("/__reset");
  });

  test("loads user list", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("h2").first()).toContainText("Users");
    const rows = page.locator("tbody tr");
    await expect(rows).toHaveCount(5); // default per_page = 5
  });

  test("row link navigates to edit page", async ({ page }) => {
    await page.goto("/");
    // First visible row's name cell link — seed sorted by name, so Alice is first
    await page.locator("tbody tr:first-child td:first-child a").click();
    await expect(page).toHaveURL(/\/users\/\d+\/edit/);
    await expect(page.locator("h2").first()).toContainText("Edit user");
  });

  test("edit page pre-fills form fields", async ({ page }) => {
    await page.goto("/users/1/edit");
    await expect(page.locator('input[name="name"]')).toHaveValue("Alice");
  });

  test("save edit and return to list", async ({ page }) => {
    await page.goto("/users/1/edit");
    await page.waitForLoadState("networkidle");
    await page.locator('input[name="name"]').fill("Alicia");
    await page.locator("button.hyle-primary-button[type=submit]").click();
    await expect(page).toHaveURL("/");
    await expect(page.locator("tbody")).toContainText("Alicia");
  });

  test("add link navigates to new user page", async ({ page }) => {
    await page.goto("/");
    await page.locator("details.hyle-action-menu").click();
    await page.locator("a.hyle-primary-button").click();
    await expect(page).toHaveURL("/users/new");
    await expect(page.locator("h2").first()).toContainText("Add user");
  });

  test("create user success", async ({ page }) => {
    await page.goto("/users/new");
    await page.waitForLoadState("networkidle");
    await page.locator('input[name="name"]').fill("Zelda");
    await page.locator('input[name="email"]').fill("zelda@example.test");
    await page.locator("button.hyle-primary-button[type=submit]").click();
    await expect(page).toHaveURL("/");
    // Zelda sorts last — navigate to page 2 to find her
    await page.locator('button[name="page"]:has-text("Next")').click();
    await expect(page.locator("tbody")).toContainText("Zelda");
  });

  test("create user — client-side validation error (name too short)", async ({ page }) => {
    await page.goto("/users/new");
    await page.locator('input[name="name"]').fill("Z");
    await page.locator("button.hyle-primary-button[type=submit]").click();
    await expect(page).toHaveURL("/users/new");
    await expect(page.locator(".hyle-errors")).toBeVisible();
  });

  test("delete user", async ({ page }) => {
    await page.goto("/users/1/edit");
    await page.locator("button.hyle-danger-button[type=submit]").click();
    await expect(page).toHaveURL("/");
    await expect(page.locator("tbody")).not.toContainText("Alice");
  });

  test("filter by name", async ({ page }) => {
    await page.goto("/");
    await page.waitForLoadState("networkidle");
    await page.locator('.hyle-filter-bar input[name="name"]').fill("Ali");
    await page.locator('form[method="get"] button[type="submit"]:has-text("Apply")').first().click();
    const rows = page.locator("tbody tr");
    await expect(rows).toHaveCount(1);
    await expect(rows.first()).toContainText("Alice");
  });

  test("filter by name does not navigate (JS)", async ({ page }) => {
    await page.goto("/");
    await page.waitForLoadState("networkidle");
    await page.locator('.hyle-filter-bar input[name="name"]').fill("Ali");
    await page.locator('form[method="get"] button[type="submit"]:has-text("Apply")').first().click();
    await expect(page.locator("tbody tr")).toHaveCount(1);
    // URL must not have changed — filter applied reactively, no page reload
    expect(page.url()).toBe("http://localhost:8080/");
  });

  test("pagination next does not navigate (JS)", async ({ page }) => {
    await page.goto("/");
    await page.waitForLoadState("networkidle");
    await page.locator('button[name="page"]:has-text("Next")').click();
    // 7 users, per_page 5 → page 2 has 2 rows
    await expect(page.locator("tbody tr")).toHaveCount(2);
    // URL must not have changed — pagination applied reactively, no page reload
    expect(page.url()).toBe("http://localhost:8080/");
  });

  test("filter by tag (Array<Reference> select)", async ({ page }) => {
    await page.goto("/");
    await page.waitForLoadState("networkidle");
    // Select "Rust" from the tags filter select
    await page.locator('.hyle-filter-bar input[type="checkbox"][name="tags"][value="rust"]').check();
    await page.locator('form[method="get"] button[type="submit"]:has-text("Apply")').first().click();
    const rows = page.locator("tbody tr");
    // Alice, Dmitri, Evelyn, Gustavo have the rust tag (4 users, per_page 5)
    await expect(rows).toHaveCount(4);
  });

  test("pagination — next page", async ({ page }) => {
    await page.goto("/");
    await page.locator('button[name="page"]:has-text("Next")').click();
    // 7 users, per_page 5 → page 2 has 2 rows
    const rows = page.locator("tbody tr");
    await expect(rows).toHaveCount(2);
  });

  test("filter then paginate — filter persists across pages", async ({ page }) => {
    await page.goto("/");
    await page.waitForLoadState("networkidle");
    // Filter by email containing "e" — all 7 users match (@example.test contains 'e')
    // With per_page=5: page 1 has 5 rows, page 2 has 2 rows → Next is enabled
    await page.locator('.hyle-filter-bar input[name="email"]').fill("e");
    await page.locator('form[method="get"] button[type="submit"]:has-text("Apply")').first().click();
    await expect(page.locator("tbody tr")).toHaveCount(5);
    // Go to page 2 — filter must remain active
    await page.locator('button[name="page"]:has-text("Next")').click();
    await expect(page.locator("tbody tr")).toHaveCount(2);
    // Filter input must still hold the applied value — this is the regression check
    await expect(page.locator('.hyle-filter-bar input[name="email"]')).toHaveValue("e");
  });
});

// ---------------------------------------------------------------------------
// No-JS tests
// ---------------------------------------------------------------------------

test.describe("no-js", () => {
  test.use({ javaScriptEnabled: false });

  test.beforeEach(async ({ request }) => {
    await request.get("/__reset");
  });

  test("[no-js] user list SSR renders", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("h2").first()).toContainText("Users");
    const rows = page.locator("tbody tr");
    await expect(rows).toHaveCount(5);
  });

  test("[no-js] filter by name via form submit", async ({ page }) => {
    await page.goto("/");
    await page.locator('.hyle-filter-bar input[name="name"]').fill("Ali");
    await page.locator('form[method="get"] button[type="submit"]:has-text("Apply")').first().click();
    await expect(page).toHaveURL(/name=Ali/);
    const rows = page.locator("tbody tr");
    await expect(rows).toHaveCount(1);
    await expect(rows.first()).toContainText("Alice");
  });

  test("[no-js] filter by role (Reference <select>) uses value not label", async ({ page }) => {
    await page.goto("/");
    // Select "admin" role — the <select> must submit the id ("admin"), not the label ("Admin")
    await page.locator('.hyle-filter-bar select[name="role"]').selectOption("admin");
    await page.locator('form[method="get"] button[type="submit"]:has-text("Apply")').first().click();
    // URL must contain role=admin (the id), not role=Admin (the label)
    await expect(page).toHaveURL(/role=admin/);
    const rows = page.locator("tbody tr");
    // Alice and Fatima have role "admin" (2 users, per_page 5)
    await expect(rows).toHaveCount(2);
  });

  test("[no-js] filter by tag (Array<Reference> checkbox) uses value not label", async ({ page }) => {
    await page.goto("/");
    // Check the "rust" tag checkbox — value must be the id ("rust"), not the label ("Rust")
    await page.locator('.hyle-filter-bar input[type="checkbox"][name="tags"][value="rust"]').check();
    await page.locator('form[method="get"] button[type="submit"]:has-text("Apply")').first().click();
    // URL must contain tags=rust (the id), not tags=Rust (the label)
    await expect(page).toHaveURL(/tags=rust/);
    const rows = page.locator("tbody tr");
    // Alice, Dmitri, Evelyn, Gustavo have the rust tag (4 users, per_page 5)
    await expect(rows).toHaveCount(4);
  });

  test("[no-js] pagination next", async ({ page }) => {
    await page.goto("/");
    await page.locator('button[name="page"]:has-text("Next")').click();
    await expect(page).toHaveURL(/page=2/);
    const rows = page.locator("tbody tr");
    await expect(rows).toHaveCount(2);
  });

  test("[no-js] add link navigates to new user page", async ({ page }) => {
    await page.goto("/");
    await page.locator("details.hyle-action-menu").click();
    await page.locator("a.hyle-primary-button").click();
    await expect(page).toHaveURL("/users/new");
    await expect(page.locator('form[action="/users/new"]')).toBeVisible();
  });

  test("[no-js] create user — validation error pre-rendered by server", async ({ page }) => {
    await page.goto("/users/new");
    // Leave name empty, submit via native form POST
    await page.locator('form[action="/users/new"] button.hyle-primary-button[type=submit]').click();
    // Server re-renders the page with errors; no redirect
    await expect(page).toHaveURL("/users/new");
    await expect(page.locator(".hyle-errors")).toBeVisible();
  });

  test("[no-js] create user — success redirects to list", async ({ page }) => {
    await page.goto("/users/new");
    await page.locator('input[name="name"]').fill("Zelda");
    await page.locator('input[name="email"]').fill("zelda@example.test");
    await page.locator('form[action="/users/new"] button.hyle-primary-button[type=submit]').click();
    await expect(page).toHaveURL("/");
    // Zelda sorts last — navigate to page 2 to find her
    await page.locator('button[name="page"]:has-text("Next")').click();
    await expect(page.locator("tbody")).toContainText("Zelda");
  });

  test("[no-js] edit page pre-fills form fields", async ({ page }) => {
    await page.goto("/users/1/edit");
    await expect(page.locator('input[name="name"]')).toHaveValue("Alice");
  });

  test("[no-js] edit user success", async ({ page }) => {
    await page.goto("/users/1/edit");
    await page.locator('input[name="name"]').fill("Alicia");
    await page.locator('form[action="/users/1/edit"] button.hyle-primary-button[type=submit]').click();
    await expect(page).toHaveURL("/");
    await expect(page.locator("tbody")).toContainText("Alicia");
  });

  test("[no-js] delete user", async ({ page }) => {
    await page.goto("/users/1/edit");
    await page.locator('form[action="/users/1/delete"] button.hyle-danger-button[type=submit]').click();
    await expect(page).toHaveURL("/");
    await expect(page.locator("tbody")).not.toContainText("Alice");
  });
});
