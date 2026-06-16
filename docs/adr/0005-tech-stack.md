# ADR-0005: アプリケーション技術スタック

- Status: Proposed（レビュー反映済み・改訂2版。Accept前ゲートは下記スパイクの通過）
- Date: 2026-06-16
- Deciders: Takenori Kusaka
- Relates to: [ADR-0002](0002-stt-engine-strategy.md), [ADR-0003](0003-reject-google-docs-automation.md), [ADR-0004](0004-product-positioning-voice-journal.md), [ADR-0006](0006-scope-completeness-policy.md)

## Context

QuickScribe は次を**全て**満たす（[ADR-0006](0006-scope-completeness-policy.md) によりスコープは縮小しない）:

- Windows / Linux のデスクトップ常駐アプリ（タスクバー/トレイ常駐、右クリックメニュー）。
- ミニマルな配布物（軽量・起動が速い・依存ツリーが小さく監査しやすい）。
- `whisper.cpp`（C++）同梱呼び出し＋クラウドSTTプラグイン（[ADR-0002](0002-stt-engine-strategy.md)）。
- **オーディオ取り込みの全経路**: マイク入力 / **システム音声（ループバック）** / **入力デバイス切替**。
- グローバルホットキー、Stream Deck / マウスボタン等の物理トリガー。
- LLM整形（BYO認証は既存ブラウザ or 最小Webview）。
- ローカルファースト、低メモリ常駐、プライバシー（思考の生データを外に出さない）。

## Decision（提案）

**Tauri 2（Rust バックエンド + OSネイティブWebview UI）** を採用する。

- **UIフロント**: **Svelte（SPA構成、SvelteKitは使わない）** + TypeScript + Vite。単一ウィンドウ常駐UIにSSR/ルーティングは不要。
- **STT連携**: `whisper-rs`（whisper.cpp バインディング）。OSS実例 **Vibe** が Tauri + whisper-rs で先行実証。
- **トレイ/ホットキー**: Tauri `tray-icon` / `global-shortcut` プラグイン。
- **配布**: Tauri バンドラで Windows(MSI/NSIS) と Linux(AppImage/.deb) を生成し、GHリリースへ。

### 抽象境界（SOLID / DIP — 4つを一級の差し替え可能境界として定義）

レビュー指摘を受け、STTのみでなく**4境界**を明示する。実装はこれらインターフェース越しに行う。

| 境界 | 役割 | 差し替え例 |
|---|---|---|
| `TranscriptionEngine` | 音声→テキスト | whisper.cpp / kotoba-whisper / Groq / Deepgram / Azure |
| `FormattingEngine` | テキスト→整形・要約（整形の知性＝本体価値） | BYO-Cloud(Gemini/Claude) / ローカルLLM(Ollama等) |
| `AudioCapture` | 音声取得 | マイク / システム音声ループバック / デバイス選択 / **ファイル(既存音声の読込)** |
| `TriggerInput` | 起動・停止トリガー | グローバルホットキー / マウスボタン / Stream Deck |

`FormattingEngine` を Strategy 化することで、プライバシー方針（ローカル完結 vs クラウド）の将来変更にアーキを縛られず追従できる（[vision](../vision.md) オープン課題への布石）。

### オーディオ取り込みサブシステム（第一級・フルスコープ）

音声入力アプリの心臓部。Rust エコシステムでの担い手を明示する。

- **マイク入力**: `cpal`（クロスプラットフォーム）。
- **システム音声（ループバック）**:
  - Windows: WASAPI ループバック。`cpal` の対応が限定的なため、不足は `windows-rs` で WASAPI を直接叩く実装でカバー（[ADR-0003](0003-reject-google-docs-automation.md) 参照、仮想オーディオデバイス不要）。
  - Linux: PipeWire/PulseAudio の `.monitor` ソース。`pipewire-rs` ないし PulseAudio API で取得。
- **入力デバイス切替**: 利用可能なデバイスを列挙し UI から選択・実行時切替。
- **ファイルソース（既存音声の読込）**: メニューから音声ファイルを選択し `TranscriptionEngine` に流す（Story S1.6）。`AudioCapture` の一実装として、マイク/ループバックと同じ抽象境界に乗せる。**固定ファイルで決定論的なE2Eテスト**を可能にし、デバッグ効率と顧客価値（既存録音の変換）を両立する。
- **中間ファイル保存（オプション）**: 録音音声をWAV等の中間ファイルとして任意保存（Story S1.7）。デバッグ/テスト/再変換に寄与。
- これらは独立した技術検証スパイクの対象（下記ゲート）。**スコープからは外さない**（[ADR-0006](0006-scope-completeness-policy.md)）。

