import { defineConfig, devices } from "@playwright/test";

// スクショ自動生成用の Playwright 設定。Vite(モック差し替え)を起動し、
// ヘッドレス Chromium でフロントを描画して docs/assets に出力する。
export default defineConfig({
  testDir: "./e2e",
  testMatch: /(screenshots|a11y)\.spec\.ts/,
  fullyParallel: false,
  reporter: "list",
  use: {
    baseURL: "http://localhost:1421",
    viewport: { width: 1200, height: 860 },
    deviceScaleFactor: 2,
    ...devices["Desktop Chrome"],
  },
  webServer: {
    command: "npx vite --config e2e/vite.screenshot.config.ts --port 1421 --strictPort",
    url: "http://localhost:1421",
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },
});
