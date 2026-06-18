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
→ サブエージェント調査結果を反映予定（whisperX の overlap-max 割当アルゴリズム）。

## D/E. 精度・現実性・推奨
→ 本文回答に記載。

---

## 未確認・要確認点
- embedding モデル `3dspeaker_..._zh-cn` は中国語データ学習。**話者埋め込みの言語非依存性（日本語適用可否）**は別途検証中（サブエージェント）。一般に話者埋め込み(x-vector/ECAPA/ERes2Net)は音響的話者特徴を捉え言語非依存とされるが、一次情報での裏取りが必要。
- 3D-Speaker / WeSpeaker / NeMo titanet 各モデルの個別ライセンス（商用同梱可否）。
- speaker-recognition-models リリースページ(GitHub)は動的描画で未取得。release tag は綴り "speaker-recongition-models"（タイポ注意）。
