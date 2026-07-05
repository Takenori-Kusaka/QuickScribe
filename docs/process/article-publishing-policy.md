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

## 6. C4 アーキ図は Structurizr DSL（実装済み）

C4 モデルの構成図（System Context / Container / Component 等）は **Structurizr DSL を単一ソース**にし、
Mermaid と同じ思想（CI 生成・所定配置・絶対URL参照・自動コミット）で画像化する。

- **経路**: Structurizr CLI `export -format mermaid` で DSL を Mermaid 定義に変換し、既存の
  `@mermaid-js/mermaid-cli`（mmdc）で PNG/SVG にラスタライズする。レンダラを Mermaid に一本化でき、
  保守が軽い（PlantUML/graphviz を別途持ち込まない）。スクリプトは
  [`scripts/render-structurizr.mjs`](../../scripts/render-structurizr.mjs)。
- **DSL ソース**: `docs/architecture/*.dsl`（1 workspace = 1 ファイル。1 ファイルに複数ビュー可）。
- **出力**: `docs/assets/diagrams/<dslのbasename>-<viewキー>.png|svg`。
  例: `docs/architecture/engine-abstraction.dsl` のビュー `components`
  → `docs/assets/diagrams/engine-abstraction-components.png|svg`。ビューキーは DSL のビュー宣言で明示する
  （例 `container quickscribe "containers" { ... }` の `"containers"`）。記事は PNG を絶対URLで参照する。
- **CI 挙動（重要 — Mermaid とは検証方式が異なる）**: Structurizr のレンダリングには **Java（または Docker）** が要り、
  多くの執筆環境ではローカル生成できない。よって「画像がコミット済みか」は**必須ゲートにしない**。
  代わりに CI が画像を作ってコミットする:
  - **PR**: `.dsl` を実レンダリングし、生成画像を**当該 PR ブランチへ自動コミット**する。
    赤くなるのは **DSL がレンダリングできない時だけ**（構文エラー等）。執筆者は `.dsl` を書くだけでよい。
  - **main push**: 再生成して差分があれば自動コミットする（Mermaid と同じ）。
  - `npm run structurizr:check`（コミット済み画像の存在チェック）はローカル任意。CI 判定には使わない。
  - 実体は [articles.yml](../../.github/workflows/articles.yml) の `structurizr` ジョブ。
- **記事側の使い方**: ① `docs/architecture/<name>.dsl` を置く（ビューにキーを付ける）→ ② PR を出す →
  ③ CI が `docs/assets/diagrams/<name>-<viewキー>.png` を生成・コミット →
  ④ 記事から `https://raw.githubusercontent.com/Takenori-Kusaka/QuickScribe/main/docs/assets/diagrams/<name>-<viewキー>.png` を参照。

### クラス図・ER 図は Mermaid を併用

Structurizr は C4（システム/コンテナ/コンポーネント）専用で、**クラス図・ER 図は守備範囲外**。
これらは従来どおり **Mermaid の `classDiagram` / `erDiagram`**（§2 の Mermaid 経路）で描く。
「C4 構成図 = Structurizr DSL、クラス/ER 図 = Mermaid」を使い分ける。
