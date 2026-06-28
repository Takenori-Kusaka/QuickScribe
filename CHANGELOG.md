# 変更履歴 / Changelog

本ファイルは [Keep a Changelog](https://keepachangelog.com/ja/1.1.0/) に概ね従い、
バージョンは [Semantic Versioning](https://semver.org/lang/ja/) に従います。
v0.6.4 以降は [release-please](https://github.com/googleapis/release-please) が
Conventional Commits から自動生成します（#400）。以下は導入前の主な履歴の要約です。

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
