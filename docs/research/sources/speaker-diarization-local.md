# 話者分離（Speaker Diarization）をローカルRustで — 一次情報ソース

調査日: 2026-06-18
問い: QuickScribe（Tauri/Rust、ローカル完結、whisper.cpp/whisper-rs）で、Python常駐や重い実行時依存なしに、**日本語で機能する話者分離**を実装できるか。前回の「whisper単体では不可・日本語不可」結論を更新する。

---

## A. 話者分離の標準パイプライン

一般的な diarization は **VAD/セグメンテーション → 話者埋め込み(speaker embedding) → クラスタリング**。
pyannote.audio がこれを行う。sherpa-onnx はこれをPython無し・ONNX/ネイティブで再現する。

---

## B. ONNX/ネイティブで動くRust実装（最重要）

### sherpa-onnx（k2-fsa）本体
- URL: https://github.com/k2-fsa/sherpa-onnx (取得 2026-06-18)
- ライセンス: **Apache-2.0**（README フッタ／バッジで確認）
- offline speaker diarization を正式サポート（README 機能表にチェック）
- onnxruntime ベース、**Win/Linux 両対応**、インターネット接続不要（ローカル完結）
- C API / 12言語バインディング（Rust含む）

### 公式 Rust crate `sherpa-onnx`（★ sherpa-rs の後継・推奨）
- URL: https://docs.rs/sherpa-onnx (取得 2026-06-18)
- "Safe Rust wrapper for sherpa-onnx"。**Version 1.13.3 / License Apache-2.0**
- Author: csukuangfj（k2-fsa 中核メンテナ）/ repo: github.com/k2-fsa/sherpa-onnx
- diarization 型を提供:
  - `OfflineSpeakerDiarization`（主ダイアライザ）
  - `OfflineSpeakerDiarizationConfig` / `...Result` / `...Segment`
  - `OfflineSpeakerSegmentationModelConfig` / `OfflineSpeakerSegmentationPyannoteModelConfig`
  - `SpeakerEmbeddingExtractor` / `SpeakerEmbeddingManager`
- **C ライブラリを既定で静的リンク**、ビルド時にプリビルドアーカイブを自動DL（`SHERPA_ONNX_LIB_DIR` で上書き可）→ Python常駐不要、配布が軽い

### sherpa-rs（コミュニティ製・現在は非推奨）
- URL: https://github.com/thewh1teagle/sherpa-rs (取得 2026-06-18)
- ライセンス MIT、最新 v0.6.8（2025-10-05）
- **2026-06-06 にアーカイブ（read-only）。メンテナは公式 sherpa-onnx Rust バインディング使用を推奨**
- diarize 例: examples/diarize.rs（下記のモデル構成を示す）

### モデル構成（公式 Python/Rust 例の既定値）
- segmentation: `sherpa-onnx-pyannote-segmentation-3-0/model.onnx`
- embedding: `3dspeaker_speech_eres2net_base_sv_zh-cn_3dspeaker_16k.onnx`
- ソース: https://raw.githubusercontent.com/k2-fsa/sherpa-onnx/master/python-api-examples/offline-speaker-diarization.py (取得 2026-06-18)
- ソース: https://raw.githubusercontent.com/thewh1teagle/sherpa-rs/main/examples/diarize.rs (取得 2026-06-18)

### 話者数の指定（FastClusteringConfig）— 既知／未知の両対応を確認
公式 Python 例のコメントを引用（取得 2026-06-18）:
- `num_clusters`: "If you know the actual number of speakers... specify it. Otherwise, leave it to -1"
- `threshold`: "If num_clusters is -1, then this threshold is used for clustering. A smaller cluster_threshold → more speakers; larger → fewer."
→ **話者数既知なら固定、未知なら閾値で自動推定**の両方が可能。

### pyannote セグメンテーションモデル
- gated 版: https://huggingface.co/pyannote/segmentation-3.0 (取得 2026-06-18)
  - **ライセンス MIT**（商用利用・再配布可、"always remain open-source"）
  - **gated**: 条件承諾＋連絡先共有が必要（"occasionally email about premium models"）。ただしモデル自体はオープン。
  - 入力: 10秒・mono・16kHz、出力7クラス（非発話／3話者／2話者重複3通り）= VAD・重複検出・diarizationの部品
