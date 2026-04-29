import { test, expect } from "@playwright/test";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const API = "http://localhost:3001";

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
    await page.waitForSelector("tbody tr.hyle-tr--clickable");
    await page.locator("tbody tr.hyle-tr--clickable").first().click();
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
    await page.locator('input[aria-label="Name"]').fill("Zelda");
    await page.locator('input[aria-label="Email"]').fill("zelda@example.test");
    await page.locator("button.primaryButton[type=submit]").click();
    await expect(page).toHaveURL("/");
    // Zelda sorts last — navigate to page 2 to find her
    await page.locator('button:has-text("Next →")').click();
    await expect(page.locator("tbody")).toContainText("Zelda");
  });

  test("create user — client-side validation error (name too short)", async ({ page }) => {
    await page.goto("/users/new");
    await page.locator('input[aria-label="Name"]').fill("Z");
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
    await page.waitForSelector("tbody tr.hyle-tr--clickable");
    await page.locator('.hyle-col-filter input[aria-label="Name"]').fill("Ali");
    await page.locator('.hyle-table-filters button:has-text("Apply")').click();
    const rows = page.locator("tbody tr");
    await expect(rows).toHaveCount(1);
    await expect(rows.first()).toContainText("Alice");
  });

  test("filter by name does not navigate (JS)", async ({ page }) => {
    await page.goto("/");
    await page.waitForLoadState("networkidle");
    await page.locator('.hyle-col-filter input[aria-label="Name"]').fill("Ali");
    await page.locator('.hyle-table-filters button:has-text("Apply")').click();
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
});
