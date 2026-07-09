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
        .ok_or(crate::errcode::E_NO_AUDIO_TRACK)?;
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

/// 長尺音声のチャンク分割パラメータ（#600 末尾欠落の根治）。
/// whisper の窓は30秒。それ未満のチャンクに切って個別デコードすることで、
/// whisper.cpp の sequential seek が長尺で末尾を落とす構造的問題を回避する。
/// overlap は境界で語が切れないための重なり。担当区間は overlap の中点で切って重複排除する。
pub const CHUNK_SECS: f64 = 24.0;
// overlap は境界の両側マージン。担当区間は overlap の中点で切るため、各チャンクは自分の音声末尾から
// overlap/2 だけ手前までしか担当しない＝末尾帯(whisper が単一窓でも稀に落としうる領域)を担当しない。
// 隣接チャンクは overlap/2 だけ内側から担当を始める＝コールドスタート直後の不安定セグメントも避ける。
// 4s(両側2sマージン)で末尾欠落の再発余地を潰す（独立レビュー指摘）。冗長デコードは約17%。
pub const OVERLAP_SECS: f64 = 4.0;

/// 1チャンクの分割仕様（純粋計算 / #600）。時刻はセンチ秒（1/100秒＝whisperのt0単位）。
#[derive(Debug, Clone, PartialEq)]
pub struct ChunkSpec {
    /// サンプル開始インデックス（含む）。
    pub start: usize,
    /// サンプル終端インデックス（含まず）。
    pub end: usize,
    /// チャンク先頭の絶対時刻（センチ秒）。セグメント時刻のオフセットに使う。
    pub offset_cs: i64,
    /// このチャンクが出力を担当する絶対区間の開始（含む・センチ秒）。
    pub own_start_cs: i64,
    /// 担当区間の終端（含まず）。最終チャンクは i64::MAX。
    pub own_end_cs: i64,
}

