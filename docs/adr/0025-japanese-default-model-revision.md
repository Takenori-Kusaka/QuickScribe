# ADR-0025: 日本語の既定STTモデルを kotoba-whisper から large-v3-turbo へ変更（実測に基づく）

- Status: Accepted（Deciders 承認 2026-07-07）
- Date: 2026-07-07
- Deciders: Takenori Kusaka
- 改訂対象: [ADR-0021 ローカルファースト既定](0021-*.md)（日本語STT既定＝kotoba-q5 を改訂）/ [ADR-0022 モデルカタログ精選](0022-model-catalog-curation.md)（カタログ順・ラベルを更新）
- 関連: [ADR-0024 評価基盤の再設計](0024-evaluation-redesign-cer-and-nuance.md) / #600（長尺末尾欠落）/ #598（速度クラス）
- 一次情報: 本ADR末尾（3本の並列deep-research＋実録音での実測タイムスタンプ/RTF）

## 背景・課題

日本語の既定STTは kotoba-whisper v2.0 q5（ADR-0021）だった。しかし #600（長尺末尾欠落）の調査で、**「文字起こしが壊れれば整形の知性を含む全価値がゼロになる」**という本プロダクトの土台に対し、kotoba 既定が不適である可能性が浮上した。3本の並列deep-research（kotoba保守性 / Whisper本体 / 日本語ローカルSTT地勢）＋**利用者の実録音（11分40秒・自発的な一人語り）での実測**で検証した。

### 実測（利用者の実録音・音声長 11:40・no_context=true・タイムスタンプ確認）

| モデル | 末尾セグメントTS（実発話） | 末尾欠落 | RTF（当該CPU） | 会話CER（Neosophie 2026-05） | サイズ(q5) |
|---|---|---|---|---|---|
| base | [00:11:39] | なし | 0.13（最速） | 中 | 142MB |
| **kotoba-q5（旧既定）** | [00:11:16] | **約24秒欠落** | 0.90 | **0.495（崩壊）** | 538MB |
| **large-v3-turbo** | [00:11:34]（実文末まで） | 実質なし | 1.15（低速） | 0.184（良） | 547MB |

### 判明した事実（一次情報）
1. **kotoba は停滞**: コア重みは 2024-09（v2.0）が最後、全リポジトリ 2024-10 以降更新なし（約1年9か月）。運営 Kotoba 社は商用同時通訳/API/TTS へピボットし、OSS重みの改良ロードマップ無し。
2. **kotoba は用途とミスマッチ**: クリーン朗読では large-v3 と僅差だが、**自発発話・会話では崩壊（CER 0.495）**。本プロダクトの用途＝自発的な一人語り＝会話寄りで、まさに弱点。優位なのは自前学習の TV 音声(ReazonSpeech)ドメインのみ。
3. **kotoba は長尺が構造的に脆い**: distil の2層デコーダ・約7秒学習で 15秒超のタイムスタンプ予測が弱い。公式自身が「whisper.cpp/sequential は chunked より劣る」と明記。実録音でも末尾24秒を欠落。
4. **large-v3-turbo**: OpenAI 一次配布・ggml量子化入手可・会話に強く長尺の末尾も実質確実。欠点は低速（RTF 1.15＝当該CPUで11分音声に約13.5分）。
5. Whisper本体も turbo(2024-10)で開放重みは凍結＝「更新継続」を選定理由にはできない。選ぶ根拠は**長尺頑健性・会話精度・一次配布の透明性**。

## 決定