- **非gatedのONNX版が存在**: https://huggingface.co/onnx-community/pyannote-segmentation-3.0 (取得 2026-06-18)
  - ONNX 重み同梱（Transformers.js 互換）。sherpa-onnx も独自にONNX化済みモデルを配布。
- 別のONNX化実装: https://github.com/pengzhendong/pyannote-onnx (取得 2026-06-18)

---

## C. Whisper との統合（whisperX 方式）
- ソース: https://github.com/m-bain/whisperX/blob/main/whisperx/diarize.py (取得 2026-06-18)
- `assign_word_speakers()`: 各単語/セグメントを**時間的オーバーラップ（intersection）が最大の話者**に割当（IoUではなく重複時間の合算で argmax）。
  - `intersection = min(end_turn, end_word) - max(start_turn, start_word)`、話者ごとに合算し最大話者を採用。
  - 重なり無し時は `fill_nearest=True` のとき最近傍話者を割当（既定Falseでは未割当）。
- Rust再実装: 各 whisper セグメント区間 vs 各 diarization 話者区間で `max(0, min(e1,e2)-max(s1,s2))` を話者キーで合算し argmax。線形走査で十分（IntervalTreeは性能最適化のみ）。

## D/E. 精度・現実性・推奨
→ 本文回答に記載。

---

## 埋め込みモデルのライセンス（商用バンドル可否）
- **3D-Speaker (modelscope/3D-Speaker)**: コードは Apache-2.0（https://github.com/modelscope/3D-Speaker/blob/main/LICENSE 取得 2026-06-18）。⚠️ **重みのライセンスは別物の可能性** — ModelScope モデルカード（`iic/speech_eres2net_..._zh-cn_3dspeaker_16k`）のライセンス欄を要確認。sherpa-onnx 再配布は商用可の傍証。
- **WeSpeaker (wenet-e2e/wespeaker)**: コード Apache-2.0。ただし **VoxCeleb 学習済みモデルは CC BY 4.0**（帰属必須）。
- **NeMo TitaNet (nvidia/speakerverification_en_titanet_large)**: **CC-BY-4.0**（商用可・帰属必須）。https://huggingface.co/nvidia/speakerverification_en_titanet_large (取得 2026-06-18)
- いずれもバンドル・商用配布可。CC-BY系・Apache系とも **帰属表示/NOTICE保持義務**あり。

## クラスタリング設定（C++ ヘッダ一次確認）
- ソース: https://github.com/k2-fsa/sherpa-onnx/blob/master/sherpa-onnx/csrc/fast-clustering-config.h (取得 2026-06-18)
- `num_clusters > 0` → 固定話者数（threshold無視、話者数既知なら推奨）。`num_clusters <= 0` → threshold で自動話者数検出。

## 話者埋め込みの言語非依存性（日本語適用）
- 埋め込み(ERes2Net/ECAPA/x-vector)は原理的に**概ね言語非依存**。zh-cn 学習モデルの他言語転移は正の汎化を示す（https://arxiv.org/pdf/1908.01447）。
- ただし**完全な言語不変ではない**: クロスリンガルでスコアシフト（劣化）あり、言語依存正規化等で補償（https://arxiv.org/pdf/2110.09150）。
- 実務: zh-cn ERes2Net を日本語ジャーナルの diarization に**使用可能**だが、精度劣化の可能性 → 閾値調整・日本語実測が必要。

## 未確認・要確認点（更新）
1. 3D-Speaker ONNX 重みの ModelScope モデルカード ライセンス欄（コードと別の可能性）。
2. WeSpeaker 事前学習モデルカードのライセンス。
3. sherpa-onnx `c-api.h` の `SherpaOnnxFastClusteringConfig` フィールド名（Rust FFI 時）— ただし公式 `sherpa-onnx` crate を使えば FFI 自作不要。
4. 日本語データでの zh-cn ERes2Net 実測精度（EER/閾値）。

---

## 未確認・要確認点
- embedding モデル `3dspeaker_..._zh-cn` は中国語データ学習。**話者埋め込みの言語非依存性（日本語適用可否）**は別途検証中（サブエージェント）。一般に話者埋め込み(x-vector/ECAPA/ERes2Net)は音響的話者特徴を捉え言語非依存とされるが、一次情報での裏取りが必要。
- 3D-Speaker / WeSpeaker / NeMo titanet 各モデルの個別ライセンス（商用同梱可否）。
- speaker-recognition-models リリースページ(GitHub)は動的描画で未取得。release tag は綴り "speaker-recongition-models"（タイポ注意）。