/// 音声を `chunk_secs` 長・`overlap_secs` 重なりで分割する計画を返す（純粋関数・テスト対象 / #600）。
/// 各チャンクを個別に whisper へ渡すことで長尺末尾の欠落を防ぐ。担当区間 `[own_start_cs, own_end_cs)`
/// は隣接チャンクの重なりの中点で境界を引き、overlap 領域の重複出力を決定的に排除する。
/// `total` が `chunk_secs` 以下なら単一チャンク＝従来動作（短尺は無回帰）。
pub fn chunk_plan(total: usize, sr: u32, chunk_secs: f64, overlap_secs: f64) -> Vec<ChunkSpec> {
    if total == 0 {
        return Vec::new();
    }
    let sr_f = sr as f64;
    let chunk_n = ((chunk_secs * sr_f).round() as usize).max(1);
    // overlap は「チャンクの半分」を上限に制限する。stride>=chunk/2 を保証し、病的入力
    // (overlap>=chunk)での stride≒1サンプル→チャンク爆発を防ぐ（レビュー指摘）。overlap>50%は無意味。
    let overlap_n = ((overlap_secs.max(0.0) * sr_f).round() as usize).min(chunk_n / 2);
    let stride_n = (chunk_n - overlap_n).max(1);
    // half_overlap は「実際に stride に使う clamp 後の overlap_n」から導出する（生の overlap_secs から
    // 導くと overlap>=chunk の病的入力で担当境界が実 overlap 領域外へずれる / レビュー指摘 Finding3）。
    let half_overlap_cs = ((overlap_n as f64 / sr_f / 2.0) * 100.0).round() as i64;
    let to_cs = |sample: usize| ((sample as f64 / sr_f) * 100.0).round() as i64;

    let mut specs: Vec<ChunkSpec> = Vec::new();
    let mut start = 0usize;
    loop {
        let end = (start + chunk_n).min(total);
        specs.push(ChunkSpec {
            start,
            end,
            offset_cs: to_cs(start),
            own_start_cs: 0,
            own_end_cs: 0,
        });
        if end >= total {
            break;
        }
        start += stride_n;
    }

    // 担当区間を埋める: 先頭は0から、以降は自チャンク先頭＋overlap中点。終端は次チャンクの担当開始。
    let n = specs.len();
    for i in 0..n {
        let own_start = if i == 0 {
            0
        } else {
            specs[i].offset_cs + half_overlap_cs
        };
        let own_end = if i + 1 < n {
            specs[i + 1].offset_cs + half_overlap_cs
        } else {
            i64::MAX
        };
        specs[i].own_start_cs = own_start;
        specs[i].own_end_cs = own_end;
    }
    specs
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
    use_gpu: bool,
    on_progress: P,
    on_segment: S,
) -> Result<String, String>
where
    P: FnMut(i32) + Send + 'static,
    S: FnMut(String) + Send + 'static,
{
    let model = model_path.to_str().ok_or(crate::errcode::E_STT_MODEL_PATH)?;
    // #600 根治: 長尺を chunk_secs 未満に分割し個別デコードする。whisper.cpp の sequential seek が
    // 長尺で末尾を落とす構造的問題を回避する（短尺は単一チャンク＝従来動作で無回帰）。
    let chunks = chunk_plan(audio.len(), WHISPER_SR, CHUNK_SECS, OVERLAP_SECS);
    if chunks.is_empty() {
        return Ok(String::new());
    }
    let n_chunks = chunks.len();
    let threads = num_cpus::get_physical().max(1) as i32;
    // 進捗はチャンクをまたいで 0-100 に正規化して通知する（複数の full() 呼び出しを1本の進捗に集約）。
    let progress = std::sync::Arc::new(std::sync::Mutex::new(on_progress));
    let mut on_seg = on_segment;

    // GPU(Vulkan変種)は設定・実行環境判定に従う(ADR-0028)。GPUの失敗は「ctx初期化」だけでなく
    // 「チャンク実行時(create_state/full)のVRAM不足等」が現実的な失敗面（レビュー指摘）のため、
    // どちらでも CPU で全体を再試行し「速度は落ちても文字起こしは必ず完了する」を保証する。
    // 注: GPU試行で一部チャンクが成功していた場合、プレビューsegmentは再試行で重複しうるが、
    // 最終テキスト(戻り値)は再試行分のみ＝正しい（取りこぼさない方を優先）。
    let gpu_attempts: &[bool] = if use_gpu { &[true, false] } else { &[false] };
    let n_attempts = gpu_attempts.len();
    for (attempt, &gpu) in gpu_attempts.iter().enumerate() {
        let has_fallback = attempt + 1 < n_attempts;
        // モデル(ctx)は試行ごとに1回ロードし、各チャンクは state を都度作って回す（再ロード回避）。
        let mut ctx_params = WhisperContextParameters::default();
        ctx_params.use_gpu(gpu);
        let ctx = match WhisperContext::new_with_params(model, ctx_params) {
            Ok(c) => c,
            Err(e) if has_fallback => {
                eprintln!("[stt] GPU init failed, falling back to CPU: {e}");
                continue;
            }
            Err(e) => return Err(crate::errcode::ec(crate::errcode::E_STT_MODEL_LOAD, e)),
        };
        let mut text = String::new();
        // 1チャンクのデコード失敗で全体(=他チャンクの成功分)を捨てないための直近エラー保持。
        // 全チャンク失敗で結果が空のときだけ最終的に Err を返す（Ok("")で握り潰さない）。
        let mut last_err: Option<String> = None;

        for (idx, spec) in chunks.iter().enumerate() {
            let mut state = match ctx.create_state() {
                Ok(s) => s,
                Err(e) => {
                    last_err = Some(e.to_string());
                    continue;
                }
            };
            let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
            if let Some(l) = lang {
                params.set_language(Some(l));
            }
            // 物理コア数でスレッド設定（論理コア全指定はメモリ帯域律速で逆効果になりうる）。
            params.set_n_threads(threads);
            params.set_print_progress(false);
            params.set_print_special(false);
            params.set_print_realtime(false);
            // ハルシネーション/反復ループ抑制(#600)。前文脈を使うと末尾無音等での反復ループが固定され
            // 末尾が失われるため、前文脈を使わない。チャンク化と併せ末尾欠落を根治する。温度フォールバック・
            // entropy/logprob 閾値は whisper.cpp 既定のループ回復が有効。効果は日本語CERベンチ(ADR-0024)で監視。
            params.set_no_context(true);

            // チャンク内進捗(0-100)を全体の [idx/n, (idx+1)/n] 区間へ写像して通知する。
            let pshare = progress.clone();
            let base = idx as f32 / n_chunks as f32;
            let span = 1.0 / n_chunks as f32;
            params.set_progress_callback_safe(move |p: i32| {
                let global = (((base + span * (p.clamp(0, 100) as f32 / 100.0)) * 100.0) as i32).clamp(0, 100);
                if let Ok(mut f) = pshare.lock() {
                    f(global);
                }
            });

            let chunk = &audio[spec.start..spec.end];
            // チャンク単位で失敗を隔離: このチャンクだけ飛ばし、成功した他チャンクの文字起こしは残す。
            // 隣接チャンクの overlap が失われた区間の一部を補う。長尺で1チャンクの一過性失敗が全体を潰さない。
            if let Err(e) = state.full(params, chunk) {
                eprintln!("[stt] chunk {idx} full() failed, skipped: {e}");
                last_err = Some(e.to_string());
                continue;
            }

            // セグメントを絶対時刻へオフセットし、担当区間 [own_start, own_end) の分だけ採用（重複排除）。
            let n_seg = match state.full_n_segments() {
                Ok(n) => n,
                Err(e) => {
                    last_err = Some(e.to_string());
                    continue;
                }
            };
            for i in 0..n_seg {
                // 稀にチャンク境界の途切れトークンで UTF-8 が壊れ Err になることがある。
                // その1セグメントだけスキップし、全体の文字起こしは捨てない（堅牢性 / #600）。
                // 静かなデータ欠落を残さないよう stderr に痕跡を残す（レビュー指摘: 可視性）。
                let seg = match state.full_get_segment_text(i) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("[stt] chunk {idx} segment {i} text decode failed, skipped: {e}");
                        continue;
                    }
                };
                let seg = seg.trim();
                if seg.is_empty() {
                    continue;
                }
                let t0_local = state.full_get_segment_t0(i).unwrap_or(0);
                let abs_t0 = spec.offset_cs + t0_local;
                // overlap 中点で切った担当区間外のセグメントは隣接チャンクが出すのでスキップ。
                if abs_t0 < spec.own_start_cs || abs_t0 >= spec.own_end_cs {
                    continue;
                }
                // 確定セグメントの逐次通知（プレビュー用）。担当分のみ＝重複プレビューを出さない。
                on_seg(seg.to_string());
                if timestamps {
                    // 絶対時刻を行頭に付与。AI整形が時間関係を解釈できる。
                    text.push_str(&format!("[{}] {}\n", format_timestamp(abs_t0), seg));
                } else {
                    text.push_str(seg);
                }
            }
        }

        // GPU試行で失敗チャンクがあれば、CPUで全体を再試行する（部分欠落を黙って返さない）。
        if last_err.is_some() && has_fallback {
            eprintln!("[stt] chunk failure(s) on GPU attempt, retrying whole transcription on CPU");
            continue;
        }
        if let Ok(mut f) = progress.lock() {
            f(100);
        }
        let out = text.trim().to_string();
        // 結果が空でどこかのチャンクが失敗していたら、その失敗を返す。発話が無く自然に空なら Ok("")。
        if out.is_empty() {
            if let Some(e) = last_err {
                return Err(e);
            }
        }
        return Ok(out);
    }
    unreachable!("gpu_attempts は常に1要素以上")
}

