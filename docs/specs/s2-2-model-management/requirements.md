# S2.2 モデル管理（DL/選択/kotoba-whisper） — requirements

> Status: Draft (2026-06-27) / 対象 Issue: #23（Epic E2）。
> 記法: 軽量BDD＋EARS。ローカルSTT(プライバシー既定)の実用性を高める。

## ユビキタス言語
- **whisperモデル**: ローカル文字起こしに使う ggml モデル。標準(tiny/base/small/medium)と**日本語特化 kotoba-whisper**を選べる。

## 受入基準（EARS）
- **R1（ubiquitous）**: The system shall offer a catalog of selectable whisper models (標準＋kotoba-whisper)。
- **R2（state）**: While no model is chosen, the system shall use `base`（既定・後方互換）。
- **R3（event）**: When the user selects a model and records locally, the system shall download that model on first use and transcribe with it。
- **R4（unwanted）**: If a download fails mid-way, then the system shall not leave a corrupt model（`.part`→rename）。
- **R5（ubiquitous・プライバシー）**: Model selection shall apply only to the local engine（クラウド選択時は無関係）。

## BDD 例
```gherkin
Scenario: kotoba-whisperで日本語精度を上げる (R1,R3)
  Given 文字起こしエンジン=ローカル、モデル=「kotoba-whisper 量子化」
  When 初めて録音→停止する
  Then モデルが自動DLされ、その後はそのモデルで文字起こしされる
```

## テストリスト（実装済み・model.rs）
- [x] `model_for` 空/未知→base、既知idは解決
- [x] カタログにkotoba-whisper含む・id一意
- [x] `list_models` 先頭が既定base
- [ ] 実機: kotoba-whisper DL＆日本語文字起こし（CIは重DL不可＝手動）

## カタログ（一次確認済みURL）
- 標準: ggerganov/whisper.cpp の ggml-{tiny,base,small,medium}.bin
- 日本語: kotoba-tech/kotoba-whisper-v2.0-ggml の ggml-kotoba-whisper-v2.0(-q5_0).bin

## 範囲外（後続）
モデル削除UI・容量表示・モデル更新通知は後続。本増分は選択＋自動DL＋kotoba対応。
