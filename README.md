# QuickScribe

思考整理・自己理解のための、ローカル完結ボイスジャーナル。
話した内容を、ニュアンスを残しつつ賢く成形・要約し、自分の考えを整理する。

> 企画・設計の背景は [docs/vision.md](docs/vision.md) と [docs/adr/](docs/adr/) を参照。

## 状態

開発初期（Walking Skeleton）。Phase 1 は「トレイ常駐 + ウィンドウ + 録音トグル +
グローバルホットキー + 指定フォルダ保存」を貫通させる段階。文字起こし(whisper)・
整形(LLM)・システム音声取り込み・デバイス切替・Stream Deck連携は後続の縦切りで追加する
（[ADR-0006](docs/adr/0006-scope-completeness-policy.md) によりスコープからは外さない）。

## 技術スタック

- Tauri 2（Rust）+ Svelte 5（TypeScript）— [ADR-0005](docs/adr/0005-tech-stack.md)
- 文字起こし: ローカル whisper.cpp 既定（予定）— [ADR-0002](docs/adr/0002-stt-engine-strategy.md)

## 開発

前提: Node 20+, Rust stable, および各OSのTauri依存（Linuxは webkit2gtk-4.1 等）。

```bash
npm ci
npm run icons     # アイコン生成（src-tauri/icons/）
npm run tauri dev # 開発起動
```

## ビルド / リリース

- `main` への push / PR で CI がクロスプラットフォーム(win/linux)ビルドを検証。
- `v*` タグの push で Release ワークフローがインストーラを生成し GitHub Releases へ公開。

## ライセンス

[MIT License](LICENSE)（[ADR-0008](docs/adr/0008-licensing-and-distribution.md)）。

本アプリは whisper.cpp（MIT）・libopus（BSD-3-Clause）・Tauri（MIT/Apache-2.0）等の
オープンソースを利用しており、配布物には第三者ライセンスの帰属表記
（`THIRD-PARTY-NOTICES`）を同梱する。同ファイルは CI で `cargo-about` により
依存関係から自動生成される。
