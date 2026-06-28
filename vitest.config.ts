import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: ["src/**/*.test.ts"],
    environment: "node",
    coverage: {
      // v1.0.0 に向けたカバレッジ計測基盤（#402 Phase1: まず可視化、ゲートは段階導入）。
      provider: "v8",
      reporter: ["text", "html", "lcov"],
      reportsDirectory: "coverage",
      // 計測対象は現状フロントの純ロジック(src/lib の .ts)。
      // App.svelte 等の .svelte はコンポーネントテスト基盤が未整備のため対象外
      // (#402 Phase2 で App.svelte をlib抽出→計測対象化する)。
      include: ["src/**/*.ts"],
      exclude: ["src/**/*.test.ts", "src/main.ts", "src/vite-env.d.ts", "e2e/**"],
      all: true,
    },
  },
});