### Stream Deck / 物理トリガー（フルスコープ）

- ベースは `TriggerInput` 抽象に集約したグローバルホットキー（マウスボタンも同経路）。
- **Stream Deck 公式プラグイン（Node/JS SDK）も提供する**。本体(Tauri/Rust)とはローカルIPC/ホットキーで疎結合に連携。
  「ホットキー経由のみ」に縮小せず、公式プラグインまで作り切る（統合体験は vision 差別化4軸目）。

### コード署名・自動更新（信頼の根幹）

- **Windows**: Authenticode 署名（無署名は SmartScreen 警告 → 「摩擦ゼロ」に反する）。EV証明書要否・コスト・鍵管理を CI/CD ADR の前提に。
- **Linux**: AppImage/.deb の署名・チェックサム配布。
- **自動更新**: Tauri updater + 署名鍵管理。鍵管理はプライバシー約束と並ぶ信頼の柱。

### 認証情報の保管（プライバシー約束の一部）

- BYO-LLM のトークンは **OSセキュアストレージ**（Windows Credential Manager / Linux Secret Service）に格納（`keyring` crate）。
- 認証はシステム既定ブラウザの OAuth を基本、最小Webview はフォールバック。Webview時は Cookie/トークンを隔離。

## Alternatives considered（非機能基準で再評価）

| 候補 | 評価軸: 常駐メモリ / 起動速度 / whisper連携の素直さ / 依存ツリーの小ささ | 不採用理由 |
|---|---|---|
| **Electron** | 劣 / 劣 / 中（native addon・子プロセス）/ 劣（Chromium同梱） | 常駐メモリ・起動速度で劣り、whisper-rsのような成熟バインディングの優位を活かせない。依存ツリーが大きく監査面で不利。OpenWhispr等の採用例はあるが本プロジェクトの軽量・プライバシー要件に非最適 |
| **Wails (Go)** | 良 / 良 / 中 / 良 | Tauri類似だが whisper連携の先行実証が乏しく、トレイ/グローバルホットキー/Webviewの実績がTauriに劣る。Vibeのような実証事例の厚みでTauriに譲る |
| **Qt (C++/PySide)** | 中 / 中 / 良（同言語） | UI反復が遅く配布が煩雑。モダンなUX反復に不向き |
| **.NET (Avalonia)** | 中 / 中 / 中 | Linuxのトレイ/オーディオ周りの実績が弱い |

**積極的論拠**: Tauri は **Vibe（Tauri + whisper-rs）** という whisper連携の先行実証を持つ唯一格。ネイティブWebviewでChromium非同梱＝軽量・高速起動・小依存ツリーが、[ADR-0004](0004-product-positioning-voice-journal.md) のプライバシー/摩擦ゼロと整合。

## Accept前ゲート（縦切りスパイク — 通過でAcceptへ）

スコープは縮小せず、リスクは**検証**で潰す（[ADR-0006](0006-scope-completeness-policy.md)）:

1. **Linux スパイク**: 主要ディストリ（Ubuntu 22.04/24.04, Fedora, Arch）で WebKitGTK(webkit2gtk-4.1) を前提に、**日本語IME入力 × トレイ × グローバルホットキー**が動作することを実機確認。
2. **オーディオ スパイク**: Win(WASAPIループバック) / Linux(PipeWire monitor) で**マイク＋システム音声＋デバイス切替**を取得し whisper.cpp に流すまでを実証。
3. **配布スパイク**: 署名付き MSI / AppImage を生成し、SmartScreen 警告なしで起動できることを確認。

## Consequences

- 軽量・高速起動・小依存ツリー → プライバシー/摩擦ゼロ体験と整合。
- OSネイティブWebview採用で Chromium 非同梱（「既存ブラウザに依存しない」を軽量に達成）。
- **コスト**: Rust 学習/ビルド時間、Linux WebKitGTK のディストリ差異。→ ガードレール: **Rustはバックエンド配線/FFIに限定し、整形ロジック（プロンプト・逐語/要約スタイル）はTS側/LLM側に寄せる**。本体価値（整形の知性）に工数を集中する。
- CI/CD（次ADR）は Rust + Node のマトリクスビルド、署名、updater を前提に設計する。

## Open questions（解決済みとして本文に反映）

- フロント: **Svelte（SPA）に確定**。
- 整形LLMのローカル/クラウド: **両方を `FormattingEngine` Strategy で一級市民化**。既定はBYO-Cloud、ローカルLLMも差し替え可能に（フルスコープ）。
- Stream Deck: **ホットキー経由＋公式プラグインの両方を提供**（縮小しない）。