/// コールバックなしの簡易版（統合テスト等で使用）。
pub fn transcribe(model_path: &Path, audio: &[f32], lang: Option<&str>) -> Result<String, String> {
    transcribe_with(model_path, audio, lang, false, true, |_| {}, |_| {})
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
            "{}/v1/listen?model={}&language={}&smart_format=true&mip_opt_out=true",
            crate::api_base("https://api.deepgram.com", "QS_TEST_DEEPGRAM_BASE"),
            self.model,
            lang
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
            "{}/speechtotext/transcriptions:transcribe?api-version=2025-10-15",
            crate::api_base(
                &format!("https://{}.cognitiveservices.azure.com", self.resource.trim()),
                "QS_TEST_AZURE_BASE"
            )
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
    /// GPUで実行するか（Vulkan変種のみ実効。CPUビルドでは無視される / ADR-0028）。
    pub use_gpu: bool,
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
            self.use_gpu,
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
    /// ローカル whisper をGPUで実行するか（ADR-0027。クラウド/CPUビルドでは未使用）。
    pub use_gpu: bool,
}

/// 文字起こしプロバイダ（S2.3/S2.4）。別名解釈・クラウド判定・既定モデル・エンジン生成を
/// 単一ソースに集約する（refine.rs の `RefineProvider` と対称 / #392 の横展開 = #581）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SttProvider {
    /// ローカル whisper（既定・フォールバック先）。
    Local,
    Groq,
    OpenAi,
    Deepgram,
    Azure,
}

