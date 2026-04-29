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
    await page.locator("button.primaryButton[type=submit]").click();
    await expect(page).toHaveURL("/");
    await expect(page.locator("tbody")).toContainText("Alicia");
  });

  test("add link navigates to new user page", async ({ page }) => {
    await page.goto("/");
    await page.locator("a.primaryButton").click();
    await expect(page).toHaveURL("/users/new");
    await expect(page.locator("h2").first()).toContainText("Add user");
  });

  test("create user success", async ({ page }) => {
    await page.goto("/users/new");
    await page.waitForLoadState("networkidle");
    await page.locator('input[name="name"]').fill("Zelda");
    await page.locator('input[name="email"]').fill("zelda@example.test");
    await page.locator("button.primaryButton[type=submit]").click();
    await expect(page).toHaveURL("/");
    // Zelda sorts last — navigate to page 2 to find her
    await page.locator('button[name="page"]:has-text("Next")').click();
    await expect(page.locator("tbody")).toContainText("Zelda");
  });

  test("create user — client-side validation error (name too short)", async ({ page }) => {
    await page.goto("/users/new");
    await page.locator('input[name="name"]').fill("Z");
    await page.locator("button.primaryButton[type=submit]").click();
    await expect(page).toHaveURL("/users/new");
    await expect(page.locator(".errors")).toBeVisible();
  });

  test("delete user", async ({ page }) => {
    await page.goto("/users/1/edit");
    await page.locator("button.dangerButton[type=submit]").click();
    await expect(page).toHaveURL("/");
    await expect(page.locator("tbody")).not.toContainText("Alice");
  });

  test("filter by name", async ({ page }) => {
    await page.goto("/");
    await page.waitForLoadState("networkidle");
    await page.locator('.columnFilter-wrap input[name="name"]').fill("Ali");
    await page.locator('form[method="get"] button[type="submit"]:has-text("Apply")').first().click();
    const rows = page.locator("tbody tr");
    await expect(rows).toHaveCount(1);
    await expect(rows.first()).toContainText("Alice");
  });

  test("filter by name does not navigate (JS)", async ({ page }) => {
    await page.goto("/");
    await page.waitForLoadState("networkidle");
    await page.locator('.columnFilter-wrap input[name="name"]').fill("Ali");
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

  test("pagination — next page", async ({ page }) => {
    await page.goto("/");
    await page.locator('button[name="page"]:has-text("Next")').click();
    // 7 users, per_page 5 → page 2 has 2 rows
    const rows = page.locator("tbody tr");
    await expect(rows).toHaveCount(2);
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
    await page.locator('.columnFilter-wrap input[name="name"]').fill("Ali");
    await page.locator('form[method="get"] button[type="submit"]:has-text("Apply")').first().click();
    await expect(page).toHaveURL(/name=Ali/);
    const rows = page.locator("tbody tr");
    await expect(rows).toHaveCount(1);
    await expect(rows.first()).toContainText("Alice");
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
    await page.locator("a.primaryButton").click();
    await expect(page).toHaveURL("/users/new");
    await expect(page.locator('form[action="/users/new"]')).toBeVisible();
  });

  test("[no-js] create user — validation error pre-rendered by server", async ({ page }) => {
    await page.goto("/users/new");
    // Leave name empty, submit via native form POST
    await page.locator('form[action="/users/new"] button.primaryButton[type=submit]').click();
    // Server re-renders the page with errors; no redirect
    await expect(page).toHaveURL("/users/new");
    await expect(page.locator(".errors")).toBeVisible();
  });

  test("[no-js] create user — success redirects to list", async ({ page }) => {
    await page.goto("/users/new");
    await page.locator('input[name="name"]').fill("Zelda");
    await page.locator('input[name="email"]').fill("zelda@example.test");
    await page.locator('form[action="/users/new"] button.primaryButton[type=submit]').click();
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
    await page.locator('form[action="/users/1/edit"] button.primaryButton[type=submit]').click();
    await expect(page).toHaveURL("/");
    await expect(page.locator("tbody")).toContainText("Alicia");
  });

  test("[no-js] delete user", async ({ page }) => {
    await page.goto("/users/1/edit");
    await page.locator('form[action="/users/1/delete"] button.dangerButton[type=submit]').click();
    await expect(page).toHaveURL("/");
    await expect(page.locator("tbody")).not.toContainText("Alice");
  });
});
