// ESLint flat config（フロント TS/Svelte の静的解析ゲート / #392・#393）。
// 方針: v1.0.0に向け「ゲートを先に張る」。既知の大型課題(a11y=#395, 神コンポーネント=#392)は
// 別issueで段階対応するため、本PRでは騒がしいルールを warn に留め errors=0 でCI通過させる。
import js from "@eslint/js";
import ts from "typescript-eslint";
import svelte from "eslint-plugin-svelte";
import prettier from "eslint-config-prettier";
import globals from "globals";

export default ts.config(
  {
    ignores: [
      "dist/",
      "coverage/",
      "node_modules/",
      "src-tauri/",
      "site/", // 独立した VitePress サブプロジェクト（自前のツールを持つ）。ビルド成果物含め対象外。
      "playwright-report/",
      "test-results/",
      "e2e/mocks/**",
      "e2e/vendor/**", // vendored な第三者ミニファイ済みJS(axe-core)はlint対象外。
    ],
  },
  js.configs.recommended,
  ...ts.configs.recommended,
  ...svelte.configs["flat/recommended"],
  prettier,
  ...svelte.configs["flat/prettier"],
  {
    languageOptions: {
      globals: { ...globals.browser, ...globals.node },
    },
  },
  {
    // .svelte に加え、Svelte5 runes モジュール(.svelte.ts/.svelte.js / #392)も TS パーサで解析する。
    files: ["**/*.svelte", "**/*.svelte.ts", "**/*.svelte.js"],
    languageOptions: {
      parserOptions: { parser: ts.parser },
    },
  },
  {
    // Node スクリプト（CJS）。
    files: ["scripts/**/*.cjs"],
    languageOptions: { globals: { ...globals.node } },
    rules: { "@typescript-eslint/no-require-imports": "off", "no-undef": "off" },
  },
  {
    // WebdriverIO e2e（mocha+wdioグローバル / CJS設定）。
    files: ["e2e/**/*.{js,cjs}"],
    languageOptions: {
      globals: {
        ...globals.mocha,
        ...globals.node,
        browser: "readonly",
        $: "readonly",
        $$: "readonly",
      },
    },
    rules: {
      "@typescript-eslint/no-require-imports": "off",
      "no-undef": "off",
    },
  },
  {
    // 段階導入: 既知課題は warn（CIは error のみで落とす）。後続issueで error 化していく。
    rules: {
      "@typescript-eslint/no-unused-vars": [
        "warn",
        { argsIgnorePattern: "^_", varsIgnorePattern: "^_" },
      ],
      "@typescript-eslint/no-explicit-any": "warn",
      // XSSシンクの混入だけは error で固定（防御）。
      "svelte/no-at-html-tags": "error",
      // 以下は既存コードに多数あり別issueで段階対応するため当面 warn:
      // require-each-key/no-useless-mustaches=#392, prefer-svelte-reactivity=#392(Map/Set反応性),
      // a11y系=#395(モーダルdialog/ラベル等)。
      "svelte/require-each-key": "warn",
      "svelte/no-useless-mustaches": "warn",
      "svelte/prefer-svelte-reactivity": "warn",
    },
  },
);
