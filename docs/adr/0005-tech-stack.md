# ADR-0005: アプリケーション技術スタック

- Status: Proposed
- Date: 2026-06-16
- Deciders: Takenori Kusaka
- Relates to: [ADR-0002](0002-stt-engine-strategy.md), [ADR-0003](0003-reject-google-docs-automation.md), [ADR-0004](0004-product-positioning-voice-journal.md)

## Context

QuickScribe は次を満たす必要がある:

- Windows / Linux のデスクトップ常駐アプリ（タスクバー/トレイ常駐、右クリックメニュー）。
- **ミニマルな配布物**（「ミニマルなものを梱包」という要件、ローカル/プライバシー志向と整合）。
- グローバルホットキー、Stream Deck / マウスボタン等の物理トリガー対応。
- `whisper.cpp`（C++）の同梱・呼び出し（[ADR-0002](0002-stt-engine-strategy.md)）。
- LLM整形の BYO認証は既存ブラウザ or 最小Webviewで（[ADR-0003](0003-reject-google-docs-automation.md) によりSTT用ブラウザ自動化は廃止済み）。
- ローカルファースト、低メモリ、起動が速い。

## Decision（提案）

**Tauri 2（Rust バックエンド + OSネイティブWebviewのUI）** を採用する。

- UIフロントエンド: 軽量フレームワーク（**SvelteKit** もしくは SolidJS）+ TypeScript。
- STT連携: `whisper-rs`（whisper.cpp の Rust バインディング）。OSS実例 **Vibe** が Tauri + whisper-rs で先行実証済み。
- トレイ/ホットキー: Tauri の `tray-icon` と `global-shortcut` プラグイン。
- 配布: Tauri バンドラで Windows(MSI/NSIS) と Linux(AppImage/.deb) を生成し、GHリリースに添付。

## Alternatives considered

| 候補 | 長所 | 不採用理由 |
|---|---|---|
| **Electron** | 巨大エコシステム、実装容易、競合(OpenWhispr)採用 | Chromium同梱で重い（〜150MB+）。「ミニマル梱包」「軽量ローカル」の本質に反する |
| **Wails (Go)** | 軽量、Tauri類似 | トレイ/グローバルホットキー/Webviewの成熟度がTauriに劣る。whisper連携実例が乏しい |
| **Qt (C++/PySide)** | ネイティブ、whisper.cppと同言語 | UI開発が遅い、配布が煩雑、モダンなUX反復に不向き |
| **.NET (Avalonia)** | C#、クロスプラットフォーム | Linuxのトレイ/オーディオ周りの実績が弱い |

## Consequences

- 配布物が小さく起動が速い → プライバシー/ローカル完結の体験価値（[ADR-0004](0004-product-positioning-voice-journal.md)）と整合。
- OSネイティブWebview（Win=WebView2, Linux=WebKitGTK）を使うため Chromium 非同梱。「既存ブラウザに依存しない」を満たしつつ軽量。
- **コスト**: Rust の学習・ビルド時間。Linux Webview のディストリ差異（WebKitGTKバージョン）に注意が要る。
- CI/CD（次ADR）は Rust + Node のクロスコンパイル/マトリクスビルドを前提に設計する。
- BYO-LLM認証は、初期は「システム既定ブラウザでOAuth/ログイン → トークン受領」を基本とし、最小Webviewはフォールバックとする（セキュリティ上、認証は隔離したい）。

## Open questions（レビューで詰める）

- フロントは Svelte と Solid のどちらか（バンドルサイズ vs エコシステム）。
- LLM整形をローカルLLM（Ollama等）で完結させる選択肢を一級市民にするか（プライバシー約束の強化）。
- Stream Deck連携は公式プラグイン(Rust/Node SDK)か、汎用ホットキー経由か。
