# 記事公開・再デプロイポリシー

> Status: Living（2026-07-05 初版）。技術記事（Zenn / Qiita）の単一ソース管理・公開・再デプロイの運用規約。
> 関連: [ADR-0008 ライセンス・配布](../adr/0008-licensing-and-distribution.md) / [design.md](../design.md)。

## 目的

技術記事を「腐らない（エバーグリーン）」「転載でリンクが切れない」「本文と図・数値が乖離しない」状態で
継続提供する。プラットフォーム（Zenn / Qiita）の差異はツールで吸収し、人間の判断が必要な点（実公開）だけ
人間ゲートに残す。

## 1. 単一ソースとプラットフォーム出し分け

- **正典は `articles/*.md`（Zenn 形式）**。記事はここにだけ手で書く。
- **Qiita 版 `public/*.md` は機械生成物**。`npm run qiita:build`（`scripts/zenn-to-qiita.mjs`）で
  front-matter を変換して生成する。手で編集しない（CI が `public/` の同期ずれを検出して落とす）。
- front-matter マッピング:
  | Zenn | Qiita |
  |---|---|
  | `title` | `title` |
  | `topics` | `tags`（先頭5件） |
  | `published: false`（下書き） | `private: true` / `ignorePublish: true`（自動公開しない） |
  | `published: true` | `private: false` / `ignorePublish: false` |

## 2. 画像・図は必ず絶対URL

- 記事内の画像参照は **絶対URL**（`https://raw.githubusercontent.com/Takenori-Kusaka/QuickScribe/main/...`）にする。
  相対パスは Qiita 転載でリンク切れになるため**禁止**。`qiita:build` は保険として相対リンクを絶対URLへ書き換えるが、
  記事は最初から絶対URLで書く。
- **図は Mermaid を単一ソースにし、CI で SVG/PNG へ書き出す**（`npm run diagrams` / `scripts/render-diagrams.mjs`）。
  対象は `docs/design.md` の ```mermaid``` ブロック。出力は `docs/assets/diagrams/design-<n>.svg|png`。
  記事は PNG を絶対URLで参照する（Zenn/Qiita の双方で確実に埋め込めるため）。SVG も差分可読なソースとして併置する。
- 図の鮮度は CI が担保する: PR では `diagrams:check`（期待する図がコミット済みか）を検証し、`main` への
  push では図を再生成して差分があれば自動コミットする（[articles.yml](../../.github/workflows/articles.yml)）。

## 3. 校正（textlint 必須）

- `articles/**/*.md` は **textlint（preset-ja-technical-writing）** をゼロエラーで通すこと（`npm run lint:text`）。
  CI ジョブ（articles.yml）が必須ゲート。設定は [`.textlintrc.json`](../../.textlintrc.json)。
- 文体は **である調**に統一（設定で強制）。1文120字以内・読点4つ以内を目安に短く書く。

## 4. 公開 CI と再デプロイの頻度

- **Zenn**: Zenn ダッシュボードで GitHub リポジトリ連携を設定すれば `articles/` が **自動同期**される。
  push 型の公開 CI は不要（連携が担う）。
- **Qiita**: qiita-cli の GitHub Action（[qiita-publish.yml](../../.github/workflows/qiita-publish.yml)）で公開する。
  `QIITA_TOKEN` は Secrets 前提。未設定環境では publish を**スキップ**する（フォークでも失敗しない）。
- **再デプロイは「メジャーバージョンアップ時のみ」**（タグ `vX.0.0`）。記事は製品の設計思想を語る性質上、
  マイナー/パッチのたびに再公開しない。**細かい使い方の改訂は GitHub Pages（ドキュメントサイト）側で対応**し、
  記事は大きな節目でのみ更新・再デプロイする。

## 5. 実公開は人間ゲート

- 記事は既定で `published: false`（Zenn 下書き）＝ `private: true`（Qiita 非公開）。
- **実公開はメンテナが front-matter を切り替えて判断する**。CI は「切り替え済みの記事を、メジャー版の節目で
  反映する」役割に留める。自動で `published: true` にはしない。

## 6. 将来検討（記録）

- 図の重厚化（C4 / Structurizr DSL）は将来検討。現状は Mermaid→SVG/PNG の軽量経路で
  「CI 生成・所定配置・絶対パス」の要件を満たす（[design.md](../design.md) の図がソース）。
