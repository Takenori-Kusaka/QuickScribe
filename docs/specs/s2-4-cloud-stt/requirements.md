# S2.4 クラウドSTT（Phase A: Groq/OpenAI） — requirements

> Status: Draft (2026-06-27) / 対象 Issue: #25（Epic E2）/ [ADR-0016](../../adr/0016-cloud-stt-providers.md)
> 一次情報: [cloud-stt-providers.md](../../research/sources/cloud-stt-providers.md)。記法: 軽量BDD＋EARS。

## ユビキタス言語
- **クラウドSTT**: 音声を端末外のAPIへ送って文字起こしする選択肢。既定はローカル whisper（プライバシー）。
- **OpenAI互換エンジン**: Groq/OpenAI を同一multipart形状で扱う `OpenAiCompatibleSttEngine`。

## 受入基準（EARS）
- **R1（state・既定）**: While the STT provider is unset/local, the system shall transcribe with local whisper（音声は端末外に出ない）。
- **R2（event）**: When the user selects Groq/OpenAI and provides an API key, the system shall send WAV(16k mono PCM16) to that provider and use the `.text` response.
- **R3（unwanted）**: If the API key is empty, then the system shall return a clear error（送信しない）。
- **R4（ubiquitous・コスト回避）**: While using a cloud provider, the system shall not download the local whisper model（不要なDLをしない）。
- **R5（ubiquitous・透明性）**: The settings UI shall warn that cloud STT sends audio off-device and link/мention provider data policy.
- **R6（unwanted）**: If the provider returns an HTTP error, then the system shall surface the status and a short detail.

## BDD 例
```gherkin
Scenario: ローカル既定 (R1)
  Given STTプロバイダ未設定
  When 録音→停止
  Then ローカルwhisperで文字起こしされ、音声は外部送信されない

Scenario: Groqで文字起こし (R2,R4)
  Given Groqを選びAPIキーを入力
  When 録音→停止
  Then 16k mono WAVがGroqへ送られ .text が文字起こし結果になる（whisperモデルDLは発生しない）
```

## テストリスト（実装済み・stt.rs）
- [x] `engine_for` 全域性（local/groq/openai/未知）
- [x] `is_cloud_provider` 分類（実装済みのみ true、未実装はfalse=ローカル扱い）
- [x] `encode_wav_16k_mono` が RIFF/WAVE ヘッダ＋PCM16 長さ
- [ ] 実機: 実キーでGroq/OpenAIに送って日本語が返る（CIでは鍵なし＝不可、手動）

## Phase B（実装済み）: Deepgram / Azure
- Deepgram: `POST v1/listen?model=nova-3&language=ja&smart_format=true&mip_opt_out=true`・`Token`認証・生WAV本文・`results.channels[0].alternatives[0].transcript`。
- Azure: `transcribe?api-version=2025-10-15`・`Ocp-Apim-Subscription-Key`・multipart(audio＋definition `{"locales":["ja-JP"]}`)・`combinedPhrases[0].text`。リソース名を設定UIで入力。
- 共通: `read_json_response` でHTTPエラーを状態＋短い詳細にして surface（R6）。

## 範囲（ADR-0006: 段階）
- Phase A: Groq + OpenAI（OpenAI互換1パス）。Phase B: Deepgram + Azure。**S2.4の主要スコープ完了。**
- 長尺(>~13分=25MB超 ※Groq/OpenAI)のOpus/分割は後続（Deepgram 2GB/Azure 500MBは余裕）。
