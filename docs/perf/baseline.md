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

## 残（#403 後続）
- 日本語精度（WER/CER）ベンチ（JSUT 等サブセット、[#26](https://github.com/Takenori-Kusaka/QuickScribe/issues/26)）。
- 起動時間の計測（GUI 起動→ready）。
- メジャーバージョン間の回帰比較（前回比の閾値ゲート）。
