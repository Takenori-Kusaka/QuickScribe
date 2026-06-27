# クラウドSTTプロバイダ — 一次情報リサーチ（S2.4 #25）

> 調査日: 2026-06-27 / 問い設計: [s2-4-cloud-stt-question-design.md](../s2-4-cloud-stt-question-design.md)（[ADR-0007](../../adr/0007-research-question-framing-method.md)）
> 全クレーム一次情報URL付き。前提: 同期HTTP(ureq)、16k mono f32 → WAV(hound)送信。

## 結論（先出し）
**Groq と OpenAI は byte-for-byte 同一のmultipart形状**（`POST .../v1/audio/transcriptions`・`Authorization: Bearer`・フィールド`file`+`model`(+`language`,`response_format`)・本文テキストは`.text`）。**1つのコードパスで両対応**でき、差分は base_url と model id のみ。Deepgram(生バイト+クエリ)・Azure(multipart+JSON定義)は別パス。全社 WAV PCM16 16k mono を受付。全社 **既定でAPI音声を学習利用しない**。

## 比較表

| プロバイダ | エンドポイント | 認証 | 音声送信 | テキスト取得パス | WAV | サイズ上限 | 学習利用 |
|---|---|---|---|---|---|---|---|
| Groq | `POST api.groq.com/openai/v1/audio/transcriptions` | `Authorization: Bearer` | multipart `file`+`model` | `.text` | ✅ | 25MB(無料)/100MB | なし(ZDR可) |
| OpenAI | `POST api.openai.com/v1/audio/transcriptions` | `Authorization: Bearer` | multipart `file`+`model` | `.text` | ✅ | 25MB | なし(audio端点は無保持) |
| Deepgram | `POST api.deepgram.com/v1/listen?model=..&language=ja` | `Authorization: Token` | 生バイト本文+`Content-Type: audio/wav` | `results.channels[0].alternatives[0].transcript` | ✅ | 2GB | 既定なし(`mip_opt_out=true`) |
| Azure | `POST {res}.cognitiveservices.azure.com/speechtotext/transcriptions:transcribe?api-version=2025-10-15` | `Ocp-Apim-Subscription-Key` | multipart `audio`+`definition`(JSON `{"locales":["ja-JP"]}`) | `combinedPhrases[0].text` | ✅ | 500MB/5h | なし |

**料金概算(/時)**: Groq turbo $0.04 / Groq v3 $0.111 / OpenAI mini $0.18・std $0.36 / Azure fast $0.36 / Deepgram Nova-3 ~$0.46。

## 音声送信
- 16k mono f32 → **WAV PCM16 を hound で生成**（全社受付・最小）。16k mono16bit ≈ 1.9MB/分 → 25MB上限で約13分。長尺は将来Opus/分割（Phase外）。
- 日本語: Groq/OpenAI=`language=ja`、Deepgram=`?language=ja`、Azure=`definition.locales=["ja-JP"]`(BCP-47)。

## 同期/ストリーミング
- 全社「録音停止後に一括POST→JSON」で完結（ureq同期でOK）。ストリーミングは別端点で不要。

## プライバシー（正直な注意喚起用）
- 全社 **既定でAPI音声を学習利用しない**。Groq(ZDR)・OpenAI(audio端点 無保持)・Azure(fast transcription 無保持)が最強。Deepgramは既定ゼロ保持＋防御的に`mip_opt_out=true`。
- UIで「クラウドSTTは音声を端末外へ送信します。プロバイダXは学習利用しないと明言（リンク）」を明示。**既定はローカル whisper**（プライバシー差別化を維持）。

## 主要ソース
Groq: https://console.groq.com/docs/speech-to-text / https://console.groq.com/docs/your-data / https://groq.com/pricing
OpenAI: https://developers.openai.com/api/docs/guides/speech-to-text / https://developers.openai.com/api/docs/guides/your-data / https://developers.openai.com/api/docs/pricing
Deepgram: https://developers.deepgram.com/docs/pre-recorded-audio / https://developers.deepgram.com/docs/the-deepgram-model-improvement-partnership-program / https://deepgram.com/pricing
Azure: https://learn.microsoft.com/en-us/azure/ai-services/speech-service/fast-transcription-create / https://learn.microsoft.com/en-us/azure/foundry/responsible-ai/speech-service/speech-to-text/data-privacy-security / https://azure.microsoft.com/en-us/pricing/details/speech/