1. **日本語の既定STTモデルを kotoba-q5 → large-v3-turbo(q5) へ変更**。会話精度と長尺末尾の確実性を優先（コア価値＝文字起こしが壊れない）。
2. ~~**kotoba-q5 はカタログに残すが降格**（静音・朗読 in-domain 向けの選択肢）。~~ → **取消（[ADR-0029](0029-simplify-offering-drop-cuda-and-kotoba.md)）**。コア用途＝自発発話で崩壊し、朗読ニッチは本プロダクトにほぼ非該当、開発元も停止のため、kotoba（q5/フル）をカタログから撤去した。
3. **base は速く頑健なフォールバック**（末尾欠落なし・最速）。低スペック機や turbo が遅すぎる場合の推奨。
4. **選定は data-driven に**: 今後の既定判断は ADR-0024 の CER ベンチ（Common Voice/FLEURS＋会話サンプル）＋対象端末の実測 RTF で行う。turbo をベンチ対象に追加する。
5. **長尺の末尾欠落はモデルに依らずチャンク化/VADで根治**（#600）。turbo でも 30秒窓の残存リスクがあるため、既定変更とは別に継続。組み込みVADは whisper-rs/sys 更新(#569級)が要るため当面はアプリ側チャンク化。

## 結果・トレードオフ

- **Pro**: 用途（会話・長尺）での文字起こし品質が実測で改善。既定が「末尾を落とさない」モデルになる。停滞した単一小組織依存(kotoba)から、一次配布(OpenAI)＋透明な素性へ。
- **Con/リスク**: turbo は**低速**（RTF~1.15）。当該CPUで11分→約13.5分。低スペック機では体感が悪化 → base への案内が要る。DLサイズ 547MB。長尺の残存リスクは #600 のチャンク化まで残る。
- **却下案**:
  - **base を既定**: 速く末尾も確実だが精度中。コア価値（品質最優先）で turbo を採用。base は高速オプションに。
  - **kotoba 継続**: 用途で崩壊が実測確認され不可。
  - **full large-v3 を既定**: 長尺最強だが 2.9GB と重く、turbo で用途は足りる。
  - **ReazonSpeech nemo-v2**: 長尺最強だが PyTorch/NeMo 巨大依存でローカル配布非現実的（ローカル完結の思想と相反）。

## 「考えが変わる条件」（反証）

- CER ベンチ（自分のコーパス＋会話サンプル）で turbo が kotoba/base に対し優位を示さない → 既定を再考。
- 典型的な利用端末で turbo の RTF が許容外（体感を著しく損なう）と判明 → base を既定に、turbo を高精度オプションに。
- kotoba が更新を再開し会話・長尺の弱点を克服 → 再評価。
- **ReazonSpeech k2-v2 + sherpa-onnx**（Python不要C++・非Whisper）のスパイクで、幻覚特性の異なる軽量な代替が有望と分かれば選択肢に追加。

## 一次情報

- kotoba: [HF v2.0](https://huggingface.co/kotoba-tech/kotoba-whisper-v2.0)（CER表・distil・長尺推奨）/ [v2.0-ggml](https://huggingface.co/kotoba-tech/kotoba-whisper-v2.0-ggml)（whisper.cpp相性・「chunkedがsequentialより良い」）/ [GitHub commits 最終2024-10-23](https://github.com/kotoba-tech/kotoba-whisper/commits/main) / [Kotoba社 Seed2調達・商用ピボット 2025-08](https://www.kotoba.tech/en/news-all/we-have-raised-usd-1183-million-jpy-17-billion-in-a-seed-2-round-to-accelerate-commercialization)
- Whisper: [large-v3-turbo release 2024-10](https://github.com/openai/whisper/discussions/2363) / [whisper.cpp models(ggmlサイズ)](https://github.com/ggml-org/whisper.cpp/blob/master/models/README.md) / [反復幻覚 issue #3744](https://github.com/ggml-org/whisper.cpp/issues/3744)
- ベンチ: [Neosophie 日本語ASRベンチ 2026-05（会話CER: turbo 0.184 / kotoba 0.495）](https://neosophie.com/en/blog/20260226-japanese-asr-benchmark)
- 代替: [reazonspeech-nemo-v2](https://huggingface.co/reazon-research/reazonspeech-nemo-v2) / [reazonspeech-k2-v2](https://huggingface.co/reazon-research/reazonspeech-k2-v2)
- 実測: 利用者実録音 rec-20260705-171253.opus（11:40）のモデル別タイムスタンプ到達点・RTF（本ADR冒頭の表）
