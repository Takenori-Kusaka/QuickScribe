# パフォーマンス・ベースライン

> Status: Living（2026-06-29 初版）。計測方法と基準値を記録する（#403）。実測は `perf-bench` ワークフローのアーティファクト（`perf-report.md`）が一次情報。

## 計測方法

- **ワークフロー**: `.github/workflows/perf.yml`（`workflow_dispatch` ＋ release 公開時）。
- **対象**: ローカル whisper（`ggml-tiny`、Linux x64、決定的 AVX2 ベースライン / [ADR-0012](../adr/0012-windows-multiarch-multisimd-distribution.md)）。
- **指標**:
  - **RTF（実時間比）** = 文字起こし経過秒 / 音源長秒。**< 1.0 が実時間以内**の目安。
  - **ピークメモリ（RSS）** = `/usr/bin/time -v` の Maximum resident set size。
- 固定音源（espeak-ng の既知発話・約十数秒）で計測。コンパイルは事前ビルド（`--no-run`）で計測区間から除外。

## 目標（暫定 / [NFR](../non-functional-requirements.md)）

| 指標 | 目標（暫定） |
|---|---|
| RTF（tiny / x64 AVX2） | ≤ 1.0（実時間以内） |
| ピークメモリ（文字起こし時） | ≤ 300 MB 目安 |

## ベースライン実測

| 日付 | バージョン | モデル | 音源長 | RTF | ピークRSS | 備考 |
|---|---|---|---|---|---|---|
| 2026-06-29 | v0.6.4時点 | ggml-tiny | 18.88s | **0.857** ✅ | 1518.9 MB※ | GitHub Actions ubuntu-22.04 x64・AVX2。RTF<1.0=実時間以内（目標達成） |

> ※ ピークRSSは **`cargo test` ハーネス全体**の値（テストプロセス＋ビルド成果物＋whisperコンテキスト）であり、**配布アプリのアイドル時メモリではない**。アプリ実体のメモリ計測（GUI起動時）は #403 後続で別途行う。RTF は実用域（実時間以内）を確認。

## 起動時間・アイドルメモリ（#554 / #403）

- `perf.yml` の `startup-time` ジョブが xvfb ヘッドレスで `run()` 入口→フロント `onMount` の経過をアプリ計装で記録（`QS_PERF_STARTUP=1`）。
- 同ジョブで ready 後に**アプリ実プロセス(quickscribe)のアイドル RSS** をサンプルしピーク値を記録（`cargo test` ハーネス値ではなく配布バイナリ実体）。値は `startup-report.md` アーティファクト＋ジョブサマリが一次情報。

## 日本語精度（CER / #26）

- **方法**: 本人音読のパブリックドメイン作品3点（`src-tauri/tests/fixtures/ja-accuracy`）を `QS_LANG=ja` で認識し、`scripts/cer_ja.py`（NFKC・約物空白除去・文字単位 Levenshtein / 参照長）で CER を算出。`perf.yml` の「日本語精度 CER」ジョブが計測。
- **注**: 原文へのルビ混入で絶対CERは悲観側。N=3。**相対/回帰指標**として扱う（絶対精度の主張には使わない）。

### 初期ベースライン（平均CER・ローカル実測 Windows/opus/ja）

| モデル | 平均CER | 位置づけ |
|---|---|---|
| ggml-tiny | 56.9% | 日本語で base に完全劣位（[ADR-0022]） |
| ggml-base | 44.5% | 頑健な既定 |
| kotoba-whisper v2.0 q5 | **40.2%** | 日本語推奨（素材により幻覚で悪化しうる） |

- 回帰ゲートの基準値は `docs/perf/ja-cer-baseline.json`（margin=5pt）。CI 初回実行後に環境差を踏まえ更新する。

## 残（#403 後続 / 小粒）
- **録音時**メモリ（ヘッドレスで録音を駆動する必要があり別途）。アイドル時RSSは計測済み。
- RTF/メモリの CI ハード変動を吸収する統計的な回帰ゲート（精度CERは閾値ゲート実装済み）。

[ADR-0022]: ../adr/0022-model-catalog-curation.md
