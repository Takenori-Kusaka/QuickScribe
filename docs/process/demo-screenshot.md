# スクリーンショットの自動生成（CI）と暫定プレースホルダ

> Status: Plan（2026-06-28）。README/サイト用のUIスクリーンショットは **CIで自動生成**し、リリースごとに `docs/assets/` を更新する方針。実スクショは v1.0.0 から。それまでは暫定プレースホルダを使う。

## 方針（なぜ自動化できるか）
QuickScribe のフロントは Svelte（純Web UI）。Tauri の `invoke` 以外はブラウザだけで描画できる。したがって:

1. **Vite dev** でフロントを起動（Rust/cargo/MSVC/マイク **不要**）。
2. **Playwright（ヘッドレス Chromium）** で開く。`@tauri-apps/api` の `invoke` を**モック**し、保管庫一覧などにダミーデータを返す。
3. 主要画面（メイン／保管庫）を**決め打ちの状態に遷移させて `page.screenshot()`**。
4. 生成画像を `docs/assets/` に出力。リリース時に自動コミット（または artifact）。

→ ubuntu ランナーで完結。Tauri WebView も Chromium系のため見た目はほぼ一致（README用途に十分）。

## 実装計画（別issue）
- `tests/screenshots/` に Playwright スクリプト（`capture.ts`）。Tauri IPCモック層（`mock-invoke.ts`）でエントリ一覧・設定・整形結果のフィクスチャを注入。
- `package.json` に `screenshots` スクリプト（vite起動→playwright実行）。
- `.github/workflows/screenshots.yml`: リリース（または手動 workflow_dispatch）で実行し `docs/assets/screenshot-*.png` を更新するPRを作成 or 直コミット。
- 撮る画面: ①メイン（録音→文字起こし→整形結果）②保管庫（一覧＋横断発見）。UI改修（アイコン/保管庫レイアウト）確定後に実態へ合わせる。

## 暫定プレースホルダ（現状）
- `docs/assets/screenshot-main.placeholder.png` — メイン画面（録音→文字起こし→整形）。
- `docs/assets/screenshot-vault.placeholder.png` — 保管庫/ライブラリ（一覧＋横断発見）。
- README はメインのプレースホルダを表示中。v1.0.0で自動生成画像に差し替える。

## プライバシー
- 自動生成はすべて**ダミーデータ**で描画（実ジャーナル・実APIキーを使わない）。漏えいリスクなし。
