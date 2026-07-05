# 記事公開・再デプロイポリシー

> Status: Living（2026-07-05 初版 / 2026-07-05 Qiita 撤去・Zenn 一本化）。技術記事の単一ソース管理・公開・再デプロイの運用規約。
> 関連: [ADR-0008 ライセンス・配布](../adr/0008-licensing-and-distribution.md) / [design.md](../design.md)。

## 目的

技術記事を「腐らない（エバーグリーン）」「本文と図・数値が乖離しない」状態で継続提供する。
人間の判断が必要な点（実公開）だけ人間ゲートに残し、それ以外はツールと CI で担保する。

## 0. プラットフォームは Zenn 一本（Qiita は採らない）

- 発信は **Zenn に一本化**する。記事（Article）と、複数章を束ねる **Book（本）** の両方を Zenn で管理する。
- **Qiita は採用しない**。理由: Qiita には記事の親子関係・章立て（Zenn Book 相当のグルーピング）が無く、
  体系立てて束ねる方針と噛み合わないため。以前あった Qiita 変換パイプライン（`public/` 生成・
  `scripts/zenn-to-qiita.mjs`・`qiita-publish.yml`）は 2026-07-05 に撤去した。

## 1. 単一ソース

- **正典は `articles/*.md`（Zenn 記事形式）**。記事はここにだけ手で書く。
- Book を作る場合は `books/<slug>/`（`config.yaml` ＋ 章 Markdown）で管理する。順序は `config.yaml` の
  `chapters` 配列で明示する（`chapters` に未列挙の章は Zenn 上で非公開になる点に注意）。

## 2. 画像・図は必ず絶対URL

- 記事内の画像参照は **絶対URL**（`https://raw.githubusercontent.com/Takenori-Kusaka/QuickScribe/main/...`）にする。
  Zenn から参照するリポジトリ内の画像は相対パスでは解決できないため。
- **図は Mermaid を単一ソースにし、CI で SVG/PNG へ書き出す**（`npm run diagrams` / `scripts/render-diagrams.mjs`）。
  対象は `docs/design.md` の ```mermaid``` ブロック。出力は `docs/assets/diagrams/design-<n>.svg|png`。
  記事は PNG を絶対URLで参照する。SVG も差分可読なソースとして併置する。
- 図の鮮度は CI が担保する: PR では `diagrams:check`（期待する図がコミット済みか）を検証し、`main` への
  push では図を再生成して差分があれば自動コミットする（[articles.yml](../../.github/workflows/articles.yml)）。

## 3. 校正（textlint 必須）

- `articles/**/*.md` は **textlint（preset-ja-technical-writing）** をゼロエラーで通すこと（`npm run lint:text`）。
  CI ジョブ（articles.yml）が必須ゲート。設定は [`.textlintrc.json`](../../.textlintrc.json)。
- 文体は **ですます調**に統一（設定で強制）。1文120字以内・読点4つ以内を目安に短く書く。

## 4. 公開と再デプロイの頻度

- **Zenn の GitHub 連携**で `articles/` と `books/` が **自動同期**される。push 型の公開 CI は不要（連携が担う）。
- **再デプロイは「メジャーバージョンアップ時のみ」**（タグ `vX.0.0`）。記事は製品の設計思想を語る性質上、
  マイナー/パッチのたびに再公開しない。**細かい使い方の改訂は GitHub Pages（ドキュメントサイト）側で対応**し、
  記事は大きな節目でのみ更新・再デプロイする。

## 5. 実公開は人間ゲート

- 記事・本は既定で `published: false`（Zenn 下書き）。
- **実公開はメンテナが front-matter を切り替えて判断する**。CI は自動で `published: true` にしない。

## 6. 将来検討（記録）

- 図の重厚化（C4 / Structurizr DSL）は将来検討。現状は Mermaid→SVG/PNG の軽量経路で
  「CI 生成・所定配置・絶対パス」の要件を満たす（[design.md](../design.md) の図がソース）。
