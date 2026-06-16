// 文字起こし(TranscriptionEngine の実装: ローカル whisper.cpp / ADR-0002)。
// 音声(16kHz mono f32) → テキスト。音声ファイル入力(S1.6)とマイク録音(S1.1)の双方から使う。

use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

/// 16kHz mono を前提に WAV を f32 サンプル列へ読み込む。
/// （多チャンネル/異サンプルレートの正規化は呼び出し側の責務。E2Eは 16k mono を用意する。）
pub fn read_wav_16k_mono(path: &Path) -> Result<Vec<f32>, String> {
    let mut reader = hound::WavReader::open(path).map_err(|e| e.to_string())?;
    let spec = reader.spec();
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .map(|s| s.unwrap_or(0.0))
            .collect(),
        hound::SampleFormat::Int => {
            let max = (1i64 << (spec.bits_per_sample - 1)) as f32;
            reader
                .samples::<i32>()
                .map(|s| s.unwrap_or(0) as f32 / max)
                .collect()
        }
    };
    Ok(samples)
}

/// whisper.cpp モデルで音声を文字起こしする。
pub fn transcribe(model_path: &Path, audio: &[f32], lang: Option<&str>) -> Result<String, String> {
    let model = model_path.to_str().ok_or("モデルパスが不正です")?;
    let ctx = WhisperContext::new_with_params(model, WhisperContextParameters::default())
        .map_err(|e| format!("モデル読込に失敗: {e}"))?;
    let mut state = ctx.create_state().map_err(|e| e.to_string())?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    if let Some(l) = lang {
        params.set_language(Some(l));
    }
    params.set_print_progress(false);
    params.set_print_special(false);
    params.set_print_realtime(false);

    state.full(params, audio).map_err(|e| e.to_string())?;

    let n = state.full_n_segments().map_err(|e| e.to_string())?;
    let mut text = String::new();
    for i in 0..n {
        let seg = state
            .full_get_segment_text(i)
            .map_err(|e| e.to_string())?;
        text.push_str(seg.trim());
    }
    Ok(text.trim().to_string())
}
