// 文字起こし(TranscriptionEngine の実装: ローカル whisper.cpp / ADR-0002)。
// 任意の対応音声(mp3/wav/flac/aac/ogg-vorbis 等) → 16kHz mono f32 → テキスト。
// デコードは純Rustの symphonia（外部ffmpeg不要・配布が軽い / S1.6 圧縮音声対応）。
// ただし .opus(Ogg Opus)は symphonia 非対応のため、録音で使う opus+ogg クレートで自前デコードする
// （自分で保存した .opus 録音の再文字起こしを可能に。新規依存なし）。

use std::fs::File;
use std::path::Path;
use symphonia::core::codecs::audio::AudioDecoderOptions;
use symphonia::core::codecs::CodecParameters;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

/// whisper が要求するサンプルレート。
pub const WHISPER_SR: u32 = 16000;

/// 任意の対応音声ファイルを 16kHz mono f32 にデコードする。
pub fn decode_to_16k_mono(path: &Path) -> Result<Vec<f32>, String> {
    // .opus(Ogg Opus)は symphonia0.6 が非対応。録音(save_opus)で使う opus(libopus)+ogg クレートを
    // そのまま流用して自前デコードする（新規依存なし。自分で保存した .opus 録音の再文字起こしを可能に）。
    // 他形式(mp3/wav/flac/aac/ogg-vorbis 等)は従来どおり symphonia。
    if path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("opus"))
    {
        return decode_opus_ogg_to_16k(path);
    }
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    // symphonia0.6: get_probe().probe() は FormatReader を直接返す(fmt/meta opts は値渡し)。
    let mut format = symphonia::default::get_probe()
        .probe(
            &hint,
            mss,
            FormatOptions::default(),
            MetadataOptions::default(),
        )
        .map_err(|e| crate::errcode::ec(crate::errcode::E_AUDIO_PROBE, e))?;

    let track = format
        .default_track(TrackType::Audio)
        .ok_or("音声トラックがありません")?;
    let track_id = track.id;
    // symphonia0.6: codec_params は Option<CodecParameters> の enum。Audio を取り出す。
    let audio_params = match &track.codec_params {
        Some(CodecParameters::Audio(p)) => p,
        _ => return Err(crate::errcode::E_NO_CODEC_PARAMS.into()),
    };
    let src_rate = audio_params.sample_rate.unwrap_or(WHISPER_SR);
    let mut decoder = symphonia::default::get_codecs()
        .make_audio_decoder(audio_params, &AudioDecoderOptions::default())
        .map_err(|e| crate::errcode::ec(crate::errcode::E_DECODER_BUILD, e))?;

    let mut mono: Vec<f32> = Vec::new();
    let mut interleaved: Vec<f32> = Vec::new();

    // symphonia0.6: next_packet() は EOF で Ok(None) を返す。SampleBuffer は廃止され、
    // GenericAudioBufferRef::copy_to_vec_interleaved で f32 インターリーブを取得する。
    while let Some(packet) = format
        .next_packet()
        .map_err(|e| crate::errcode::ec(crate::errcode::E_PACKET_READ, e))?
    {
        if packet.track_id != track_id {
            continue;
        }
        match decoder.decode(&packet) {
            Ok(decoded) => {
                let channels = decoded.spec().channels().count().max(1);
                interleaved.clear();
                decoded.copy_to_vec_interleaved(&mut interleaved);
                for frame in interleaved.chunks(channels) {
                    let sum: f32 = frame.iter().copied().sum();
                    mono.push(sum / channels as f32);
                }
            }
            Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
            Err(e) => return Err(crate::errcode::ec(crate::errcode::E_DECODE, e)),
        }
    }

    Ok(resample_linear(&mono, src_rate, WHISPER_SR))
}

