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
      exclude: [
        "src/**/*.test.ts",
        "src/main.ts",
        "src/vite-env.d.ts",
        "src/lib/i18n/**", // i18n初期化(副作用設定)はロジックでないため計測対象外。
        "e2e/**",
      ],
      all: true,
      // ゲート化（#402）。現状 lib カバレッジは ~81%。回帰を防ぐ下限を設定し、
      // App.svelte の lib 抽出を進めながら段階的に引き上げる。
      thresholds: {
        lines: 75,
        statements: 75,
        functions: 85,
        branches: 85,
      },
    },
  },
});
