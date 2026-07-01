import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { svelteTesting } from "@testing-library/svelte/vite";

export default defineConfig({
  // .svelte のコンパイル＋@testing-library/svelte のテスト用解決(browser条件・自動cleanup)。
  // App.svelte 等のコンポーネントテストを可能にする(#402 Phase2)。
  plugins: [svelte({ hot: false }), svelteTesting()],
  test: {
    include: ["src/**/*.test.ts"],
    // 既定は node（src/lib の純ロジック）。コンポーネントテスト(*.svelte.test.ts)のみ jsdom。
    environment: "node",
    environmentMatchGlobs: [["src/**/*.svelte.test.ts", "jsdom"]],
    setupFiles: ["src/test/setup.ts"],
    coverage: {
      // v1.0.0 に向けたカバレッジ計測基盤（#402 Phase1: まず可視化、ゲートは段階導入）。
      provider: "v8",
      reporter: ["text", "html", "lcov"],
      reportsDirectory: "coverage",
      // 計測対象は src/lib の .ts と App.svelte 等の .svelte の両方(実効カバレッジを正直に測る)。
      include: ["src/**/*.ts", "src/**/*.svelte"],
      exclude: [
        "src/**/*.test.ts",
        "src/main.ts",
        "src/vite-env.d.ts",
        "src/lib/i18n/**", // i18n初期化(副作用設定)はロジックでないため計測対象外。
        "src/test/**", // テストセットアップは計測対象外。
        "e2e/**",
      ],
      all: true,
      // ゲート化（#402/#481-item13）。**計測を正直化**: App.svelte も対象に含める。
      // 旧ゲート(75/85)は src/lib(全体の約14%)のみを測っており、実効カバレッジ(50%)を
      // 過大(87%)に見せていた。App.svelte を含めた実測を回帰防止の下限とする。
      // 履歴(lines): 50%(正直化) → 64 → 66 → 68 → 71 → 71.6 → 75.4 → **80.1%達成**。
      // lines/statements/functions が目標の80%前後に到達(実測 local ~82%)。
      // CI環境差(約0.4pt低い)を見込み下限は少し下に置く。branchesは分岐が多く73を下限に、
      // 引き続きエラー経路のテストで80%化を継続。
      thresholds: {
        lines: 80,
        statements: 80,
        functions: 79,
        branches: 73,
      },
    },
  },
});
