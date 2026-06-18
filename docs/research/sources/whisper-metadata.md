# whisper-rs 0.14 / whisper.cpp メタデータ抽出 — 一次情報ソース

取得日: 2026-06-18
対象: whisper-rs 0.14 系（docs.rs は 0.14.4 を参照）/ whisper.cpp master。

## 参照URL一覧

- WhisperState API: https://docs.rs/whisper-rs/0.14.4/whisper_rs/struct.WhisperState.html
- FullParams API: https://docs.rs/whisper-rs/0.14.4/whisper_rs/struct.FullParams.html
- WhisperTokenData: https://docs.rs/whisper-rs/latest/whisper_rs/type.WhisperTokenData.html
- WhisperContextParameters: https://docs.rs/whisper-rs/0.14.4/whisper_rs/struct.WhisperContextParameters.html
- DtwParameters: https://docs.rs/whisper-rs/0.14.4/whisper_rs/struct.DtwParameters.html
- DtwModelPreset: https://docs.rs/whisper-rs/0.14.4/whisper_rs/enum.DtwModelPreset.html
- SegmentCallbackData: https://docs.rs/whisper-rs/0.14.4/whisper_rs/struct.SegmentCallbackData.html
- whisper.cpp README (tinydiarize): https://github.com/ggerganov/whisper.cpp (README master)
- whisper.cpp whisper.h: https://github.com/ggml-org/whisper.cpp/blob/master/include/whisper.h
- 時間単位（centiseconds）議論: https://github.com/ggml-org/whisper.cpp/issues/3370 / https://github.com/ggml-org/whisper.cpp/discussions/1584

## 引用（一次情報）

### A. セグメントタイムスタンプ（WhisperState）
```
pub fn full_n_segments(&self) -> Result<c_int, WhisperError>
pub fn full_get_segment_t0(&self, segment: c_int) -> Result<i64, WhisperError>
pub fn full_get_segment_t1(&self, segment: c_int) -> Result<i64, WhisperError>
pub fn full_get_segment_text(&self, segment: c_int) -> Result<String, WhisperError>
```
単位: whisper-rs の doc には明記なし（未確認点）。whisper.h の `whisper_full_get_segment_t0`
にも単位コメントなし。ただし whisper.cpp 慣行および issue #3370 で「返り値はセンチ秒
(1/100 秒)。ms にするには ×10」と確認。t0=100 → 1.00 秒。

### B. トークン単位タイムスタンプ（FullParams + WhisperState + WhisperTokenData）
FullParams（いずれも [EXPERIMENTAL]）:
```
pub fn set_token_timestamps(&mut self, token_timestamps: bool)  // default false
pub fn set_split_on_word(&mut self, split_on_word: bool)        // default false
pub fn set_max_len(&mut self, max_len: c_int)                   // default 0
pub fn set_max_tokens(&mut self, max_tokens: c_int)            // default 0 (=無制限)
```
WhisperState:
```
pub fn full_n_tokens(&self, segment: c_int) -> Result<c_int, WhisperError>
pub fn full_get_token_data(&self, segment, token) -> Result<WhisperTokenData, WhisperError>
pub fn full_get_token_text(&self, segment, token) -> Result<String, WhisperError>
pub fn full_get_token_prob(&self, segment, token) -> Result<f32, WhisperError>
```
WhisperTokenData フィールド: id:i32, tid:i32, p:f32, plog:f32, pt:f32, ptsum:f32,
t0:i64, t1:i64, t_dtw:i64, vlen:f32。
- p = トークン確信度（probability）。t0/t1 = トークン時刻（センチ秒、token_timestamps 有効時）。
- t_dtw = DTW 由来時刻（DTW 有効時のみ）。

DTW（より正確な単語境界）は **context params 側**で有効化:
```
WhisperContextParameters::dtw_parameters(&mut self, DtwParameters<'a>) -> &mut Self
pub struct DtwParameters<'a> { pub mode: DtwMode<'a>, pub dtw_mem_size: usize }
DtwModelPreset 変種: TinyEn, Tiny, BaseEn, Base, SmallEn, Small, MediumEn, Medium,
  LargeV1, LargeV2, LargeV3, LargeV3Turbo
```
注意（docs 引用）: "DTW will be disabled if flash_attn is true"（flash_attn と排他）。
未確認点: DtwMode の正確な変種名（ModelPreset(DtwModelPreset)/Custom 等）。
→ 確認方法: docs.rs の enum.DtwMode.html / src の dtw.rs を確認。

### C. 話者ダイアライゼーション（tinydiarize）
FullParams: `pub fn set_tdrz_enable(&mut self, tdrz_enable: bool)` // [EXPERIMENTAL][TDRZ] default false
WhisperState: `pub fn full_get_segment_speaker_turn_next(&mut self, i_segment: c_int) -> bool`
whisper.cpp README 引用:
- モデル: `./models/download-ggml-model.sh small.en-tdrz` → `ggml-small.en-tdrz.bin`（**英語専用**）
- 実行: `whisper-cli -f a13.wav -m ggml-small.en-tdrz.bin -tdrz`
- 出力に `[SPEAKER_TURN]` マーカー（話者交代点のみ。誰が話したかの ID は付かない）
結論: whisper 単体での **複数話者の識別（誰が話したか）は不可**。tinydiarize は
「話者が交代した境界」を出すだけ。日本語は tdrz モデルが英語専用のため **不可**。

### D. その他メタデータ
- 言語自動判定: WhisperState `full_lang_id_from_state(&self) -> c_int`,
  `lang_detect(&self, offset_ms, threads) -> Result<(i32, Vec<f32>), WhisperError>`
  FullParams `set_detect_language(bool)` / `set_language(None or "auto")`
- no_speech 確率: whisper.h には `whisper_full_get_segment_no_speech_prob` が存在するが、
  **whisper-rs 0.14 の WhisperState には未露出**（メソッドなし）。未確認点: 0.14 系の最新
  パッチ/0.16 で追加されたか。→ 確認方法: docs.rs 最新 + GitHub src/whisper_state.rs grep。
- `set_no_speech_thold(f32)` は存在するが doc に "Currently not implemented" の注記。

### E. 速度トレードオフ（定性。一次の定量値は未確認）
- token_timestamps: 追加コストは比較的小（後処理寄り）。
- DTW (dtw_token_timestamps): クロスアテンション保持＋整列で **追加計算・メモリ増**。
  flash_attn と排他。プリセット指定でモデル一致が必須。
- tinydiarize: 専用モデル（small.en）使用そのものの速度。tdrz 判定自体の追加コストは小。
未確認点: 公式の定量ベンチは README に無し。→ 自前計測（同一音声で on/off 比較）が確実。
