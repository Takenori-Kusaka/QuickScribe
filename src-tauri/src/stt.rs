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

/// whisperのセグメント時刻(センチ秒=1/100秒)を [HH:MM:SS] 文字列にする（純粋関数・テスト対象）。
pub fn format_timestamp(centiseconds: i64) -> String {
    let total_secs = (centiseconds.max(0) / 100) as u64;
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;
    format!("{h:02}:{m:02}:{s:02}")
}

/// whisper.cpp モデルで文字起こしする（進捗0-100%と確定セグメントを逐次通知）。
/// 進捗UX(プログレス/逐次表示)を可能にしつつ、全CPUコアで高速化する。
/// `timestamps` が true のとき、各セグメント行頭に [HH:MM:SS] を付与する
/// （整形AIが発話の時間関係を解釈できるようにする / whisper-metadata 調査）。
pub fn transcribe_with<P, S>(
    model_path: &Path,
    audio: &[f32],
    lang: Option<&str>,
    timestamps: bool,
    on_progress: P,
    on_segment: S,
) -> Result<String, String>
where
    P: FnMut(i32) + Send + 'static,
    S: FnMut(String) + Send + 'static,
{
    let model = model_path.to_str().ok_or("モデルパスが不正です")?;
    let ctx = WhisperContext::new_with_params(model, WhisperContextParameters::default())
        .map_err(|e| format!("モデル読込に失敗: {e}"))?;
    let mut state = ctx.create_state().map_err(|e| e.to_string())?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    if let Some(l) = lang {
        params.set_language(Some(l));
    }
    // 物理コア数でスレッド設定（論理コア全指定はメモリ帯域律速で逆効果になりうる）。
    let threads = num_cpus::get_physical().max(1) as i32;
    params.set_n_threads(threads);
    params.set_print_progress(false);
    params.set_print_special(false);
    params.set_print_realtime(false);

    // 進捗(0-100)と確定セグメントの逐次通知。
    params.set_progress_callback_safe(on_progress);
    let mut on_seg = on_segment;
    params.set_segment_callback_safe(move |data: whisper_rs::SegmentCallbackData| {
        on_seg(data.text);
    });

    state.full(params, audio).map_err(|e| e.to_string())?;

    let n = state.full_n_segments().map_err(|e| e.to_string())?;
    let mut text = String::new();
    for i in 0..n {
        let seg = state.full_get_segment_text(i).map_err(|e| e.to_string())?;
        let seg = seg.trim();
        if timestamps {
            // セグメント開始時刻(センチ秒)を行頭に付与。AI整形が時間関係を解釈できる。
            let t0 = state.full_get_segment_t0(i).unwrap_or(0);
            text.push_str(&format!("[{}] {}\n", format_timestamp(t0), seg));
        } else {
            text.push_str(seg);
        }
    }
    Ok(text.trim().to_string())
}

/// コールバックなしの簡易版（統合テスト等で使用）。
pub fn transcribe(model_path: &Path, audio: &[f32], lang: Option<&str>) -> Result<String, String> {
    transcribe_with(model_path, audio, lang, false, |_| {}, |_| {})
}

/// 文字起こしエンジンの抽象（S2.3 / Strategy・DIP 境界）。
/// ローカル whisper.cpp とクラウドSTT(S2.4)を同一インターフェイスで差し替え可能にする。
/// 進捗(0-100)と確定セグメントは所有クロージャ(whisperのコールバック制約=Send+'static)で受ける。
pub trait TranscriptionEngine {
    fn transcribe(
        &self,
        audio: &[f32],
        lang: Option<&str>,
        timestamps: bool,
        on_progress: Box<dyn FnMut(i32) + Send>,
        on_segment: Box<dyn FnMut(String) + Send>,
    ) -> Result<String, String>;
}

/// 16kHz mono f32 を WAV(PCM16) バイト列へエンコードする（クラウドSTT送信用 / S2.4）。
/// 全クラウドプロバイダが受け付ける最小形式。16k mono16bit ≈ 1.9MB/分。
pub fn encode_wav_16k_mono(samples: &[f32]) -> Result<Vec<u8>, String> {
    use std::io::Cursor;
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: WHISPER_SR,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut cursor = Cursor::new(Vec::<u8>::new());
    {
        let mut writer =
            hound::WavWriter::new(&mut cursor, spec).map_err(|e| format!("WAV生成に失敗: {e}"))?;
        for &s in samples {
            let v = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            writer
                .write_sample(v)
                .map_err(|e| format!("WAV書込に失敗: {e}"))?;
        }
        writer.finalize().map_err(|e| format!("WAV確定に失敗: {e}"))?;
    }
    Ok(cursor.into_inner())
}