impl SttProvider {
    /// フロントから渡る文字列を解釈する。空/未知/"local"/"whisper" は Local（プライバシー既定）。
    pub fn parse(provider: &str) -> Self {
        match provider.trim().to_ascii_lowercase().as_str() {
            "groq" => Self::Groq,
            "openai" => Self::OpenAi,
            "deepgram" => Self::Deepgram,
            "azure" => Self::Azure,
            _ => Self::Local,
        }
    }

    /// クラウド（端末外送信）か。Local だけが false。
    pub fn is_cloud(self) -> bool {
        !matches!(self, Self::Local)
    }

    /// クラウドの既定モデルID（未指定時のフォールバック）。Azure はロケール指定・Local はローカル
    /// モデルパスを使い、いずれもモデルIDを使わないため空。
    pub fn default_model(self) -> &'static str {
        match self {
            Self::Groq => "whisper-large-v3-turbo",
            Self::OpenAi => "gpt-4o-transcribe",
            Self::Deepgram => "nova-3",
            Self::Azure | Self::Local => "",
        }
    }

    /// 設定から対応する文字起こしエンジンを生成する（Strategy の差し替え = DIP 境界）。
    pub fn make_engine(self, cfg: SttConfig) -> Box<dyn TranscriptionEngine> {
        // クラウドのモデルは未指定なら既定へフォールバックする。
        let model = |default: &str| {
            if cfg.model.trim().is_empty() {
                default.to_string()
            } else {
                cfg.model.clone()
            }
        };
        match self {
            Self::Groq => Box::new(OpenAiCompatibleSttEngine {
                base_url: "https://api.groq.com/openai/v1".into(),
                model: model(self.default_model()),
                api_key: cfg.api_key,
            }),
            Self::OpenAi => Box::new(OpenAiCompatibleSttEngine {
                base_url: "https://api.openai.com/v1".into(),
                model: model(self.default_model()),
                api_key: cfg.api_key,
            }),
            Self::Deepgram => Box::new(DeepgramSttEngine {
                model: model(self.default_model()),
                api_key: cfg.api_key,
            }),
            Self::Azure => Box::new(AzureSttEngine {
                resource: cfg.azure_resource,
                api_key: cfg.api_key,
                locale: "ja-JP".into(),
            }),
            Self::Local => Box::new(LocalWhisperEngine {
                model_path: cfg.model_path,
                use_gpu: cfg.use_gpu,
            }),
        }
    }
}

/// STT設定からエンジンを解決する（S2.3抽象 / S2.4でクラウド追加 / ADR-0016）。
/// 空/未知/"local"/"whisper" はローカル whisper にフォールバック（プライバシー既定）。
pub fn engine_for(cfg: SttConfig) -> Box<dyn TranscriptionEngine> {
    SttProvider::parse(&cfg.provider).make_engine(cfg)
}

