import { test, expect } from "@playwright/test";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const API = "http://localhost:3001";
const SSR = "http://localhost:4173";

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

test.describe("js", () => {
  test.use({ javaScriptEnabled: true });

  test.beforeEach(async ({ request }) => {
    const res = await request.get(`${API}/__reset`);
    expect(res.status()).toBe(200);
  });

  test("loads user list", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("h2").first()).toContainText("Users");
    const rows = page.locator("tbody tr");
    await expect(rows).toHaveCount(5); // default per_page = 5
  });

  test("row click navigates to edit page", async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector("tbody tr.hyle-row-clickable");
    await page.locator("tbody tr.hyle-row-clickable").first().click();
    await expect(page).toHaveURL(/\/users\/\d+\/edit/);
    await expect(page.locator("h2").first()).toContainText("Edit user");
  });

  test("edit page pre-fills form fields", async ({ page }) => {
    await page.goto("/users/1/edit");
    await expect(page.locator('input[aria-label="Name"]')).toHaveValue("Alice");
  });

  test("save edit and return to list", async ({ page }) => {
    await page.goto("/users/1/edit");
    await page.locator('input[aria-label="Name"]').fill("Alicia");
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
    await page.locator('input[aria-label="Name"]').fill("Zelda");
    await page.locator('input[aria-label="Email"]').fill("zelda@example.test");
    await page.locator("button.hyle-primary-button[type=submit]").click();
    await expect(page).toHaveURL("/");
    // Zelda sorts last — navigate to page 2 to find her
    await page.locator('button:has-text("Next →")').click();
    await expect(page.locator("tbody")).toContainText("Zelda");
  });

  test("create user — client-side validation error (name too short)", async ({ page }) => {
    await page.goto("/users/new");
    await page.locator('input[aria-label="Name"]').fill("Z");
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
    await page.waitForSelector("tbody tr.hyle-row-clickable");
    await page.locator('.hyle-filter-bar input[aria-label="Name"]').fill("Ali");
    await page.locator('.hyle-filter-actions button:has-text("Apply")').click();
    const rows = page.locator("tbody tr");
    await expect(rows).toHaveCount(1);
    await expect(rows.first()).toContainText("Alice");
  });

  test("filter by name does not navigate (JS)", async ({ page }) => {
    await page.goto("/");
    await page.waitForLoadState("networkidle");
    await page.locator('.hyle-filter-bar input[aria-label="Name"]').fill("Ali");
    await page.locator('.hyle-filter-actions button:has-text("Apply")').click();
    await expect(page.locator("tbody tr")).toHaveCount(1);
    // URL must not have changed — filter applied reactively, no page reload
    expect(page.url()).toBe("http://localhost:4173/");
  });

  test("pagination next does not navigate (JS)", async ({ page }) => {
    await page.goto("/");
    await page.waitForLoadState("networkidle");
    await page.locator('button:has-text("Next →")').click();
    // 7 users, per_page 5 → page 2 has 2 rows
    await expect(page.locator("tbody tr")).toHaveCount(2);
    // URL must not have changed — pagination applied reactively, no page reload
    expect(page.url()).toBe("http://localhost:4173/");
  });

  test("pagination — next page", async ({ page }) => {
    await page.goto("/");
    await page.locator('button:has-text("Next →")').click();
    // 7 users, per_page 5 → page 2 has 2 rows
    const rows = page.locator("tbody tr");
    await expect(rows).toHaveCount(2);
  });

  test("filter then paginate — filter persists across pages", async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector("tbody tr.hyle-row-clickable");
    // First paginate to page 2 (no filter, 7 users / per_page 5)
    await page.locator('button:has-text("Next →")').click();
    await expect(page.locator("tbody tr")).toHaveCount(2);
    // Now go back to page 1 via Prev and apply a filter
    await page.locator('button:has-text("← Prev")').click();
    await expect(page.locator("tbody tr")).toHaveCount(5);
    await page.locator('.hyle-filter-bar input[aria-label="Name"]').fill("a");
    await page.locator('.hyle-filter-actions button:has-text("Apply")').click();
    await expect(page.locator("tbody tr")).toHaveCount(4);
    // Filter input must still hold the applied value
    await expect(page.locator('.hyle-filter-bar input[aria-label="Name"]')).toHaveValue("a");
  });
});

// ---------------------------------------------------------------------------
// No-JS tests (SSR server at localhost:4173)
// ---------------------------------------------------------------------------

test.describe("no-js", () => {
  test.use({ javaScriptEnabled: false });

  test.beforeEach(async ({ request }) => {
    const res = await request.get(`${SSR}/__reset`);
    expect(res.status()).toBe(200);
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
});
