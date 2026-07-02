import { defineConfig } from "vitepress";

// GitHub Project Pages: https://takenori-kusaka.github.io/QuickScribe/
export default defineConfig({
  lang: "ja",
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
  themeConfig: {
    nav: [
      { text: "ホーム", link: "/" },
      { text: "使い方", link: "/guide" },
      { text: "ダウンロード", link: "/download" },
      { text: "プライバシー", link: "/privacy" },
      { text: "GitHub", link: "https://github.com/Takenori-Kusaka/QuickScribe" },
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
    socialLinks: [
      { icon: "github", link: "https://github.com/Takenori-Kusaka/QuickScribe" },
    ],
    footer: {
      message: "MIT License. Windows は現在未署名（コード署名は将来対応予定）。",
      copyright: "© Takenori Kusaka",
    },
    outline: { label: "このページの内容" },
    docFooter: { prev: "前へ", next: "次へ" },
  },
});