/// multipart/form-data 本文を組み立てる（ureqに組込multipartが無いため最小実装 / S2.4）。
/// fields=テキスト項目、末尾にファイル項目を1つ付す。戻り値=(Content-Type, body)。
fn build_multipart(
    fields: &[(&str, &str)],
    file_field: &str,
    filename: &str,
    file_ct: &str,
    file_bytes: &[u8],
) -> (String, Vec<u8>) {
    let boundary = "----QuickScribeFormBoundary8x2Lq9Zk1Wp0Vn";
    let mut body = Vec::new();
    for (k, v) in fields {
        body.extend_from_slice(
            format!("--{boundary}\r\nContent-Disposition: form-data; name=\"{k}\"\r\n\r\n{v}\r\n")
                .as_bytes(),
        );
    }
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"{file_field}\"; filename=\"{filename}\"\r\nContent-Type: {file_ct}\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(file_bytes);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    (format!("multipart/form-data; boundary={boundary}"), body)
}

/// OpenAI互換の文字起こしAPI（Groq / OpenAI / ADR-0016 Phase A）。
/// `POST {base_url}/audio/transcriptions`・Bearer認証・multipart(file+model)・本文は `.text`。
/// クラウドは逐次セグメント通知を持たないため進捗は段階的、本文は一括（timestamps は未対応）。
pub struct OpenAiCompatibleSttEngine {
    /// 例: https://api.groq.com/openai/v1 / https://api.openai.com/v1
    pub base_url: String,
    pub model: String,
    pub api_key: String,
}

impl TranscriptionEngine for OpenAiCompatibleSttEngine {
    fn transcribe(
        &self,
        audio: &[f32],
        lang: Option<&str>,
        _timestamps: bool,
        mut on_progress: Box<dyn FnMut(i32) + Send>,
        mut on_segment: Box<dyn FnMut(String) + Send>,
    ) -> Result<String, String> {
        if self.api_key.trim().is_empty() {
            return Err("クラウドSTTのAPIキーが未設定です（設定から入力してください）".into());
        }
        on_progress(5);
        let wav = encode_wav_16k_mono(audio)?;
        let url = format!("{}/audio/transcriptions", self.base_url.trim_end_matches('/'));
        let mut fields: Vec<(&str, &str)> = vec![("model", self.model.as_str())];
        if let Some(l) = lang {
            fields.push(("language", l));
        }
        fields.push(("response_format", "json"));
        let (content_type, body) = build_multipart(&fields, "file", "audio.wav", "audio/wav", &wav);
        on_progress(30);
        let resp = match ureq::post(&url)
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .set("Content-Type", &content_type)
            .send_bytes(&body)
        {
            Ok(r) => r,
            Err(ureq::Error::Status(code, r)) => {
                let detail: String = r
                    .into_string()
                    .unwrap_or_default()
                    .chars()
                    .take(300)
                    .collect();
                return Err(format!("クラウドSTTエラー({code}): {detail}"));
            }
            Err(e) => return Err(format!("クラウドSTT通信に失敗: {e}")),
        };
        let json: serde_json::Value = resp
            .into_json()
            .map_err(|e| format!("クラウドSTT応答の解析に失敗: {e}"))?;
        let text = json
            .get("text")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        on_progress(100);
        if !text.is_empty() {
            on_segment(text.clone());
        }
        Ok(text)
    }
}

/// ローカル whisper.cpp 実装（既定エンジン / ADR-0002）。
pub struct LocalWhisperEngine {
    pub model_path: std::path::PathBuf,
}

impl TranscriptionEngine for LocalWhisperEngine {
    fn transcribe(
        &self,
        audio: &[f32],
        lang: Option<&str>,
        timestamps: bool,
        on_progress: Box<dyn FnMut(i32) + Send>,
        on_segment: Box<dyn FnMut(String) + Send>,
    ) -> Result<String, String> {
        transcribe_with(
            &self.model_path,
            audio,
            lang,
            timestamps,
            on_progress,
            on_segment,
        )
    }
}

