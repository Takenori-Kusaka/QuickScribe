# 変更履歴 / Changelog

本ファイルは [Keep a Changelog](https://keepachangelog.com/ja/1.1.0/) に概ね従い、
バージョンは [Semantic Versioning](https://semver.org/lang/ja/) に従います。
v0.6.4 以降は [release-please](https://github.com/googleapis/release-please) が
Conventional Commits から自動生成します（#400）。以下は導入前の主な履歴の要約です。

## [0.7.0](https://github.com/Takenori-Kusaka/QuickScribe/compare/v0.6.4...v0.7.0) (2026-06-29)


### ✨ 新機能 / Features

* **i18n:** i18n基盤(svelte-i18n)導入＋メイン画面スライスをキー化 ([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401) Phase1) ([#434](https://github.com/Takenori-Kusaka/QuickScribe/issues/434)) ([717a928](https://github.com/Takenori-Kusaka/QuickScribe/commit/717a928b9cc92ba3a10501d56f97d913c8502274))
* **i18n:** ジャーナルパネルをキー化 ([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401) Phase1) ([#436](https://github.com/Takenori-Kusaka/QuickScribe/issues/436)) ([7646ebd](https://github.com/Takenori-Kusaka/QuickScribe/commit/7646ebdefe325bf825aa7a4e715881b8a4707e3e))
* **i18n:** 結果・アクション領域＋ヘッダ補助ラベルをキー化 ([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401) Phase1) ([#435](https://github.com/Takenori-Kusaka/QuickScribe/issues/435)) ([ee60470](https://github.com/Takenori-Kusaka/QuickScribe/commit/ee60470f6717cee61242247471ef96e633e57092))


### ♻️ リファクタ / Refactor

* **front:** refine引数組み立てをlib抽出＋カバレッジゲート有効化 ([#402](https://github.com/Takenori-Kusaka/QuickScribe/issues/402) Phase2) ([#428](https://github.com/Takenori-Kusaka/QuickScribe/issues/428)) ([8442de2](https://github.com/Takenori-Kusaka/QuickScribe/commit/8442de27c5adba72cdc762d818e290d058b99797))
* **front:** モデルキャッシュ鮮度・横断発見ロジックをlib抽出＋テスト ([#402](https://github.com/Takenori-Kusaka/QuickScribe/issues/402) Phase2) ([#430](https://github.com/Takenori-Kusaka/QuickScribe/issues/430)) ([31736b1](https://github.com/Takenori-Kusaka/QuickScribe/commit/31736b1c7eaea809b3581f0bf5e3e06039693cbc))


### 📝 ドキュメント / Docs

* HANDOFF更新(v0.6.4配信・[#402](https://github.com/Takenori-Kusaka/QuickScribe/issues/402)ゲート・[#403](https://github.com/Takenori-Kusaka/QuickScribe/issues/403) perf実測・[#427](https://github.com/Takenori-Kusaka/QuickScribe/issues/427)) ([#433](https://github.com/Takenori-Kusaka/QuickScribe/issues/433)) ([496c491](https://github.com/Takenori-Kusaka/QuickScribe/commit/496c491e7d5a32800e7ed3877b4f69d7801e9f93))
* **perf:** 初回ベースライン実測を記録(RTF 0.857) ([#403](https://github.com/Takenori-Kusaka/QuickScribe/issues/403)) ([#432](https://github.com/Takenori-Kusaka/QuickScribe/issues/432)) ([478fc46](https://github.com/Takenori-Kusaka/QuickScribe/commit/478fc468fd9c0e962a946c3c93f5157327cf5a1e))

## [0.6.4](https://github.com/Takenori-Kusaka/QuickScribe/compare/v0.6.3...v0.6.4) (2026-06-28)


### ✨ 新機能 / Features

* **a11y:** モーダルのdialog化・フォーカストラップ・Esc閉じ＋コントラスト是正 ([#395](https://github.com/Takenori-Kusaka/QuickScribe/issues/395)) ([#413](https://github.com/Takenori-Kusaka/QuickScribe/issues/413)) ([4ea8ade](https://github.com/Takenori-Kusaka/QuickScribe/commit/4ea8adee45f86e81951ae27444eafca4642345c6))
* **robustness:** 入力ファイルのサイズ上限ガード＋対応形式/上限のUI通知 ([#397](https://github.com/Takenori-Kusaka/QuickScribe/issues/397)) ([#414](https://github.com/Takenori-Kusaka/QuickScribe/issues/414)) ([6cd301f](https://github.com/Takenori-Kusaka/QuickScribe/commit/6cd301f38e98aade10fbf0d0bd173892903b89cc))
* **ui:** ヘッダUX改善(保管庫→ジャーナル/SVGアイコン/IA整理) ([#388](https://github.com/Takenori-Kusaka/QuickScribe/issues/388)) ([f994674](https://github.com/Takenori-Kusaka/QuickScribe/commit/f9946748919e586c238fd6bd25cf7abcf34708f5))


### 🐛 修正 / Bug Fixes

* **settings:** 設定項目の誤分類を修正(タスクバー/自動起動→アプリ全般) ([#404](https://github.com/Takenori-Kusaka/QuickScribe/issues/404)) ([#422](https://github.com/Takenori-Kusaka/QuickScribe/issues/422)) ([66d54ad](https://github.com/Takenori-Kusaka/QuickScribe/commit/66d54adf0c16ab546c4313d801595e068d380ccb))
* **ui:** 内部ID"S2.2"露出を除去＋エラー文言をユーザー向けに整備 ([#398](https://github.com/Takenori-Kusaka/QuickScribe/issues/398)) ([#411](https://github.com/Takenori-Kusaka/QuickScribe/issues/411)) ([1fa8011](https://github.com/Takenori-Kusaka/QuickScribe/commit/1fa8011714a482b7ec36196fb5b83e2c72e867ec))


### ♻️ リファクタ / Refactor

* **front:** App.svelteの純粋関数をlib抽出＋テスト ([#402](https://github.com/Takenori-Kusaka/QuickScribe/issues/402) Phase2) ([#425](https://github.com/Takenori-Kusaka/QuickScribe/issues/425)) ([c81794d](https://github.com/Takenori-Kusaka/QuickScribe/commit/c81794d29e033651d884cdd7bf2cd4f466b6ee76))
* **front:** プロバイダ定義・定数を constants.ts に集約(SSOT) ([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401) Phase0) ([#410](https://github.com/Takenori-Kusaka/QuickScribe/issues/410)) ([752af0c](https://github.com/Takenori-Kusaka/QuickScribe/commit/752af0c4be1a585bae9b89cee5866bf677a9f6ef))
* **front:** プロバイダ鍵バリデーションをlib抽出＋テスト ([#402](https://github.com/Takenori-Kusaka/QuickScribe/issues/402) Phase2) ([#426](https://github.com/Takenori-Kusaka/QuickScribe/issues/426)) ([e742ec0](https://github.com/Takenori-Kusaka/QuickScribe/commit/e742ec0df420db521f916c16dfad527de7976f04))


### 📝 ドキュメント / Docs

* **community:** 行動規範(Contributor Covenant 2.1)＋Issueテンプレ config.yml (E8) ([3bc6f6f](https://github.com/Takenori-Kusaka/QuickScribe/commit/3bc6f6f5c535b8cc1b3797fd5f3a40ddf11eea1c))
* **github:** CODEOWNERS＋SUPPORT.md 追加 (OSSガバナンス / [#406](https://github.com/Takenori-Kusaka/QuickScribe/issues/406)) ([5865f52](https://github.com/Takenori-Kusaka/QuickScribe/commit/5865f5296f2e385cfa4786a5a38f77ecdbd7e41e))
* HANDOFF引継ぎ資料を最新化(v1.0.0プログラム進行状況) ([#423](https://github.com/Takenori-Kusaka/QuickScribe/issues/423)) ([05045c7](https://github.com/Takenori-Kusaka/QuickScribe/commit/05045c7b82d376cdc7ea8ce08daeacc7a1a8b834))
* **license:** npm依存の第三者ライセンス帰属を生成・公開 ([#394](https://github.com/Takenori-Kusaka/QuickScribe/issues/394)) ([#421](https://github.com/Takenori-Kusaka/QuickScribe/issues/421)) ([555c8b8](https://github.com/Takenori-Kusaka/QuickScribe/commit/555c8b8aeb2026cdd992538f9adb87e571e945ef))
* **planning:** v1.0.0 リリース・レディネス監査(18観点の統合) (E9) ([df6e7fc](https://github.com/Takenori-Kusaka/QuickScribe/commit/df6e7fc63f313907df4d31603b7f78f796ea2d24))
* **process:** デモGIFの実機キャプチャ手順 (S9.1 [#55](https://github.com/Takenori-Kusaka/QuickScribe/issues/55)) ([d29de27](https://github.com/Takenori-Kusaka/QuickScribe/commit/d29de27beb05f5b9a2a35f701a6bdd5d24e275ce))
* **readme:** バッジ/特徴/Quick start/Privacy節に刷新 (S9.1 [#55](https://github.com/Takenori-Kusaka/QuickScribe/issues/55)) ([925cce3](https://github.com/Takenori-Kusaka/QuickScribe/commit/925cce3b1aa58827b7bad96e07043d631744de59))
* **readme:** 視覚素材をGIFからスクリーンショットに変更 (S9.1 [#55](https://github.com/Takenori-Kusaka/QuickScribe/issues/55)) ([31ab6bd](https://github.com/Takenori-Kusaka/QuickScribe/commit/31ab6bd0b1ed15b967af2406c6cdfe754170532e))
* **research:** 競合ランドスケープ分析を作成(一次情報) ([#399](https://github.com/Takenori-Kusaka/QuickScribe/issues/399)) ([#424](https://github.com/Takenori-Kusaka/QuickScribe/issues/424)) ([b88d7f8](https://github.com/Takenori-Kusaka/QuickScribe/commit/b88d7f83aa6213cf1671dce29571b470f7a53d60))
* **site:** 「保管庫」表記をアプリに合わせ「ジャーナル」へ統一 ([#388](https://github.com/Takenori-Kusaka/QuickScribe/issues/388)追従) ([bc54195](https://github.com/Takenori-Kusaka/QuickScribe/commit/bc54195179b3068a387c41546850aa9b040eea19))
* アーキテクチャ設計(design.md)＋非機能要件(NFR)集約 ([#390](https://github.com/Takenori-Kusaka/QuickScribe/issues/390)) ([#420](https://github.com/Takenori-Kusaka/QuickScribe/issues/420)) ([e0fad8b](https://github.com/Takenori-Kusaka/QuickScribe/commit/e0fad8b204757eda6cd43630bb915bdb2bee042d))
* ドキュメントの矛盾・陳腐化を解消(ADR索引/署名方針/vision) ([#390](https://github.com/Takenori-Kusaka/QuickScribe/issues/390)) ([#419](https://github.com/Takenori-Kusaka/QuickScribe/issues/419)) ([3554622](https://github.com/Takenori-Kusaka/QuickScribe/commit/3554622193b449f0ebabfbe3458e495c2377e329))

## [0.6.3] - 2026-06-27

### ✨ 新機能 / Features

- 用語補正フェーズ（誤変換の確認・置換）を整形前に追加（#384）
- 設定・エントリのスキーマ版管理＋検証（#382）
- GitHub Pages ドキュメントサイト（VitePress）公開（#385）

### 📝 ドキュメント / Docs

- SECURITY.md（最新版のみ / 非公開報告 / プライバシー方針）
- 配布・署名の手順とコストゼロ方針（SignPath OSS 無料署名）

### 🔧 その他 / Internal

- CI: crates.io ダウンロードの HTTP/2 フレーク対策（retry + HTTP/1.1）（#383）

## [0.6.0] - 2026-06-27

### ✨ 新機能 / Features

- クラウド文字起こしエンジン（Groq / OpenAI / Deepgram / Azure）の選択に対応
- 整形プロバイダの拡充（Gemini / Anthropic / OpenAI / Ollama / AWS Bedrock / Claude Platform on AWS）

## [0.2.7] - 2026-06

### ✨ 新機能 / Features

- システム音ループバック録音（Windows / WASAPI）
- 物理トリガー（グローバルホットキー・モーメンタリ録音）

## [0.1.0] - 2026-06

### ✨ 新機能 / Features

- Walking Skeleton: トレイ常駐・録音トグル・ローカル whisper 文字起こし・整形・保管庫保存の縦断実装

[0.6.3]: https://github.com/Takenori-Kusaka/QuickScribe/releases/tag/v0.6.3
[0.6.0]: https://github.com/Takenori-Kusaka/QuickScribe/releases/tag/v0.6.0
[0.2.7]: https://github.com/Takenori-Kusaka/QuickScribe/releases/tag/v0.2.7
[0.1.0]: https://github.com/Takenori-Kusaka/QuickScribe/releases/tag/v0.1.0
