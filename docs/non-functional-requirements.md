# 非機能要件（NFR）

> Status: Living（2026-06-28 初版）。性能・可用性/信頼性・セキュリティ・プライバシーの目標値と測定方法を集約する。
> 各特性は ISO/IEC 25010 の品質特性に対応（[3.5 waterfall-and-quality](planning/3.5-waterfall-and-quality.md)）。
> 性能・精度の**実測ベースライン**は [#403](https://github.com/Takenori-Kusaka/QuickScribe/issues/403)（ベンチCI）で確立する。本書の数値目標は計測前は「目標（暫定）」。

## 1. 性能 (Performance Efficiency)

| 指標 | 目標（暫定） | 測定方法 | 状態 |
|---|---|---|---|
| 起動時間（プロセス起動→操作可能） | ≤ 2 秒（キャッシュ温時） | `perf.yml` の `startup-time` ジョブ（アプリ計装 `QS_PERF_STARTUP=1`） | 計測済（#554・`startup-report.md` アーティファクトが一次情報・[perf/baseline.md](perf/baseline.md)） |
| ローカル文字起こし RTF（実時間比, tiny, x64 AVX2） | ≤ 1.0（実時間以内） | 固定音源で計測（`.github/workflows/perf.yml`） | 実測 **0.857**（2026-06-29・[perf/baseline.md](perf/baseline.md)・達成 ✅） |
| アイドル時メモリ（RSS） | ≤ 300 MB 目安 | `perf.yml` の `startup-time` ジョブ（配布バイナリ実体の RSS サンプリング） | 計測済（#554・`startup-report.md` が一次情報） |
| 日本語精度（CER・相対/回帰指標） | ベースライン比 +5pt 以内（回帰ゲート） | `perf.yml` の「日本語精度 CER」ジョブ（本人音読PD3作品・`scripts/cer_ja.py`） | 実測確定（#26/#403: tiny 56.9% / base 44.0% / kotoba-q5 38.3%・[perf/baseline.md](perf/baseline.md)・[ADR-0022](adr/0022-model-catalog-curation.md)） |
| 録音→停止→文字起こし開始の体感遅延 | 即時（非同期・UIブロックなし） | 実装で担保（バックグラウンド文字起こし） | 実装済 |

- 文字起こしは別スレッド＋イベント通知で UI をブロックしない（`transcribe-done`/`progress`）。
- whisper は決定的 AVX2 ベースライン（[ADR-0012](adr/0012-windows-multiarch-multisimd-distribution.md)）。AVX512最適化は将来の別ビルドで。
- 日本語精度（CER）ベンチは再現可能化済み（#26/#403）。fixtures は本人音読のパブリックドメイン3作品（`src-tauri/tests/fixtures/ja-accuracy`）、回帰ゲート基準は `docs/perf/ja-cer-baseline.json`。

## 2. 可用性・信頼性 (Reliability)

| 要件 | 受入基準 | 状態 |
|---|---|---|
| 設定・エントリの非破壊マイグレーション | スキーマ版が上がっても既存データを壊さず移行（[ADR-0017](adr/0017-schema-versioning-and-migration.md)） | 実装済（検証あり） |
| 文字起こし失敗時の安全側挙動 | 例外を握り潰さずユーザー向けエラーを表示、録音状態を破綻させない | 実装済（[#398](https://github.com/Takenori-Kusaka/QuickScribe/issues/398)） |
| 巨大入力に対する堅牢性 | サイズ上限ガードで無警告ブロック/メモリ膨張を防ぐ | 実装済（[#397](https://github.com/Takenori-Kusaka/QuickScribe/issues/397)） |
| 自動更新の完全性 | minisign 署名で配布物の改ざんを検出 | 実装済 |
| クラッシュ耐性 | 録音中クラッシュ時にデータ保護（保存済みエントリは独立ファイル） | 設計で担保（プレーンファイル保存） |

## 3. セキュリティ (Security)

詳細は [SECURITY.md](../SECURITY.md)。要点:

| 観点 | 方針 | 状態 |
|---|---|---|
| 秘密情報 | APIキー/AWS資格情報は OS keyring 保存。平文設定ファイルに置かない | 実装済 |
| 権限最小化 | Tauri capabilities を必要最小（global-shortcut/dialog/updater/process/autostart/notification）に限定 | 実装済（notification は習慣ナッジ #58 / [ADR-0023](adr/0023-habit-nudge.md)） |
| XSS/注入 | `{@html}` 等の DOM 注入シンクを使わない（lint で `no-at-html-tags`=error） | 実装済 |
| 多層防御 | 制限的 CSP の設定 | 実装済（[#455](https://github.com/Takenori-Kusaka/QuickScribe/pull/455)・`tauri.conf.json`） |
| 供給網 | Dependabot / Secret scanning / CodeQL / cargo-audit/deny / Private Vulnerability Reporting | 有効化済 |
| モデルDL完全性 | whisperモデルの SHA256＋サイズ検証 | 実装済（[#457](https://github.com/Takenori-Kusaka/QuickScribe/pull/457)・`model.rs`） |

## 4. プライバシー (Privacy)

詳細は [サイトのプライバシーポリシー](https://takenori-kusaka.github.io/QuickScribe/privacy)。要点:

- **既定でローカル完結**: 録音・ローカル文字起こし（whisper.cpp/kotoba-whisper）・整形前処理は端末内。**音声は既定で外部送信されない**。
- **クラウド連携はオプトインのみ**: クラウドSTT（Groq/OpenAI/Deepgram/Azure）・整形LLM（Gemini/Anthropic/OpenAI/Ollama/AWS）はユーザーが明示選択し鍵を入力した場合に限り対象データを送信。
- **テレメトリ・解析なし**: 利用状況の収集・送信を行わない（コードと一致）。
- 自動更新の通信に個人識別情報を含めない。

## 5. アクセシビリティ (Usability / Accessibility)

- 目標: **WCAG 2.1 レベル AA**（= JIS X 8341-3:2016 AA を包含・デジタル庁推奨）。詳細チェックリストと進捗は [#395](https://github.com/Takenori-Kusaka/QuickScribe/issues/395)。
- 実装済: モーダルの dialog 化・フォーカストラップ・Esc 閉じ・本文コントラスト AA・`prefers-reduced-motion`・入力ラベル網羅・axe CI（`e2e/a11y.spec.ts`、WCAG 2.0/2.1 A+AA を main の必須チェックとして常時実行）。
- 残: NVDA/ナレーターによる手動スクリーンリーダー検証（実施記録を残す）。

## 6. 国際化 (i18n)

- **4言語出荷済み**: 日本語（既定）＋英語/中国語/スペイン語（[#454](https://github.com/Takenori-Kusaka/QuickScribe/pull/454)）。UI/設定/結果/オンボ/整形出力言語まで全キー化・4言語パリティ。起動ロケール=保存設定>OS言語>ja。
- 残（[#462](https://github.com/Takenori-Kusaka/QuickScribe/issues/462)・Phase2）: Rustバックエンドのエラー/ステータス文字列のコード化（現状 `detail` に日本語が残る）。
