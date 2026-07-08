# 文字起こし効率化の限界 調査（streaming vs チャンク化・高速化アルゴリズム・日本語特性）

> 対象: ローカルCPU（Win/Linux・単一バイナリ・ADR-0012）で日本語ボイスジャーナル（10〜30分の独り語り）を最も効率的・高速に文字起こしする方式と、その**性能の限界**。
> 方法: ADR-0007 問い設計 → deep research（論文・OSS一次エビデンス・3票敵対的検証。104エージェント・確定/棄却明示）。
> 実行日: 2026-07-08。関連: [turbo高速化調査](turbo-speedup-question-design.md)（180分=反復ループ暴走の主因確定）/ #600 / ADR-0025。

## 結論サマリ

1. **方式は「録音停止後のチャンク化バッチ」継続が最適（D1）**。ストリーミングは技術的に成立する（LocalAgreement/Simul-Whisper/AlignAtt、Simul-Whisperは1秒チャンクで平均WER劣化1.46%）が、**ベンチはGPU・英語/多言語中心で「日本語×ローカルCPU」の一次実測が存在しない**。日本語は同音異義（高速/拘束 等）の解消に後続文脈が要り、逐次確定は本質的リスク（3-0確定）。
2. **さらなる高速化の本命は「VADベース動的チャンク＋バッチ推論」（D2）**。WhisperX（Interspeech 2023）は VADカット&マージ(~30秒)＋バッチで **精度を落とさず約12倍**（TED-LIUM 11.8×・WER 10.52→9.70 改善）。現行 #600（固定24s＋オーバーラップ）の正当性を支持しつつ、次の伸び代を示す。
3. **CPU性能の限界（D3）**: 量子化はモデルが大きいほど効き、INT4 で large-v3 が CPU 2.2〜2.4×（TARQ, 2-1）。ただし **「large-v3 が CPU でリアルタイム(RTF<1)」という主張は 0-3 で棄却**＝CPU大モデルのリアルタイム化は未確立。faster-whisper(CTranslate2) の「4x」は Python openai/whisper 比で、**whisper.cpp は既に最適化済みのため相対利得は縮む**（公式ベンチ: whisper.cpp fp32 2m05s は faster-whisper 非バッチ 2m37s より速い。バッチ+int8 で 51s＝バッチ化が本命）。
4. **日本語ストリーミングの精度パリティは未達（D4）**: 日本語低レイテンシ認識は実現可能（APSIPA2020: CER 9.87%・240ms）だが Whisper 級（~5-7%）に劣る。同音異義の文脈依存は 3-0 確定。
5. **OSS/ライセンス（D5）**: whisper.cpp=MIT で ADR-0012 と最適合（現行継続）。faster-whisper(MIT)/sherpa-onnx(Apache) はライセンス両立だが **Rust 単一バイナリとの統合は別のパッケージング工数**。sherpa-onnx は日本語専用 Zipformer(ReazonSpeech 35k時間) を提供＝将来の代替候補（ニュアンス保持・CER要検証）。

## 達成可能な性能の限界（暫定・実測で更新）

- **現行アーキテクチャ（whisper.cpp turbo q5＋#600チャンク化）の律速はチャンクの逐次実行**。チャンクは独立なので**並列/バッチ化が理論上の次の伸び代**（WhisperX 方式で英語 ~12x の一次実績）。ただし whisper.cpp はチャンク内で既に全コアを使うため、**CPU ではチャンク並列の利得は限定的**（コア競合）＝バッチの実利得は GPU 前提の数字。CPU での現実的な限界は:
  - 反復暴走の解消（#600・済）＝ RTF を「素の turbo」水準（~1.15、実測要更新）に戻す
  - ＋VAD で無音スキップ（独り語りの沈黙分を丸ごと削減、比率は録音に依存）
  - ＋量子化の最適点（q5→int8/INT4 で 大モデル 2.2〜2.4×の一次値、日本語CER要検証）
  - → **CPU では「音声実時間の 0.5〜1.0 倍程度」が現実的な下限帯**（リアルタイム下回りは未確立）。それ以上（分オーダー）は **opt-in GPU（Vulkan）または ADR-0012 見直し（CTranslate2/ONNX系）が必要**。
- 「体感」の限界突破はアーキテクチャでなく **UX 側**（マルチジョブ逐次キュー #621 で「待たずに次を録れる」は達成済み。ストリーミング逐次表示は日本語精度リスクに見合わない）。

## 棄却された主張（採用しないこと）

- 「ストリーミング(WhisperPipe)がオフライン比 WER 2%以内で 3-5x 低レイテンシ」（0-3 棄却）
- 「INT4 で large-v3 が CPU リアルタイム RTF=0.91」（0-3 棄却）
- 「日本語ASRの訂正にドメイン固有知識が必須」（0-3 棄却。必要なのは後続文脈であり外部知識ではない）

## 未解決の問い（次の実測で埋める）

1. **現行 #600 設定での日本語 17分実録音のベースライン実測**（CER・所要時間・RTF）— 最優先。本セッションで実測中。
2. LocalAgreement/Simul-Whisper 系を日本語×CPU で回したときの CER 劣化と再訂正頻度。
3. sherpa-onnx 日本語 Zipformer の CER・句読点・言い淀み保持（ニュアンス保持の観点で turbo 比較）。ADR-0025 を覆すか。
4. whisper.cpp 単一バイナリのまま VADカット&マージ＋並列チャンクを載せる実装経路 vs ADR-0012 見直しの工数対効果。

## QuickScribe への実装含意（今すぐ効く順）

1. **#600 チャンク化（済）**: 180分暴走の主因対策（[turbo調査](turbo-speedup-question-design.md)）。turbo 実測で効果を定量化（実測値は本ファイル更新）。
2. **VAD 無音スキップ**: 独り語りの無音削減＝処理量と反復リスクの双方に効く。whisper.cpp `--vad`(Silero) は #569（whisper-rs/sys 更新）とセット。**次の実装本命**。
3. **チャンク境界の VAD 化**（固定24s→無音境界カット&マージ）: WhisperX 方式の単一バイナリ内適用。境界の単語分断・ハルシネーションをさらに抑制。
4. **量子化最適点の日本語検証**（q5 vs q8/int8）: ADR-0024 CERベンチに乗せて数値比較。
5. **opt-in GPU（Vulkan）**: 分オーダーを狙うなら。新ADR＋ビルド変種。
6. **エンジン代替（sherpa-onnx/CTranslate2）**: 最終手段。ADR-0012/0025 見直しが要る。

## 一次ソース（主要）

- LocalAgreement: Macháček et al. 2023 https://arxiv.org/abs/2307.14743
- Simul-Whisper (Interspeech 2024): https://arxiv.org/abs/2406.10052
- SimulStreaming/AlignAtt (IWSLT 2025): https://arxiv.org/abs/2506.17077
- WhisperX (Interspeech 2023, VADカット&マージ＋バッチ ~12x): https://arxiv.org/html/2303.00747v2
- faster-whisper 公式ベンチ（CPU 13分: whisper.cpp 2m05s / fw batched int8 51s）: https://github.com/SYSTRAN/faster-whisper
- INT4 量子化 (TARQ): https://arxiv.org/pdf/2605.27808
- 日本語 streaming (APSIPA 2020, CER 9.87%/240ms): Chen/Nishimura/Kitaoka
- 日本語同音異義とASR誤り: https://arxiv.org/html/2408.16180v2
- sherpa-onnx（日本語 Zipformer/ReazonSpeech）: https://github.com/k2-fsa/sherpa-onnx
