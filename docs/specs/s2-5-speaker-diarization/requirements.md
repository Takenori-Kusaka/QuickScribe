# S2.5 話者特定（Speaker Diarization） — requirements

> Status: Draft (2026-07-10) / 対象: [ADR-0031](../../adr/0031-speaker-diarization-optional-utility.md)（default-OFF・利用幅拡大の実用ユーティリティ）。
> コア価値（ソロの自己理解）への寄与エビデンスは無いため**体験価値と謳わない**。会議・インタビュー等の複数話者録音への到達拡大が狙い。
> 前提調査: `docs/research/sources/speaker-diarization-local.md`（sherpa-onnx でローカル実装可）。

## ユビキタス言語

- **Diarizer**: 16kHz mono 波形から話者区間 `Vec<SpeakerSegment{ start_cs, end_cs, speaker }>` を返す戦略インターフェイス。実装は `SherpaDiarizer`（sherpa-onnx: pyannote segmentation ＋ 話者埋め込み ＋ クラスタリング）。
- **SpeakerSegment**: ある話者が連続発話した絶対時間区間（センチ秒）＋話者ID。
- **話者割当（whisperX方式）**: 各 whisper セグメント区間 `[t0,t1)` と各話者区間の**時間重複長を合算し argmax** で話者を1人選ぶ。重なりが無ければ最近傍（fill-nearest）。
- **話者名リネーム**: 文字起こし後、検出された話者（`話者1`…）に本人の名前を割り当て、本文中の `[話者N]` を全置換する工程。既存の一括置換 `applyCorrections`（`text.split(原文).join(提案)`）を再利用する。

## 受入基準（EARS）

- **R1（ubiquitous）**: The system shall expose speaker diarization as an **opt-in** transcription setting, **default OFF**（`sttDiarize`）。OFF 時は文字起こしの出力・処理・モデルDLが現行と一切変わらない（非退行）。
- **R2（event）**: When diarization is ON and local whisper is used, the system shall prefix each transcript segment with a speaker label（例 `[話者1] …`／タイムスタンプ併用時は `[HH:MM:SS][話者1] …`）。
- **R3（ubiquitous）**: The diarization pipeline shall run fully local（ネットワーク送信ゼロ）・Python非依存で、録音経路とファイル経路の双方に適用できる（共通の 16kHz mono 波形を入力）。
- **R4（event）**: When diarization models are absent, the system shall download them on first use with integrity verification（既存 `download_to`/`verify_integrity` を再利用）し、進捗を通知する。
- **R5（unwanted）**: If diarization initialization or inference fails, the system shall fall back to normal（話者ラベル無し）transcription without crashing（**abort させない**。sherpa/ONNX の FFI 失敗を境界で吸収）。
- **R6（ubiquitous）**: 話者数は既知なら固定、未知なら閾値で自動推定（`FastClusteringConfig`）。単一話者音声では実質1話者に収束する。
- **R7（ubiquitous）**: 純割当ロジック（whisperX 方式の重複 argmax）はエンジンから独立した純粋関数として実装し、単体テストで固定する。
- **R8（event）**: When a transcript contains speaker labels, the system shall offer a **話者名リネームUI** that lists detected speakers（`話者1`…）with an editable name field。空欄の話者は変更しない。
- **R9（event）**: When the user applies speaker names, the system shall all-replace `[話者N]` → `[入力名]` in the transcript via the existing `applyCorrections`（新規置換ロジックを作らない）。適用は本文編集＝取り消し可能な通常の編集として扱う。
- **R10（ubiquitous・非破壊）**: 話者名の入力有無に関わらず、リネームUIはOFF既定の diarization を有効化した時のみ出現し、話者ラベルが無い文字起こしには一切出ない（簡便さ）。

## テストリスト

