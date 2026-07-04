import { defineConfig } from "vitepress";

// GitHub Project Pages: https://takenori-kusaka.github.io/QuickScribe/
export default defineConfig({
  title: "QuickScribe",
  description:
    "話すだけで、思考が整う。ローカル完結・プライバシー重視のボイスジャーナル。",
  base: "/QuickScribe/",
  lastUpdated: true,
  cleanUrls: true,
  head: [
    ["meta", { name: "theme-color", content: "#4f46e5" }],
    ["meta", { property: "og:title", content: "QuickScribe" }],
    [
      "meta",
      {
        property: "og:description",
        content:
          "話すだけで、思考が整う。ローカル完結・プライバシー重視のボイスジャーナル。",
      },
    ],
  ],
  // i18n: 既定は日本語(root)。/en/ に英語ロケールを追加し、nav の言語メニューで切替(#401)。
  locales: {
    root: {
      label: "日本語",
      lang: "ja",
      themeConfig: {
        nav: [
          { text: "ホーム", link: "/" },
          { text: "使い方", link: "/guide" },
          { text: "ダウンロード", link: "/download" },
          { text: "プライバシー", link: "/privacy" },
          {
            text: "GitHub",
            link: "https://github.com/Takenori-Kusaka/QuickScribe",
          },
        ],
        sidebar: [
          {
            text: "QuickScribe",
            items: [
              { text: "ホーム", link: "/" },
              { text: "使い方", link: "/guide" },
              { text: "ダウンロード", link: "/download" },
              { text: "プライバシーポリシー", link: "/privacy" },
            ],
          },
        ],
        footer: {
          message: "MIT License. Windows は現在未署名（コード署名は将来対応予定）。",
          copyright: "© Takenori Kusaka",
        },
        outline: { label: "このページの内容" },
        docFooter: { prev: "前へ", next: "次へ" },
      },
    },
    en: {
      label: "English",
      lang: "en",
      link: "/en/",
      description:
        "Speak, and your thinking gets organized. A local-first, privacy-focused voice journal.",
      themeConfig: {
        nav: [
          { text: "Home", link: "/en/" },
          { text: "Guide", link: "/en/guide" },
          { text: "Download", link: "/en/download" },
          { text: "Privacy", link: "/privacy" },
          {
            text: "GitHub",
            link: "https://github.com/Takenori-Kusaka/QuickScribe",
          },
        ],
        sidebar: [
          {
            text: "QuickScribe",
            items: [
              { text: "Home", link: "/en/" },
              { text: "Guide", link: "/en/guide" },
              { text: "Download", link: "/en/download" },
              { text: "Privacy Policy", link: "/privacy" },
            ],
          },
        ],
        footer: {
          message:
            "MIT License. Windows builds are currently unsigned (code signing planned).",
          copyright: "© Takenori Kusaka",
        },
      },
    },
  },
  themeConfig: {
    socialLinks: [
      { icon: "github", link: "https://github.com/Takenori-Kusaka/QuickScribe" },
    ],
  },
});
