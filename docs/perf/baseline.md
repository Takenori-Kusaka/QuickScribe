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

| 日付 | バージョン | モデル | RTF | ピークRSS | 備考 |
|---|---|---|---|---|---|
| _（perf-bench 初回実行後に記入）_ | | ggml-tiny | | | |

## 残（#403 後続）
- 日本語精度（WER/CER）ベンチ（JSUT 等サブセット、[#26](https://github.com/Takenori-Kusaka/QuickScribe/issues/26)）。
- 起動時間の計測（GUI 起動→ready）。
- メジャーバージョン間の回帰比較（前回比の閾値ゲート）。