/// 自前録音の Ogg Opus(.opus) を 16kHz mono f32 へデコードする。
/// symphonia は Opus 非対応だが、録音で使う opus(libopus vendored)+ogg クレートをそのまま使う。
/// save_opus と対称: 48kHz mono で復号し、先頭の識別/コメントヘッダ(OpusHead/OpusTags)は
/// 音声でないためスキップ。最後に 48kHz→16kHz へ線形リサンプルする。
fn decode_opus_ogg_to_16k(path: &Path) -> Result<Vec<f32>, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut reader = ogg::reading::PacketReader::new(std::io::BufReader::new(file));
    let mut dec = opus::Decoder::new(48_000, opus::Channels::Mono)
        .map_err(|e| crate::errcode::ec(crate::errcode::E_DECODER_BUILD, e))?;
    let mut pcm: Vec<f32> = Vec::new();
    let mut buf = vec![0f32; 5760]; // 120ms @48kHz mono の上限フレーム。
    while let Some(packet) = reader
        .read_packet()
        .map_err(|e| crate::errcode::ec(crate::errcode::E_PACKET_READ, e))?
    {
        // 識別ヘッダ(OpusHead)/コメントヘッダ(OpusTags)は音声パケットではないのでスキップ。
        if packet.data.starts_with(b"OpusHead") || packet.data.starts_with(b"OpusTags") {
            continue;
        }
        match dec.decode_float(&packet.data, &mut buf, false) {
            Ok(n) => pcm.extend_from_slice(&buf[..n]),
            Err(_) => continue, // 壊れた/端数パケットは飛ばす。
        }
    }
    if pcm.is_empty() {
        return Err(crate::errcode::E_NO_CODEC_PARAMS.into());
    }
    Ok(resample_linear(&pcm, 48_000, WHISPER_SR))
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
        .map_err(|e| crate::errcode::ec(crate::errcode::E_STT_MODEL_LOAD, e))?;
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
            hound::WavWriter::new(&mut cursor, spec).map_err(|e| crate::errcode::ec(crate::errcode::E_WAV_BUILD, e))?;
        for &s in samples {
            let v = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            writer
                .write_sample(v)
                .map_err(|e| crate::errcode::ec(crate::errcode::E_WAV_WRITE, e))?;
        }
        writer.finalize().map_err(|e| crate::errcode::ec(crate::errcode::E_WAV_FINALIZE, e))?;
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
            return Err(crate::errcode::E_CLOUD_STT_NO_KEY.into());
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
        let json = read_json_response(
            ureq::post(&url)
                .config()
                .http_status_as_error(false)
                .build()
                .header("Authorization", &format!("Bearer {}", self.api_key))
                .header("Content-Type", &content_type)
                .send(&body[..]),
        )?;
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

/// ureq3のPOST応答をJSONにし、非2xxは状態と短い詳細でErrにする共通処理。
/// ureq3 は既定で非2xxを Err(StatusCode) にして本文を捨てるため、呼び出し側は
/// `.config().http_status_as_error(false).build()` で 4xx/5xx も Ok として渡すこと。
fn read_json_response(
    result: Result<http::Response<ureq::Body>, ureq::Error>,
) -> Result<serde_json::Value, String> {
    let mut r = result.map_err(|e| crate::errcode::ec(crate::errcode::E_CLOUD_STT_HTTP, e))?;
    let code = r.status().as_u16();
    if !(200..300).contains(&code) {
        let detail: String = r
            .body_mut()
            .read_to_string()
            .unwrap_or_default()
            .chars()
            .take(300)
            .collect();
        return Err(crate::errcode::ec(crate::errcode::E_CLOUD_STT_STATUS, format!("{code}: {detail}")));
    }
    r.body_mut()
        .read_json()
        .map_err(|e| crate::errcode::ec(crate::errcode::E_CLOUD_STT_PARSE, e))
}

/// Deepgram 文字起こし（pre-recorded / ADR-0016 Phase B）。
/// `POST api.deepgram.com/v1/listen?model=..&language=ja`・`Token`認証・生WAV本文。
/// 既定でモデル学習に使われないが、防御的に `mip_opt_out=true` を付す。
/// 本文は `results.channels[0].alternatives[0].transcript`。
pub struct DeepgramSttEngine {
    pub model: String,
    pub api_key: String,
}

