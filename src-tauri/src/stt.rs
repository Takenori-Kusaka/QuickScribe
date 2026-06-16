// 文字起こし(TranscriptionEngine の実装: ローカル whisper.cpp / ADR-0002)。
// 任意の対応音声(mp3/wav/flac/aac/ogg 等) → 16kHz mono f32 → テキスト。
// デコードは純Rustの symphonia（外部ffmpeg不要・配布が軽い / S1.6 圧縮音声対応）。

use std::fs::File;
use std::path::Path;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

/// whisper が要求するサンプルレート。
pub const WHISPER_SR: u32 = 16000;

/// 任意の対応音声ファイルを 16kHz mono f32 にデコードする。
pub fn decode_to_16k_mono(path: &Path) -> Result<Vec<f32>, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| format!("音声形式の判別に失敗: {e}"))?;
    let mut format = probed.format;

    let track = format.default_track().ok_or("音声トラックがありません")?;
    let track_id = track.id;
    let src_rate = track.codec_params.sample_rate.unwrap_or(WHISPER_SR);
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| format!("デコーダ生成に失敗: {e}"))?;

    let mut mono: Vec<f32> = Vec::new();
    let mut sample_buf: Option<SampleBuffer<f32>> = None;
    let mut channels = 1usize;

    while let Ok(packet) = format.next_packet() {
        if packet.track_id() != track_id {
            continue;
        }
        match decoder.decode(&packet) {
            Ok(decoded) => {
                if sample_buf.is_none() {
                    let spec = *decoded.spec();
                    channels = spec.channels.count().max(1);
                    let dur = decoded.capacity() as u64;
                    sample_buf = Some(SampleBuffer::<f32>::new(dur, spec));
                }
                if let Some(buf) = sample_buf.as_mut() {
                    buf.copy_interleaved_ref(decoded);
                    for frame in buf.samples().chunks(channels) {
                        let sum: f32 = frame.iter().copied().sum();
                        mono.push(sum / channels as f32);
                    }
                }
            }
            Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
            Err(e) => return Err(format!("デコードエラー: {e}")),
        }
    }

    Ok(resample_linear(&mono, src_rate, WHISPER_SR))
}

/// 単純な線形補間リサンプラ（純粋関数・テスト対象 S7.1）。
pub fn resample_linear(input: &[f32], from: u32, to: u32) -> Vec<f32> {
    if from == to || input.is_empty() {
        return input.to_vec();
    }
    let ratio = to as f64 / from as f64;
    let out_len = ((input.len() as f64) * ratio).round() as usize;
    let mut out = Vec::with_capacity(out_len);
    let last = input.len() - 1;
    for i in 0..out_len {
        let src_pos = i as f64 / ratio;
        let idx = src_pos.floor() as usize;
        let frac = (src_pos - idx as f64) as f32;
        let a = input[idx.min(last)];
        let b = input[(idx + 1).min(last)];
        out.push(a + (b - a) * frac);
    }
    out
}

/// whisper.cpp モデルで音声(16kHz mono f32)を文字起こしする。
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
        let seg = state.full_get_segment_text(i).map_err(|e| e.to_string())?;
        text.push_str(seg.trim());
    }
    Ok(text.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resample_same_rate_is_identity() {
        let v = vec![0.1, 0.2, 0.3];
        assert_eq!(resample_linear(&v, 16000, 16000), v);
    }

    #[test]
    fn resample_downsample_halves_length() {
        let v: Vec<f32> = (0..100).map(|i| i as f32).collect();
        let out = resample_linear(&v, 32000, 16000);
        assert!((out.len() as i32 - 50).abs() <= 1);
    }

    #[test]
    fn resample_empty_is_empty() {
        assert!(resample_linear(&[], 44100, 16000).is_empty());
    }
}
