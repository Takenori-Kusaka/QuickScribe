import { defineConfig, devices } from "@playwright/test";

// #666 の描画コスト計測専用の Playwright 設定。時間依存の数値を含むため CI からは外し、
// 手動計測のときだけこの設定で実行する:
//   npx playwright test --config e2e/playwright.perf.config.ts
export default defineConfig({
  testDir: ".",
  testMatch: /perf-entry-list\.spec\.ts/,
  fullyParallel: false,
  workers: 1,
  reporter: "list",
  use: {
    baseURL: "http://localhost:1421",
    viewport: { width: 1200, height: 860 },
    ...devices["Desktop Chrome"],
  },
  webServer: {
    command: "npx vite --config e2e/vite.screenshot.config.ts --port 1421 --strictPort",
    // 本設定は e2e/ に置くため、webServer の cwd はリポジトリルートへ明示的に戻す。
    cwd: "..",
    url: "http://localhost:1421",
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },
});