impl TranscriptionEngine for DeepgramSttEngine {
    fn transcribe(
        &self,
        audio: &[f32],
        lang: Option<&str>,
        _timestamps: bool,
        mut on_progress: Box<dyn FnMut(i32) + Send>,
        mut on_segment: Box<dyn FnMut(String) + Send>,
    ) -> Result<String, String> {
        if self.api_key.trim().is_empty() {
            return Err(crate::errcode::E_CLOUD_STT_NO_KEY.into());
        }
        on_progress(5);
        let wav = encode_wav_16k_mono(audio)?;
        let lang = lang.unwrap_or("ja");
        let url = format!(
            "https://api.deepgram.com/v1/listen?model={}&language={}&smart_format=true&mip_opt_out=true",
            self.model, lang
        );
        on_progress(30);
        let json = read_json_response(
            ureq::post(&url)
                .config()
                .http_status_as_error(false)
                .build()
                .header("Authorization", &format!("Token {}", self.api_key))
                .header("Content-Type", "audio/wav")
                .send(&wav[..]),
        )?;
        let text = json
            .pointer("/results/channels/0/alternatives/0/transcript")
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

/// Azure AI Speech Fast Transcription（同期 / ADR-0016 Phase B）。
/// `POST {resource}.cognitiveservices.azure.com/speechtotext/transcriptions:transcribe`・
/// `Ocp-Apim-Subscription-Key`認証・multipart(audio＋definition JSON)。本文は `combinedPhrases[0].text`。
pub struct AzureSttEngine {
    /// リソース名（例: myres → host myres.cognitiveservices.azure.com）。
    pub resource: String,
    pub api_key: String,
    /// BCP-47 ロケール（例: ja-JP）。
    pub locale: String,
}

impl TranscriptionEngine for AzureSttEngine {
    fn transcribe(
        &self,
        audio: &[f32],
        _lang: Option<&str>,
        _timestamps: bool,
        mut on_progress: Box<dyn FnMut(i32) + Send>,
        mut on_segment: Box<dyn FnMut(String) + Send>,
    ) -> Result<String, String> {
        if self.api_key.trim().is_empty() {
            return Err(crate::errcode::E_CLOUD_STT_NO_KEY.into());
        }
        if self.resource.trim().is_empty() {
            return Err(crate::errcode::E_AZURE_NO_RESOURCE.into());
        }
        on_progress(5);
        let wav = encode_wav_16k_mono(audio)?;
        let url = format!(
            "https://{}.cognitiveservices.azure.com/speechtotext/transcriptions:transcribe?api-version=2025-10-15",
            self.resource.trim()
        );
        let definition = format!("{{\"locales\":[\"{}\"]}}", self.locale);
        let (content_type, body) =
            build_multipart(&[("definition", &definition)], "audio", "audio.wav", "audio/wav", &wav);
        on_progress(30);
        let json = read_json_response(
            ureq::post(&url)
                .config()
                .http_status_as_error(false)
                .build()
                .header("Ocp-Apim-Subscription-Key", self.api_key.trim())
                .header("Content-Type", &content_type)
                .send(&body[..]),
        )?;
        let text = json
            .pointer("/combinedPhrases/0/text")
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
    /// "local"(既定) / "groq" / "openai" / "deepgram" / "azure"。
    pub provider: String,
    /// クラウドのモデルID（空ならプロバイダ既定）。
    pub model: String,
    /// クラウドのAPIキー（ローカルでは未使用）。
    pub api_key: String,
    /// Azure のリソース名（azure のときのみ使用）。
    pub azure_resource: String,
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
        "deepgram" => Box::new(DeepgramSttEngine {
            model: model("nova-3"),
            api_key: cfg.api_key,
        }),
        "azure" => Box::new(AzureSttEngine {
            resource: cfg.azure_resource,
            api_key: cfg.api_key,
            locale: "ja-JP".into(),
        }),
        _ => Box::new(LocalWhisperEngine {
            model_path: cfg.model_path,
        }),
    }
}

/// プロバイダがクラウド（端末外送信）かを返す。lib側でモデルDL要否やUX分岐に使う。
/// engine_for が実装済みのものだけを列挙する（未実装はローカルへフォールバックさせる）。
pub fn is_cloud_provider(provider: &str) -> bool {
    matches!(
        provider.trim().to_ascii_lowercase().as_str(),
        "groq" | "openai" | "deepgram" | "azure"
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

    /// symphonia デコード経路の実行時検証(#467 symphonia0.6 移行の回帰ガード)。
    /// 48kHz ステレオ WAV を書き出し → 16k mono へデコード。ダウンミックス・リサンプル・
    /// copy_to_vec_interleaved が機能し、期待長の非空サンプルが得られることを確認する。
    #[test]
    fn decodes_wav_to_16k_mono() {
        let path = std::env::temp_dir().join("qs_symphonia_decode_test.wav");
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 48000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        {
            let mut w = hound::WavWriter::create(&path, spec).unwrap();
            // 0.1秒ぶんのステレオ(4800フレーム)。L/Rに同値の単純波形。
            for i in 0..4800i32 {
                let v = ((i as f32 * 0.05).sin() * 8000.0) as i16;
                w.write_sample(v).unwrap();
                w.write_sample(v).unwrap();
            }
            w.finalize().unwrap();
        }
        let out = decode_to_16k_mono(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        // 48k→16k で約1/3。0.1秒 ≈ 1600 サンプル前後(境界の丸め許容)。
        assert!(!out.is_empty(), "デコード結果が空");
        assert!(
            (1200..2200).contains(&out.len()),
            "想定外のサンプル長: {}",
            out.len()
        );
    }

    fn cfg(provider: &str) -> SttConfig {
        SttConfig {
            provider: provider.into(),
            model: String::new(),
            api_key: "k".into(),
            azure_resource: "res".into(),
            model_path: std::path::PathBuf::from("dummy.bin"),
        }
    }

    #[test]
    fn engine_for_is_total_and_returns_engine() {
        // どのプロバイダ名でもパニックせずエンジンを返す。
        for p in ["", "local", "whisper", "groq", "openai", "deepgram", "azure", "unknown"] {
            let _engine: Box<dyn TranscriptionEngine> = engine_for(cfg(p));
        }
    }

    #[test]
    fn is_cloud_provider_classifies() {
        assert!(is_cloud_provider("groq"));
        assert!(is_cloud_provider("openai"));
        assert!(is_cloud_provider("OpenAI"));
        assert!(is_cloud_provider("deepgram"));
        assert!(is_cloud_provider("azure"));
        assert!(!is_cloud_provider("local"));
        assert!(!is_cloud_provider(""));
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
