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

## 1. 単一ソースと構成（連載は Zenn Book に束ねる）

- **連載（体系立てた設計解説）は Zenn Book で管理する**。正典は `zenn-content` サブモジュール内の `zenn-content/books/<book-slug>/`（`config.yaml` ＋ 章 Markdown）。
  章の順序は `config.yaml` の `chapters` 配列（章 slug＝ファイル名）で明示する。**`chapters` に未列挙の章は Zenn 上で
  非公開になる**点に注意（書きかけ章のドラフト管理に使える）。章ファイルの front-matter は `title:` のみ（有料本で
  個別無料公開したい章だけ `free: true`）。本の公開可否・価格・topics は `config.yaml`（`published` / `price` / `topics`）。
  - 現行の本: `zenn-content/books/quickscribe-design/`（本連載）。フォルダ名 `quickscribe-design` がそのまま Zenn の本 slug になる。
  - **Zenn の `articles/` はサブフォルダを許さない（平置きのみ）**ため、章をフォルダで束ねるには Book を使う。
- 単発の記事（連載に属さない読み切り）を書く場合のみ `zenn-content/articles/<slug>.md`（平置き）に置く。
- どちらも `Takenori-Kusaka/zenn-content` リポジトリの GitHub 連携で自動同期される。

## 2. 図の作法（Zenn 画像作法に従う）

- **記事内の画像は Zenn 作法に従う**: PNG を `zenn-content` サブモジュールの `images/`（Zenn 配信の基準ルート）配下に置き、記事本文からは
  **絶対パス `![](/images/...png)`** で参照する（Zenn が `zenn-content` リポジトリの `images/` を配信する）。
  外部の raw URL（`raw.githubusercontent.com/...`）は非推奨。
- **クラス図・ER 図は Mermaid をインラインで書く**（コードフェンス ```mermaid ... ```）。
  Zenn も GitHub もこれを直接描画するため、画像への事前ラスタライズは不要。
  Zenn は記事内 **SVG を表示しない**点に注意（画像化が要るものは必ず PNG にする）。
- **C4 構成図は Structurizr DSL → 決定的 PNG**（§6）。出力は `zenn-content/images/c4/`、参照は `/images/c4/xxx.png`。

## 3. 校正（textlint 必須）

- `zenn-content/books/**/*.md` や `zenn-content/articles/**/*.md` は **textlint（preset-ja-technical-writing）** をゼロエラーで通すこと（`npm run lint:text`）。
  CI ジョブ（articles.yml）が必須ゲート。設定は [`.textlintrc.json`](../../.textlintrc.json)。
- 文体は **ですます調**に統一（設定で強制）。1文120字以内・読点4つ以内を目安に短く書く。

## 4. 公開と再デプロイの頻度

- **Zenn の GitHub 連携**で `Takenori-Kusaka/zenn-content` リポジトリ側の `articles/` と `books/` が **自動同期**される。
- **再デプロイは「メジャーバージョンアップ時のみ」**（タグ `vX.0.0`）。記事は製品の設計思想を語る性質上、
  マイナー/パッチのたびに再公開しない。**細かい使い方の改訂は GitHub Pages（ドキュメントサイト）側で対応**し、
  記事は大きな節目でのみ更新・再デプロイする。

## 5. 実公開は人間ゲート

- 記事・本は既定で `published: false`（Zenn 下書き）。
- **実公開はメンテナが front-matter を切り替えて判断する**。CI は自動で `published: true` にしない。

## 6. C4 アーキ図は Structurizr DSL → 決定的 PNG（実装済み）

C4 モデルの構成図（System Context / Container / Component 等）は **Structurizr DSL を単一ソース**にし、
`zenn-content/images/c4/` に書き出す。

- **経路（決定的）**: Structurizr CLI `export -format plantuml` で DSL を C4-PlantUML(`.puml`) に変換し、
  `plantuml -tpng`（Graphviz レイアウト）で PNG 化する。**random ID なし・レイアウト決定的**なので、
  DSL が同じなら PNG はバイト単位で不変 → 差分ゼロ → churn が起きない。
  スクリプトは [`scripts/render-structurizr.mjs`](../../scripts/render-structurizr.mjs)。
- **DSL ソース**: `docs/architecture/*.dsl`（1 workspace = 1 ファイル。1 ファイルに複数ビュー可）。
- **出力**: `zenn-content/images/c4/<dslのbasename>-<viewキー>.png`（SVG は作らない＝Zenn は SVG 非表示のため）。
  例: `docs/architecture/engine-abstraction.dsl` のビュー `components`
  → `zenn-content/images/c4/engine-abstraction-components.png`。ビューキーは DSL のビュー宣言で明示する
  （例 `container quickscribe "containers" { ... }` の `"containers"`）。
- **CI 挙動**: レンダリングには **Java + PlantUML + Graphviz** が要り、多くの執筆環境ではローカル生成できない。
  CI は container `ghcr.io/sebastienfi/structurizr-cli-with-bonus:latest`（CLI + plantuml + graphviz + git 同梱）で
  画像を再生成し、コミット漏れ（不一致）がないか検証する。実装は [articles.yml](../../.github/workflows/articles.yml) の `structurizr` ジョブ。
  - **PR / main とも**: `.dsl` から画像を再生成し、サブモジュール `zenn-content` 内に差分が生じた場合は CI エラーとする（構文エラーやコミット漏れの検知）。
  - `npm run structurizr:check`（コミット済み PNG の存在チェック）はローカル任意。CI 判定には使わない。
- **記事側の使い方**:
  ① `docs/architecture/<name>.dsl` を置く（ビューにキーを付ける）
  ② `npm run structurizr` をローカルで実行
  ③ サブモジュール `zenn-content` 側で更新された画像をコミット & プッシュ
  ④ 本リポジトリ側でサブモジュールの参照ポインタを更新して PR を出す
  ⑤ 記事から Zenn 作法の絶対パス `![](/images/c4/<name>-<viewキー>.png)` で参照する。

### クラス図・ER 図は Mermaid をインラインで

Structurizr は C4（システム/コンテナ/コンポーネント）専用で、**クラス図・ER 図は守備範囲外**。
これらは **Mermaid の `classDiagram` / `erDiagram`** を記事本文にインラインで書く（§2）。
Zenn/GitHub が直接描画するため画像化は不要。「C4 構成図 = Structurizr DSL、クラス/ER 図 = Mermaid インライン」を使い分ける。
