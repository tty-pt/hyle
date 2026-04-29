import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./tests",
  fullyParallel: false, // tests share one server; run serially to avoid state races
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: 1,
  reporter: "list",

  use: {
    baseURL: "http://localhost:8080",
    headless: true,
    trace: "on-first-retry",
  },

  projects: [
    {
      name: "js",
      use: { ...devices["Desktop Chrome"] },
    },
    {
      name: "no-js",
      use: {
        ...devices["Desktop Chrome"],
        javaScriptEnabled: false,
      },
    },
  ],

});
