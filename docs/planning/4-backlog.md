# 4. プロダクトバックログ（Epic → Story → Task）

> Status: DRAFT (2026-06-16) — 全体計画(3.1-3.7)から導出した網羅バックログ。GitHub issue起票の源泉。
> 階層: **Epic**(コア価値の塊) → **Story**(顧客価値の単位＝親issue) → **Task**(実装可能単位、Storyがスプリント投入時にissue化)。
> 全Story完了でリリース＆顧客利用開始(認知含む)に至るよう網羅。[ADR-0006](../adr/0006-scope-completeness-policy.md)でスコープ非縮小。
> 各Storyは仕様(受入基準BDD/EARS)→テスト設計→TDD実装→CI(E2E含む)→レビュー→マージのDoD([3.4](3.4-spec-and-tdd-plan.md)/[3.5](3.5-waterfall-and-quality.md))。

---

## E1. Capture（録音・トリガー）
- **S1.1 マイク録音**: cpalで既定デバイス録音→16kHz mono変換。 _Tasks_: cpal録音実装 / リサンプリング / 録音状態管理 / 単体テスト。
- **S1.2 入力デバイス選択・切替**: デバイス列挙とUI選択、実行時切替。
- **S1.3 システム音声ループバック**: Win=WASAPIループバック(windows-rs)、Linux=PipeWire monitor。
- **S1.4 グローバルホットキー設定**: 開始/停止のホットキーをユーザー設定可能に。
- **S1.5 物理トリガー連携**: マウスボタン/Stream Deck（ホットキー経由＋公式プラグイン）。
- **S1.6 音声ファイルから変換（メニューインポート）**: 既存の音声ファイルを選択→文字起こし。`AudioCapture` の **FileSource** 実装。固定ファイルで**決定論的E2E**を可能にする基盤。顧客価値（既存録音/会議音声の変換）。
- **S1.7 録音音声を中間ファイルとして保存（オプション）**: 録音音声をWAV等の中間ファイルとして任意保存。デバッグ/テスト/再変換に寄与。`AudioCapture` の出力を永続化する設定。

## E2. Transcription（文字起こしエンジン）
- **S2.1 ローカルwhisper.cpp統合**: whisper-rsで録音→文字起こし→保存。
- **S2.2 モデル管理**: モデルDL/選択、日本語はkotoba-whisper、保管先管理。
- **S2.3 TranscriptionEngine抽象**: ローカル/クラウドの差し替え(Strategy)。
- **S2.4 クラウドSTTプラグイン**: Groq/Deepgram/Azure（鍵投入で有効）。
- **S2.5 日本語精度ベンチ**: JSUT/CSJ/CommonVoice jaで自前評価。

## E3. Refinement（整形の知性＝コアドメイン）
- **S3.1 FormattingEngine抽象**: 整形戦略の差し替え(DIP)。
- **S3.2 BYO-LLM認証**: 既存ブラウザOAuth、トークンはOSセキュアストレージ(keyring)。
- **S3.3 整形スタイル**: 逐語/要約/ブレストを行き来、ニュアンス保持。
- **S3.4 ローカルLLM対応**: Ollama等で完全ローカル整形（一級市民）。
- **S3.5 「リッチすぎず簡便」UX調整**: 既定の最小操作と段階的深掘りのバランス。

## E4. Journal/Storage（保管庫・エントリ）
- **S4.1 エントリ永続化**: 保管庫フォルダ(既定=ドキュメント/QuickScribe、設定上書き)へ保存。
- **S4.2 出力形式**: プレーンテキスト/Markdown、メタデータ。
- **S4.3 内省タグ・横断発見**: タグ付けと過去エントリ横断の気づき。
- **S4.4 データスキーマ版＋自動migration**: 非破壊移行（事実上のpublic API）。

## E5. Settings / Secrets
- **S5.1 設定UI**: 右クリックメニューからの設定(フォルダ/ホットキー/エンジン)。
- **S5.2 鍵管理**: OSセキュアストレージ、`.env`方針との整合。
- **S5.3 設定スキーマ**: バージョン管理と検証。

## E6. Platform / Tray / Hotkey（常駐）
- **S6.1 トレイ常駐・右クリックメニュー是正**: Phase1で未動作だった常駐挙動の修正（[タスク#12]）。
- **S6.2 タスクバーからの開始/停止**: 当初要件「タスクバーにボタン」。
- **S6.3 起動/終了/最小化/自動起動**: 常駐アプリの基本挙動。
- **S6.4 Linux WebKitGTK×IMEスパイク**: [ADR-0005](../adr/0005-tech-stack.md) Accept前ゲート。

## E7. Quality / Test harness
- **S7.1 Rust単体テスト基盤**: `cargo test`、ドメインロジックのTDD。
- **S7.2 フロント単体(vitest)**: UIロジックのテスト。
- **S7.3 E2E(tauri-driver+WebdriverIO)**: Linux+xvfbで実起動UI操作。**前回欠落の是正**。
- **S7.4 CI必須ゲート化**: lint/型/単体/結合/E2E/セキュリティを必須。
- **S7.5 セキュリティ**: CodeQL/Dependabot/cargo-audit/cargo-deny/npm audit。

## E8. Release / Ops
- **S8.1 release-please導入**: Cargo.toml+tauri.conf版同期、Release PRゲート。
- **S8.2 リリースパイプライン**: tauri-action、updater latest.json(.sig)、provenance+SHA256SUMS。
- **S8.3 Windowsコード署名**: SSL.com eSigner（**創業者本人確認ゲート**）。
- **S8.4 配布チャネル**: winget/AppImage/.deb（条件付きFlathub）。
- **S8.5 バージョニング/チャネル**: 0.x維持、stable/nightly二系統、updater鍵分離・Runbook。
- **S8.6 SECURITY.md/PVR/サポート**: 最新版のみ・ベストエフォート、coordinated disclosure。
- **S8.7 リポジトリ堅牢化**: ブランチ保護(全PR・CI必須)、セキュリティ設定。

## E9. Marketing / Launch（顧客認知・導入）
- **S9.1 README整備**: ロゴ/GIFデモ/Privacy節/Quick start/Win・Linuxバッジ。
- **S9.2 ドキュメントサイト**: VitePress、GitHub Pages。
- **S9.3 オンボーディング**: オピニオン型(既定日記1件＋初回録音→即文字起こし=aha)。
- **S9.4 ローカル通知・習慣ナッジ**: アンカーベース、寛容なストリーク、サーバー無し。
- **S9.5 ローンチ(48hウィンドウ)**: Show HN+GitHub Trending+Reddit+PH+X。
- **S9.6 計測**: GitHub DL数/スター/cookielessリファラ、オプトインのみ。

## E10. Foundation（基盤・ほぼ完了）
- 企画/ADR/CLAUDE.md/調査/Walking Skeleton/CI/Release雛形（完了済み）。残: ブランチ保護はS8.7。

---

## スプリント0/1の優先（実装着手順）
1. **E7（テスト基盤）先行** — 以降を「起動・動作確認済み」にする土台（S7.1→S7.3→S7.4）。
2. **E6.1 トレイ是正** — Phase1の負債解消。
3. **E1.1→E2.1→E4.1**（録音→文字起こし→保存）の貫通＝次バイナリのコア。
4. **E3（整形）** — コアドメイン本体。

事業判断待ち（並行）: 収益モデル(3.1) / Windows署名契約(S8.3)。