- [ ] `assign_speakers(segments, speaker_turns)`（純関数）: 重複 argmax で正しい話者を割当（複数話者・境界・重なり無し=最近傍・空入力=全域性）
- [ ] `format_speaker_label` / ラベル前置: timestamps有無の両方で期待整形
- [ ] `sttDiarize` の既定は false（opt-in / settings-persist round-trip）
- [ ] `set_stt_settings` に diarize 引数を足しても既存呼出が回帰しない（全域性・既定OFF）
- [ ] i18n 4ロケール（ja/en/zh/es）に `settings.diarize`/`settings.tip_diarize` が揃う（catalog parity）
- [ ] OFF 時に文字起こし出力がバイト単位で現行と一致（非退行）
- [ ] `detectSpeakers(transcript)`（純関数）: 本文中の `[話者N]` を重複なく昇順抽出（ラベル無し=空）
- [ ] 話者名リネーム: 入力名で `[話者N]`→`[名前]` 全置換、空欄はスキップ（applyCorrections 経由・件数一致）
- [ ] リネームUIは話者ラベルが無い文字起こしには出ない（R10）
- [ ] e2e（実起動）: 複数話者音声で話者ラベルが付与され、単一話者では1話者へ収束

## 段階実装（ADR-0006 準拠：リスクは削らず分割で対処）

最終ゴールの機能集合（＝実バイナリで動く話者特定オプション）は不変。ネイティブ ONNX 結合のビルドリスクをフェーズで隔離する。

- **Phase 1（配線・割当・テスト）**: `Diarizer` trait ＋ `SpeakerSegment` 型、whisperX 割当の純関数、`sttDiarize` トグルの端から端まで（settings-persist → App.svelte → `set_stt_settings` → `SttConfig`）、i18n、出力ラベル整形、上記の純ロジック/配線テスト。**ネイティブ非依存でグリーン**。
- **Phase 2（sherpa-onnx バックエンド）**: `SherpaDiarizer` 実装（Cargo feature `diarization`）、segmentation＋embedding ONNX のカタログ＋`ensure_diarization_models`、`transcribe_with` への前処理挿入、FFI失敗の境界吸収（R5）、Windows ビルド/CI 結合。**先頭で `cargo` ビルドを fail-fast 検証**し、単一Vulkan配布との両立（ADR-0012）を確認してから既定ビルドへ組み込む。

## 技術検証（2026-07-10・Phase2の実機確認）

sherpa-onnx を実機で検証（scratchpad プローブ・4話者テスト音声 `0-four-speakers-zh.wav` 56.9s）:

- **ネイティブ結合は動作**: sherpa-rs 0.6.8 が Windows でビルド・リンク成功（onnxruntime.dll 他5個を動的リンク）。ONNXモデル2種をオンデマンドDL（推測したHF URLは実在＝200/206確認）し、話者分離推論が**貫通実行**（区間＋話者IDを出力）。
- **モデル精度は良好**: `num_clusters=4`（話者数既知）で**正確に4話者**を検出。
- **自動閾値の調整が必要**: `num_clusters=-1`（未知）の閾値スイープ = 0.4→8 / 0.5→7 / 0.7→5 / 0.9→4話者。**sherpa 既定 0.5 は過分割**。1人を複数話者に割る偽検出は用途上有害なため、既定を **0.8** に設定（`diarize.rs`）。
- **未検証（Phase2 継続）**: (1) 日本語実音声での最適閾値・分離精度（ADR-0031 留保）。(2) ネイティブDLLの**オンデマンド取得＋DLL検索パス設定＋遅延ロードの abort 安全性(R5)** を実配布経路で確認（プローブは build 時同梱DLLで検証したため、実行時取得経路は別途要検証）。(3) モデル/DLLの SHA256 ピン留め(R4)。

## 範囲外（後続 / 別ADR）

- 話者の**役割**同定（医師/患者など）。diarization は話者分離のみ。
- 話者名の**永続化/学習**（次回同一話者の自動命名・話者辞書）。今回は毎回手動リネーム。
- クラウドSTT経路での話者特定（本増分はローカル whisper のみ）。
- 感情/情動メタ（[ADR-0030](../../adr/0030-no-voice-emotion-metadata.md) で非対応）。
