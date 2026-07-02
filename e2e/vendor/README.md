# e2e/vendor

テスト専用にベンダリングした第三者スクリプト。**アプリ配布物には含まれない**（e2e/a11y 検証でのみ使用）。

- `axe.min.js` — [axe-core](https://github.com/dequelabs/axe-core)（Deque Systems）。ライセンス: **MPL-2.0**。
  アクセシビリティ自動検証(#395)に使用。npm 依存として追加すると Windows での
  `npm install` が package-lock の libc メタデータを破損させるため、ミニファイ済みファイルを
  直接ベンダリングしている。更新は upstream の `axe.min.js` を差し替える。

MPL-2.0 の原文は https://www.mozilla.org/MPL/2.0/ を参照（ファイル冒頭のライセンスバナーも保持）。
