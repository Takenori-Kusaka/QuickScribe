// 話者特定（Speaker Diarization / S2.5・ADR-0031）。
// 純ロジック（話者区間→whisperセグメントへの割当・話者ラベル整形）は feature 非依存で常にビルド・
// テストされる。実推論の `SherpaDiarizer` は `diarization` feature 配下（sherpa-onnx / ONNX ランタイム）。
// 配布はオンデマンド: ネイティブDLL・ONNXモデルは有効時のみ初回DL（ADR-0012 単一バイナリを維持）。

use std::collections::HashMap;

/// ある話者が連続発話した絶対時間区間（センチ秒・whisper のタイムスタンプ単位に合わせる）＋話者ID。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpeakerSegment {
    pub start_cs: i64,
    pub end_cs: i64,
    /// 話者ID（0始まり。表示は `speaker_label` で1始まりへ）。
    pub speaker: i32,
}

/// whisper セグメント区間 `[seg_start_cs, seg_end_cs)` に最も重なる話者を返す（whisperX 方式）。
/// 各話者との重複長を合算し argmax。重なりが無ければ中点最近傍（fill-nearest）。
/// 話者区間が空なら None（＝ラベル無し）。同点は小さい話者IDを優先（決定的）。
pub fn assign_speaker(seg_start_cs: i64, seg_end_cs: i64, turns: &[SpeakerSegment]) -> Option<i32> {
    if turns.is_empty() {
        return None;
    }
    let mut overlap: HashMap<i32, i64> = HashMap::new();
    for t in turns {
        let ov = (seg_end_cs.min(t.end_cs) - seg_start_cs.max(t.start_cs)).max(0);
        if ov > 0 {
            *overlap.entry(t.speaker).or_insert(0) += ov;
        }
    }
    if !overlap.is_empty() {
        // 重複合計の最大。同点なら speaker id が小さい方（b.0 の降順比較で小 id を勝たせる）。
        return overlap
            .into_iter()
            .max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(&a.0)))
            .map(|(spk, _)| spk);
    }
    // 重なり無し → セグメント中点に最も近い話者区間へ割当。
    let mid = (seg_start_cs + seg_end_cs) / 2;
    turns
        .iter()
        .min_by_key(|t| ((t.start_cs + t.end_cs) / 2 - mid).abs())
        .map(|t| t.speaker)
}

/// 話者IDを表示ラベルへ（0始まり→`[話者1]`）。バックエンドが出力する唯一の話者ラベル整形点。
pub fn speaker_label(speaker: i32) -> String {
    format!("[話者{}]", speaker + 1)
}

/// 16kHz mono 波形から話者区間を得る戦略（DIP 境界）。実装は `diarization` feature の `SherpaDiarizer`。
pub trait Diarizer {
    /// 話者区間（センチ秒）を返す。失敗は Err（呼び出し側はラベル無し文字起こしへフォールバック / R5）。
    fn diarize(&self, audio: &[f32], sr: u32) -> Result<Vec<SpeakerSegment>, String>;
}

/// sherpa-onnx による話者特定（`diarization` feature 時のみ）。pyannote segmentation ＋ 話者埋め込み
/// ＋ クラスタリング。モデル2種（segmentation/embedding）のパスは呼び出し側が解決（オンデマンドDL）。
#[cfg(feature = "diarization")]
pub struct SherpaDiarizer {
    pub segmentation_model: std::path::PathBuf,
    pub embedding_model: std::path::PathBuf,
}

#[cfg(feature = "diarization")]
impl Diarizer for SherpaDiarizer {
    fn diarize(&self, audio: &[f32], _sr: u32) -> Result<Vec<SpeakerSegment>, String> {
        use sherpa_rs::diarize::{Diarize, DiarizeConfig};
        // 話者数は未知前提: num_clusters<=0 で閾値ベースの自動推定（単一話者は1へ収束）。
        // 閾値は 0.8（実検証で決定 / 2026-07-10）。sherpa 既定 0.5 は過分割で、4話者テスト音声で
        // 0.5→7・0.7→5・0.9→4話者と検出された。1人を複数話者に割る偽検出は用途上有害なため、
        // 分割を抑える高め(0.8)を既定にする。TODO(Phase2): 日本語実音声での最適閾値を実測で確定する
        // （ADR-0031 が留保。話者数既知なら num_clusters 固定＝4話者で正確一致を確認済み）。
        let config = DiarizeConfig {
            num_clusters: Some(-1),
            threshold: Some(0.8),
            ..Default::default()
        };
        let mut d = Diarize::new(&self.segmentation_model, &self.embedding_model, config)
            .map_err(|e| format!("diarize init failed: {e}"))?;
        let segs = d
            .compute(audio.to_vec(), None)
            .map_err(|e| format!("diarize compute failed: {e}"))?;
        // sherpa は秒(f32)。whisper タイムスタンプに合わせてセンチ秒へ。
        Ok(segs
            .into_iter()
            .map(|s| SpeakerSegment {
                start_cs: (s.start * 100.0).round() as i64,
                end_cs: (s.end * 100.0).round() as i64,
                speaker: s.speaker,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seg(start_cs: i64, end_cs: i64, speaker: i32) -> SpeakerSegment {
        SpeakerSegment { start_cs, end_cs, speaker }
    }

    #[test]
    fn empty_turns_yields_no_speaker() {
        assert_eq!(assign_speaker(0, 100, &[]), None);
    }

    #[test]
    fn assigns_by_max_overlap() {
        // 話者0 [0,50), 話者1 [50,200)。セグメント [40,120) は話者1と重複80 > 話者0と重複10。
        let turns = [seg(0, 50, 0), seg(50, 200, 1)];
        assert_eq!(assign_speaker(40, 120, &turns), Some(1));
        // セグメント [0,45) は話者0のみ。
        assert_eq!(assign_speaker(0, 45, &turns), Some(0));
    }

    #[test]
    fn tie_prefers_smaller_speaker_id() {
        // 話者0 [0,100), 話者1 [100,200)。セグメント [50,150) は各50で同点→小ID(0)。
        let turns = [seg(0, 100, 0), seg(100, 200, 1)];
        assert_eq!(assign_speaker(50, 150, &turns), Some(0));
    }

    #[test]
    fn no_overlap_falls_back_to_nearest() {
        // 話者0 [0,100), 話者1 [1000,1100)。セグメント [400,420) は中点410、話者0中点50が近い。
        let turns = [seg(0, 100, 0), seg(1000, 1100, 1)];
        assert_eq!(assign_speaker(400, 420, &turns), Some(0));
        // セグメント [900,950] は話者1(中点1050)が近い。
        assert_eq!(assign_speaker(900, 950, &turns), Some(1));
    }

    #[test]
    fn single_speaker_always_that_speaker() {
        let turns = [seg(0, 5000, 0)];
        assert_eq!(assign_speaker(100, 200, &turns), Some(0));
        assert_eq!(assign_speaker(9000, 9100, &turns), Some(0)); // 区間外でも最近傍で0
    }

    #[test]
    fn label_is_one_indexed() {
        assert_eq!(speaker_label(0), "[話者1]");
        assert_eq!(speaker_label(2), "[話者3]");
    }
}
