# ADR-0016: クラウドSTTプロバイダ（Groq/OpenAI/Deepgram/Azure・段階実装）

- Status: Accepted（Phase A=Groq/OpenAI, Phase B=Deepgram/Azure 実装済み。長尺Opus/分割は後続）
- Date: 2026-06-27
- Deciders: Takenori Kusaka
- 関連: [ADR-0002 STT決定], [ADR-0005 BYO], [ADR-0006 スコープ規律], [ADR-0011 AWSプロバイダ(同型のBYO/同期HTTP方針)], [S2.3 TranscriptionEngine抽象](../specs/s2-3-transcription-engine/requirements.md)
- 一次情報: [docs/research/sources/cloud-stt-providers.md](../research/sources/cloud-stt-providers.md)
- 対象 Issue: #25（Epic E2）。

## 背景・課題
ローカル whisper は遅い/精度限界の場面がある。S2.3 の `TranscriptionEngine` 抽象に、各社クラウドSTTを `CloudEngine` として差し込む。**精度はコモディティ＝差別化にしない**（CLAUDE.md）が、選択肢として提供。音声を端末外へ送るため**プライバシー設計が要点**。

## 決定
1. **既定はローカル whisper（プライバシー優先）**。クラウドは設定で**明示選択した時のみ**有効。**BYO鍵**（keyring保管・S3.2同様）。UIに「音声を端末外へ送信」「各社は既定で学習利用しない（出典リンク）」を明示。
2. **送信形式は WAV PCM16 16k mono**（hound生成）。全社受付・最小実装。
3. **同期HTTP(ureq)で完結**（録音停止後に一括POST→JSON）。tokio不要（ADR-0011同方針）。
4. **段階実装（ADR-0006: 最終スコープ不変）**:
   - **Phase A（本ADRで実装）**: **OpenAI互換パス（Groq＋OpenAI）**。同一multipart形状で1コードパス、base_url＋model差分のみ。Groqは最安・高速、OpenAIは信頼ブランド。
   - **Phase B（後続）**: **Deepgram**（生バイト+クエリ・`Token`認証・ネストJSON）＋ **Azure**（multipart+`definition`JSON・リージョンhost・`Ocp-Apim-Subscription-Key`）。
5. クラウドは**逐次セグメント通知を持たない** → 進捗は0→100、本文は一括。UIは「クラウドで文字起こし中…」表示。

## 段階の理由
Groq/OpenAIが同形状で限界費用ゼロに近く即2社対応。Deepgram/Azureは別パス＋設定負担（特にAzureのリージョンhost）が大きいため分離してレビュー可能性を保つ。**スコープは削らず順序付け**。

## 結果・トレードオフ
- **Pro**: ローカル既定でプライバシー維持しつつ、必要時に高速/高精度を選べる。公式最小APIで独自実装回避。
- **Con/リスク**: 25MB上限(Groq/OpenAI)で約13分以上の録音はPhase外の分割/Opusが要る → `log`/UIで上限を明示。日本語精度はモデル依存（要実機評価・S2.5）。

## 「考えが変わる条件」
- 日本語精度がGroq turboで不足 → モデル既定をOpenAI gpt-4o-transcribe等へ。
- 長尺録音が常態化 → Opusエンコード/分割をPhase Bに前倒し（Deepgram 2GB/Azure 500MBが有利）。
- 各社のデータ方針が変化（point-in-time 2026-06）→ リリース毎に出典再確認。