/// プロバイダがクラウド（端末外送信）かを返す。lib側でモデルDL要否やUX分岐に使う。
pub fn is_cloud_provider(provider: &str) -> bool {
    SttProvider::parse(provider).is_cloud()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resample_same_rate_is_identity() {
        let v = vec![0.1, 0.2, 0.3];
        assert_eq!(resample_linear(&v, 16000, 16000), v);
    }

    // ─── #600 チャンク分割計画 ───

    #[test]
    fn chunk_plan_empty_audio_is_empty() {
        assert!(chunk_plan(0, 16000, 24.0, 2.0).is_empty());
    }

    #[test]
    fn chunk_plan_short_audio_is_single_chunk_owning_all() {
        // 24秒以下は単一チャンク＝従来動作（短尺は無回帰）。
        let total = 16000 * 10; // 10秒
        let specs = chunk_plan(total, 16000, 24.0, 2.0);
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].start, 0);
        assert_eq!(specs[0].end, total);
        assert_eq!(specs[0].own_start_cs, 0);
        assert_eq!(specs[0].own_end_cs, i64::MAX);
    }

    #[test]
    fn chunk_plan_long_audio_splits_with_stride_and_overlap() {
        // 60秒 / chunk=24s overlap=2s → stride=22s。開始: 0, 22, 44(→60で終端)。
        let sr = 16000;
        let total = sr as usize * 60;
        let specs = chunk_plan(total, sr, 24.0, 2.0);
        assert_eq!(specs.len(), 3, "0-24 / 22-46 / 44-60 の3チャンク");
        assert_eq!(specs[0].start, 0);
        assert_eq!(specs[1].start, sr as usize * 22);
        assert_eq!(specs[2].start, sr as usize * 44);
        // 各チャンクは chunk 長 or 末尾まで。
        assert_eq!(specs[0].end, sr as usize * 24);
        assert_eq!(specs[2].end, total, "最終チャンクは末尾まで（末尾を落とさない）");
    }

    #[test]
    fn chunk_plan_ownership_windows_are_contiguous_and_cover_all() {
        // 担当区間は隙間なく連続し、[0, MAX) を覆う（重複も欠落もしない）。
        let sr = 16000;
        let specs = chunk_plan(sr as usize * 60, sr, 24.0, 2.0);
        assert_eq!(specs[0].own_start_cs, 0);
        for w in specs.windows(2) {
            assert_eq!(
                w[0].own_end_cs, w[1].own_start_cs,
                "隣接担当区間は境界を共有（重複/欠落なし）"
            );
        }
        assert_eq!(specs.last().unwrap().own_end_cs, i64::MAX);
        // overlap中点で切れている: chunk1 は offset(22s=2200cs)+overlap/2(100cs)=2300cs から担当。
        assert_eq!(specs[1].own_start_cs, 2200 + 100);
    }

    #[test]
    fn chunk_plan_pathological_overlap_does_not_explode() {
        // overlap>=chunk の病的入力でも、overlap は chunk/2 に制限され stride>=chunk/2 を保つ。
        // チャンク数は ~2*total/chunk 程度に収まり、爆発しない（レビュー指摘 Finding3）。
        let sr = 16000;
        let total = sr as usize * 120; // 120秒
        let specs = chunk_plan(total, sr, 24.0, 30.0); // overlap>chunk
        assert!(
            specs.len() <= 12,
            "stride>=chunk/2(12s) に制限され爆発しない: {} chunks",
            specs.len()
        );
        // 担当区間は依然として連続（重複/欠落なし）。
        for w in specs.windows(2) {
            assert_eq!(w[0].own_end_cs, w[1].own_start_cs);
        }
        assert_eq!(specs.last().unwrap().own_end_cs, i64::MAX);
    }

    #[test]
    fn chunk_plan_owned_region_ends_before_chunk_audio_tail() {
        // 各チャンクの担当終端は、自分の音声末尾から overlap/2 以上手前にある
        // （末尾帯を担当しない＝whisperの末尾ドロップに強い / レビュー指摘 Finding1）。
        let sr = 16000;
        let specs = chunk_plan(sr as usize * 120, sr, 24.0, 4.0);
        let half_overlap_cs = 200; // overlap4s/2=2s=200cs
        for spec in specs.iter().take(specs.len() - 1) {
            let audio_end_cs = ((spec.end as f64 / sr as f64) * 100.0).round() as i64;
            assert!(
                spec.own_end_cs <= audio_end_cs - half_overlap_cs,
                "担当終端 {} は音声末尾 {} より overlap/2({}) 以上手前",
                spec.own_end_cs,
                audio_end_cs,
                half_overlap_cs
            );
        }
    }

    #[test]
    fn chunk_plan_zero_overlap_boundaries_at_offsets() {
        let sr = 16000;
        let specs = chunk_plan(sr as usize * 50, sr, 24.0, 0.0);
        // overlap=0 → stride=24s。担当境界はチャンク先頭時刻に一致。
        for w in specs.windows(2) {
            assert_eq!(w[0].own_end_cs, w[1].offset_cs);
        }
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
            use_gpu: false,
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
    fn stt_provider_parses_aliases_and_defaults_local() {
        assert_eq!(SttProvider::parse("groq"), SttProvider::Groq);
        assert_eq!(SttProvider::parse("OpenAI"), SttProvider::OpenAi);
        assert_eq!(SttProvider::parse("deepgram"), SttProvider::Deepgram);
        assert_eq!(SttProvider::parse("azure"), SttProvider::Azure);
        // 空/未知/local/whisper はローカル（プライバシー既定）へフォールバック。
        assert_eq!(SttProvider::parse(""), SttProvider::Local);
        assert_eq!(SttProvider::parse("local"), SttProvider::Local);
        assert_eq!(SttProvider::parse("whisper"), SttProvider::Local);
        assert_eq!(SttProvider::parse("unknown"), SttProvider::Local);
    }

    #[test]
    fn stt_provider_is_cloud_true_except_local() {
        assert!(!SttProvider::Local.is_cloud());
        for p in [
            SttProvider::Groq,
            SttProvider::OpenAi,
            SttProvider::Deepgram,
            SttProvider::Azure,
        ] {
            assert!(p.is_cloud());
        }
    }

    #[test]
    fn stt_provider_default_models() {
        assert_eq!(SttProvider::Groq.default_model(), "whisper-large-v3-turbo");
        assert_eq!(SttProvider::OpenAi.default_model(), "gpt-4o-transcribe");
        assert_eq!(SttProvider::Deepgram.default_model(), "nova-3");
        assert_eq!(SttProvider::Azure.default_model(), "");
        assert_eq!(SttProvider::Local.default_model(), "");
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

    // ─── ここから クラウドSTT/デコードの経路テスト（ローカルテストサーバ / 監査項目12） ───
    use crate::testhttp::{serve, set_envs, Route};
    use std::sync::{Arc, Mutex};

    /// エンジンを呼び、(結果, 進捗履歴, セグメント履歴) を返す。
    fn run_engine(
        engine: &dyn TranscriptionEngine,
        audio: &[f32],
    ) -> (Result<String, String>, Vec<i32>, Vec<String>) {
        let progress = Arc::new(Mutex::new(Vec::new()));
        let segments = Arc::new(Mutex::new(Vec::new()));
        let (p, s) = (progress.clone(), segments.clone());
        let r = engine.transcribe(
            audio,
            Some("ja"),
            false,
            Box::new(move |pct| p.lock().unwrap().push(pct)),
            Box::new(move |seg| s.lock().unwrap().push(seg)),
        );
        let progress = progress.lock().unwrap().clone();
        let segments = segments.lock().unwrap().clone();
        (r, progress, segments)
    }

    #[test]
    fn openai_compatible_engine_http_paths() {
        // 鍵なしは即エラー。
        let engine = OpenAiCompatibleSttEngine {
            base_url: "http://127.0.0.1:9".into(),
            model: "m".into(),
            api_key: " ".into(),
        };
        let (r, _, _) = run_engine(&engine, &[0.0; 160]);
        assert_eq!(r.unwrap_err(), crate::errcode::E_CLOUD_STT_NO_KEY);
        // 成功: multipart POST → {text} 抽出。進捗は 5→30→100、本文はセグメント通知。
        let (base, seen) = serve(vec![Route::json(
            "/audio/transcriptions",
            200,
            r#"{"text":" こんにちは "}"#,
        )]);
        let engine = OpenAiCompatibleSttEngine {
            base_url: base.clone(),
            model: "whisper-x".into(),
            api_key: "gk".into(),
        };
        let (r, progress, segments) = run_engine(&engine, &[0.1; 160]);
        assert_eq!(r.unwrap(), "こんにちは");
        assert_eq!(progress, vec![5, 30, 100]);
        assert_eq!(segments, vec!["こんにちは".to_string()]);
        let req = seen.lock().unwrap()[0].clone();
        assert!(req.to_ascii_lowercase().contains("authorization: bearer gk"), "{req}");
        assert!(req.contains("name=\"model\"") && req.contains("whisper-x"), "モデルをmultipartで送る");
        assert!(req.contains("name=\"language\"") && req.contains("ja"), "言語を送る");
        // 非2xxは E_CLOUD_STT_STATUS（本文の先頭を詳細に含む）。
        let (base2, _) = serve(vec![Route::json("/audio/transcriptions", 401, r#"{"error":"bad key"}"#)]);
        let engine = OpenAiCompatibleSttEngine {
            base_url: base2,
            model: "m".into(),
            api_key: "k".into(),
        };
        let (r, _, _) = run_engine(&engine, &[0.0; 16]);
        let err = r.unwrap_err();
        assert!(err.starts_with(crate::errcode::E_CLOUD_STT_STATUS), "{err}");
        assert!(err.contains("401") && err.contains("bad key"), "{err}");
        // 200 かつ 非JSONは E_CLOUD_STT_PARSE。
        let (base3, _) = serve(vec![Route::json("/audio/transcriptions", 200, "not-json")]);
        let engine = OpenAiCompatibleSttEngine {
            base_url: base3,
            model: "m".into(),
            api_key: "k".into(),
        };
        let (r, _, _) = run_engine(&engine, &[0.0; 16]);
        let err = r.unwrap_err();
        assert!(err.starts_with(crate::errcode::E_CLOUD_STT_PARSE), "{err}");
        // 接続不能は E_CLOUD_STT_HTTP。
        let engine = OpenAiCompatibleSttEngine {
            base_url: "http://127.0.0.1:9".into(),
            model: "m".into(),
            api_key: "k".into(),
        };
        let (r, _, _) = run_engine(&engine, &[0.0; 16]);
        let err = r.unwrap_err();
        assert!(err.starts_with(crate::errcode::E_CLOUD_STT_HTTP), "{err}");
    }

    #[test]
    fn deepgram_engine_http_paths() {
        let engine = DeepgramSttEngine {
            model: "nova-3".into(),
            api_key: "".into(),
        };
        let (r, _, _) = run_engine(&engine, &[0.0; 16]);
        assert_eq!(r.unwrap_err(), crate::errcode::E_CLOUD_STT_NO_KEY);
        // 成功: Token 認証・生WAV本文・results からの抽出。
        let (base, seen) = serve(vec![Route::json(
            "/v1/listen",
            200,
            r#"{"results":{"channels":[{"alternatives":[{"transcript":"深層の本文"}]}]}}"#,
        )]);
        {
            let _g = set_envs(&[("QS_TEST_DEEPGRAM_BASE", &base)]);
            let engine = DeepgramSttEngine {
                model: "nova-3".into(),
                api_key: "dk".into(),
            };
            let (r, progress, segments) = run_engine(&engine, &[0.1; 160]);
            assert_eq!(r.unwrap(), "深層の本文");
            assert_eq!(progress, vec![5, 30, 100]);
            assert_eq!(segments, vec!["深層の本文".to_string()]);
            let req = seen.lock().unwrap()[0].clone();
            assert!(req.to_ascii_lowercase().contains("authorization: token dk"), "{req}");
            assert!(req.contains("mip_opt_out=true"), "学習利用オプトアウトを付す: {req}");
            assert!(req.contains("language=ja"), "{req}");
        }
        // transcript 空は空文字を返しセグメント通知しない。
        let (base2, _) = serve(vec![Route::json("/v1/listen", 200, r#"{"results":{}}"#)]);
        {
            let _g = set_envs(&[("QS_TEST_DEEPGRAM_BASE", &base2)]);
            let engine = DeepgramSttEngine {
                model: "nova-3".into(),
                api_key: "dk".into(),
            };
            let (r, _, segments) = run_engine(&engine, &[0.0; 16]);
            assert_eq!(r.unwrap(), "");
            assert!(segments.is_empty());
        }
    }

    #[test]
    fn azure_engine_http_paths() {
        let engine = AzureSttEngine {
            resource: "res".into(),
            api_key: "".into(),
            locale: "ja-JP".into(),
        };
        let (r, _, _) = run_engine(&engine, &[0.0; 16]);
        assert_eq!(r.unwrap_err(), crate::errcode::E_CLOUD_STT_NO_KEY);
        // リソース名なしは安定コード。
        let engine = AzureSttEngine {
            resource: " ".into(),
            api_key: "ak".into(),
            locale: "ja-JP".into(),
        };
        let (r, _, _) = run_engine(&engine, &[0.0; 16]);
        assert_eq!(r.unwrap_err(), crate::errcode::E_AZURE_NO_RESOURCE);
        // 成功: Ocp-Apim-Subscription-Key 認証・definition(locale) を multipart で送る。
        let (base, seen) = serve(vec![Route::json(
            "transcriptions:transcribe",
            200,
            r#"{"combinedPhrases":[{"text":"アジュールの本文"}]}"#,
        )]);
        {
            let _g = set_envs(&[("QS_TEST_AZURE_BASE", &base)]);
            let engine = AzureSttEngine {
                resource: "myres".into(),
                api_key: "ak".into(),
                locale: "ja-JP".into(),
            };
            let (r, progress, segments) = run_engine(&engine, &[0.1; 160]);
            assert_eq!(r.unwrap(), "アジュールの本文");
            assert_eq!(progress, vec![5, 30, 100]);
            assert_eq!(segments, vec!["アジュールの本文".to_string()]);
            let req = seen.lock().unwrap()[0].clone();
            assert!(req.to_ascii_lowercase().contains("ocp-apim-subscription-key: ak"), "{req}");
            assert!(req.contains("ja-JP"), "locale を definition で送る: {req}");
        }
    }

    #[test]
    fn build_multipart_layout() {
        let (ct, body) = build_multipart(&[("model", "m1")], "file", "audio.wav", "audio/wav", b"AB");
        assert!(ct.starts_with("multipart/form-data; boundary="));
        let s = String::from_utf8_lossy(&body);
        assert!(s.contains("name=\"model\"") && s.contains("m1"));
        assert!(s.contains("filename=\"audio.wav\"") && s.contains("Content-Type: audio/wav"));
        assert!(s.contains("AB"));
        assert!(s.trim_end().ends_with("--"), "終端バウンダリで閉じる");
    }

    #[test]
    fn decode_missing_file_errors() {
        assert!(decode_to_16k_mono(Path::new("no-such-file.mp3")).is_err());
        assert!(decode_to_16k_mono(Path::new("no-such-file.opus")).is_err());
    }

    #[test]
    fn decode_garbage_file_reports_probe_error() {
        let path = std::env::temp_dir().join(format!("qs_garbage_{}.mp3", std::process::id()));
        std::fs::write(&path, b"this is not audio at all").unwrap();
        let err = decode_to_16k_mono(&path).unwrap_err();
        let _ = std::fs::remove_file(&path);
        assert!(err.starts_with(crate::errcode::E_AUDIO_PROBE), "{err}");
    }

    /// Opus 保存(save_opus) → .opus 自前デコードの往復（S1.6 / #560 回帰ガード）。
    #[test]
    fn opus_roundtrip_decodes_saved_recording() {
        let dir = std::env::temp_dir().join(format!("qs_opus_rt_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        // 0.5秒 48kHz mono の 440Hz 正弦波。
        let pcm: Vec<f32> = (0..24_000)
            .map(|i| (i as f32 * 440.0 * 2.0 * std::f32::consts::PI / 48_000.0).sin() * 0.5)
            .collect();
        let path = crate::audio_save::save_opus(&pcm, 48_000, 1, &dir, "rt").unwrap();
        let out = decode_to_16k_mono(&path).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        // 48k→16k で約1/3（プリスキップ等の誤差は許容）。
        assert!((6000..10000).contains(&out.len()), "想定外の長さ: {}", out.len());
        assert!(out.iter().any(|&s| s.abs() > 0.05), "無音でない");
    }
}
