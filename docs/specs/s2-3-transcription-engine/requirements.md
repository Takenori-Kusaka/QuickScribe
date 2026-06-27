# S2.3 TranscriptionEngine 抽象（Strategy） — requirements

> Status: Draft (2026-06-27) / 対象 Issue: #24（Epic E2）。
> 整形側 `FormattingEngine`(S3.1) と同型の DIP 境界を文字起こし側にも作る。S2.4(クラウドSTT)の前提。

## ユビキタス言語
- **TranscriptionEngine**: 文字起こしの戦略インターフェイス。`LocalWhisperEngine`（既定）と将来のクラウド実装を差し替え可能にする。

## 受入基準（EARS）
- **R1（ubiquitous）**: The system shall transcribe audio through a `TranscriptionEngine` trait（呼び出し側は具体実装に依存しない）。
- **R2（ubiquitous）**: The system shall provide a `LocalWhisperEngine` wrapping the existing whisper.cpp path（**挙動は現行と不変・回帰なし**）。
- **R3（event）**: When resolving an STT provider, the system shall return an engine（未知/空はローカルへフォールバック）。
- **R4（ubiquitous）**: The engine shall report progress(0-100) and finalized segments via owned callbacks（whisperのSend+'static制約に適合）。

## テストリスト
- [x] `engine_for` が任意のプロバイダ名でパニックせずエンジンを返す（全域性）
- [x] 既存の `transcribe`/`resample`/`format_timestamp` テストが維持（回帰なし）
- [x] e2e（実起動）で文字起こし経路が通る

## 範囲外（後続）
クラウドSTT実装（Groq/Deepgram/Azure）= S2.4。本増分は抽象の導入とローカル実装のみ。