/// STT解決の設定（S2.3/S2.4）。provider と鍵/モデル、ローカル用モデルパスを束ねる。
pub struct SttConfig {
    /// "local"(既定) / "groq" / "openai"（Deepgram/Azureは Phase B）。
    pub provider: String,
    /// クラウドのモデルID（空ならプロバイダ既定）。
    pub model: String,
    /// クラウドのAPIキー（ローカルでは未使用）。
    pub api_key: String,
    /// ローカル whisper のモデルファイルパス。
    pub model_path: std::path::PathBuf,
}

/// STT設定からエンジンを解決する（S2.3抽象 / S2.4でクラウド追加 / ADR-0016）。
/// 空/未知/"local"/"whisper" はローカル whisper にフォールバック（プライバシー既定）。
pub fn engine_for(cfg: SttConfig) -> Box<dyn TranscriptionEngine> {
    let model = |default: &str| {
        if cfg.model.trim().is_empty() {
            default.to_string()
        } else {
            cfg.model.clone()
        }
    };
    match cfg.provider.trim().to_ascii_lowercase().as_str() {
        "groq" => Box::new(OpenAiCompatibleSttEngine {
            base_url: "https://api.groq.com/openai/v1".into(),
            model: model("whisper-large-v3-turbo"),
            api_key: cfg.api_key,
        }),
        "openai" => Box::new(OpenAiCompatibleSttEngine {
            base_url: "https://api.openai.com/v1".into(),
            model: model("gpt-4o-transcribe"),
            api_key: cfg.api_key,
        }),
        _ => Box::new(LocalWhisperEngine {
            model_path: cfg.model_path,
        }),
    }
}

/// プロバイダがクラウド（端末外送信）かを返す。lib側でモデルDL要否やUX分岐に使う。
/// engine_for が実装済みのものだけを列挙する（未実装はローカルへフォールバックさせる）。
/// Phase B で Deepgram/Azure を実装したらここに追加する。
pub fn is_cloud_provider(provider: &str) -> bool {
    matches!(
        provider.trim().to_ascii_lowercase().as_str(),
        "groq" | "openai"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resample_same_rate_is_identity() {
        let v = vec![0.1, 0.2, 0.3];
        assert_eq!(resample_linear(&v, 16000, 16000), v);
    }

    fn cfg(provider: &str) -> SttConfig {
        SttConfig {
            provider: provider.into(),
            model: String::new(),
            api_key: "k".into(),
            model_path: std::path::PathBuf::from("dummy.bin"),
        }
    }

    #[test]
    fn engine_for_is_total_and_returns_engine() {
        // どのプロバイダ名でもパニックせずエンジンを返す。
        for p in ["", "local", "whisper", "groq", "openai", "unknown"] {
            let _engine: Box<dyn TranscriptionEngine> = engine_for(cfg(p));
        }
    }

    #[test]
    fn is_cloud_provider_classifies() {
        assert!(is_cloud_provider("groq"));
        assert!(is_cloud_provider("openai"));
        assert!(is_cloud_provider("OpenAI"));
        assert!(!is_cloud_provider("local"));
        assert!(!is_cloud_provider(""));
        // Phase B未実装はローカル扱い(フォールバック)。
        assert!(!is_cloud_provider("deepgram"));
    }

    #[test]
    fn wav_encode_has_riff_header_and_grows_with_samples() {
        // WAVは "RIFF"/"WAVE" ヘッダを持ち、44byteヘッダ＋PCM16(2byte/sample)。
        let wav = encode_wav_16k_mono(&[0.0, 0.5, -0.5, 1.0]).unwrap();
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(wav.len(), 44 + 4 * 2, "44byteヘッダ＋4サンプル×2byte");
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

    #[test]
    fn timestamp_formats_hms() {
        assert_eq!(format_timestamp(0), "00:00:00");
        assert_eq!(format_timestamp(100), "00:00:01"); // 100センチ秒=1秒
        assert_eq!(format_timestamp(6125), "00:01:01"); // 61.25秒
        assert_eq!(format_timestamp(366000), "01:01:00"); // 3660秒
    }

    #[test]
    fn timestamp_clamps_negative() {
        assert_eq!(format_timestamp(-50), "00:00:00");
    }
}
