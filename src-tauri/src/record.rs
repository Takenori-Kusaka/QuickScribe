// マイク録音(AudioCaptureService の実装 / S1.1)。
// cpal で既定の入力デバイスから録音し、停止時に 16kHz mono f32 へ変換して
// whisper(stt) へ渡す。cpal::Stream は Send でないため、キャプチャ専用スレッド内で
// 構築・駆動し、収集バッファ(Arc<Mutex<Vec<f32>>>)と停止フラグを共有する。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::stt::{resample_linear, WHISPER_SR};

/// 単一ソースのキャプチャ（停止フラグ・収集バッファ・キャプチャスレッド・入力仕様）。
struct Capture {
    stop: Arc<AtomicBool>,
    samples: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
    channels: u16,
    join: JoinHandle<()>,
}

impl Capture {
    /// 停止してスレッドをjoinし、文字起こし用 16kHz mono を返す（収集生データも返す）。
    fn finish(self) -> Result<(Vec<f32>, u32, u16), String> {
        self.stop.store(true, Ordering::Relaxed);
        let _ = self.join.join();
        let raw = self
            .samples
            .lock()
            .map_err(|_| crate::errcode::E_REC_BUFFER.to_string())?
            .clone();
        Ok((raw, self.sample_rate, self.channels))
    }
}

/// 録音中のハンドル。1ソース（マイク/ループバック）または複数ソース（ミックス）を保持する。
pub struct Recording {
    captures: Vec<Capture>,
}

/// Tauri 管理状態。録音は単一インスタンスのみ保持する。
#[derive(Default)]
pub struct RecorderState {
    pub current: Mutex<Option<Recording>>,
}

/// 録音停止後の音声データ。文字起こし用(16kHz mono)と保存用(原音)を併せ持つ。
pub struct Recorded {
    /// 文字起こし用に 16kHz mono へ変換済みの音声。
    pub mono16k: Vec<f32>,
    /// 保存用の原音（インターリーブ f32・原サンプルレート/チャンネル）。
    pub raw: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}

impl Recording {
    /// 録音を停止し、文字起こし用(16kHz mono)と保存用(原音)の音声を返す。
    /// 複数ソース（ミックス）の場合は各々を16k monoへ変換してから加算合成する（S1.3 Phase1）。
    pub fn finish(self) -> Result<Recorded, String> {
        let mut parts: Vec<(Vec<f32>, u32, u16)> = Vec::new();
        for cap in self.captures {
            parts.push(cap.finish()?);
        }
        // 単一ソース: 従来どおり原音を保存用に保持（回帰不変・R5）。
        if parts.len() == 1 {
            let (raw, sample_rate, channels) = parts.pop().unwrap();
            let mono16k = to_mono_16k(&raw, sample_rate, channels);
            return Ok(Recorded {
                mono16k,
                raw,
                sample_rate,
                channels,
            });
        }
        // ミックス: 各ソースを16k monoへ変換し加算合成（R2）。保存用も合成後の16k mono。
        let mono_parts: Vec<Vec<f32>> = parts
            .iter()
            .map(|(raw, sr, ch)| to_mono_16k(raw, *sr, *ch))
            .collect();
        let mono16k = mix_16k(&mono_parts);
        Ok(Recorded {
            raw: mono16k.clone(),
            mono16k,
            sample_rate: WHISPER_SR,
            channels: 1,
        })
    }
}

/// テスト専用: 偽キャプチャから Recording を組み立てる（実デバイス不要で finish/停止経路を検証）。
/// cfg(test) 限定のため公開挙動は不変。
#[cfg(test)]
pub(crate) fn test_recording(parts: Vec<(Vec<f32>, u32, u16)>) -> Recording {
    Recording {
        captures: parts
            .into_iter()
            .map(|(data, sample_rate, channels)| Capture {
                stop: Arc::new(AtomicBool::new(false)),
                samples: Arc::new(Mutex::new(data)),
                sample_rate,
                channels,
                join: std::thread::spawn(|| {}),
            })
            .collect(),
    }
}

/// 複数の 16kHz mono ストリームを加算合成し [-1,1] にクリップする（S1.3 Phase1 / 純粋）。
/// 長さは最長に合わせ、短いストリームは無音として扱う（片側欠損でも他方は残る・R3）。
fn mix_16k(parts: &[Vec<f32>]) -> Vec<f32> {
    let len = parts.iter().map(|p| p.len()).max().unwrap_or(0);
    let mut out = vec![0.0f32; len];
    for p in parts {
        for (i, &s) in p.iter().enumerate() {
            out[i] += s;
        }
    }
    for s in out.iter_mut() {
        *s = s.clamp(-1.0, 1.0);
    }
    out
}

/// 録音ソース(マイク入力 / 出力デバイスのループバック)の記述子(S1.3 / ADR-0013)。
/// フロントの統一デバイス選択UIへ種別ラベル付きで一列に並べる。
#[derive(serde::Serialize, Clone)]
pub struct AudioSource {
    /// 入力=デバイス名、ループバック=レンダーデバイスID。空文字は「OS既定」を意味する。
    pub id: String,
    /// 表示名。
    pub label: String,
    /// "input"(マイク) | "loopback"(システム音)。
    pub kind: String,
}

/// 録音ソースを列挙する(S1.2/S1.3)。マイク入力＋(Windowsのみ)出力デバイスのループバック。
pub fn list_audio_sources() -> Result<Vec<AudioSource>, String> {
    let mut out: Vec<AudioSource> = Vec::new();
    let host = cpal::default_host();
    if let Ok(devices) = host.input_devices() {
        let mut seen: Vec<String> = Vec::new();
        for d in devices {
            // cpal0.18: DeviceTrait::name() は廃止、Display で名前を得る。
            let name = d.to_string();
            if seen.contains(&name) {
                continue;
            }
            seen.push(name.clone());
            out.push(AudioSource {
                id: name.clone(),
                label: name,
                kind: "input".into(),
            });
        }
    }
    #[cfg(windows)]
    out.extend(list_loopback_sources());
    Ok(out)
}

/// Windows: 出力(レンダー)デバイスをループバック録音ソースとして列挙する。
/// COMアパートメントを確定させるため専用スレッド(MTA)で実行する
/// (Tauriワーカースレッドはアパートメント未確定のことがあるため)。
#[cfg(windows)]
fn list_loopback_sources() -> Vec<AudioSource> {
    std::thread::spawn(|| {
        use wasapi::{Direction, DeviceEnumerator};
        let mut out = Vec::new();
        let _ = wasapi::initialize_mta().ok();
        let enumerator = match DeviceEnumerator::new() {
            Ok(e) => e,
            Err(_) => return out,
        };
        let collection = match enumerator.get_device_collection(&Direction::Render) {
            Ok(c) => c,
            Err(_) => return out,
        };
        let n = collection.get_nbr_devices().unwrap_or(0);
        for i in 0..n {
            if let Ok(dev) = collection.get_device_at_index(i) {
                let id = dev.get_id().unwrap_or_default();
                if id.is_empty() {
                    continue;
                }
                let name = dev
                    .get_friendlyname()
                    .unwrap_or_else(|_| "出力デバイス".into());
                out.push(AudioSource {
                    id,
                    label: format!("システム音: {name}"),
                    kind: "loopback".into(),
                });
            }
        }
        out
    })
    .join()
    .unwrap_or_default()
}

/// 録音を開始する。kind により録音ソースを切り替える:
/// "loopback"=出力デバイスのループバック(Windows) / "mix"=既定マイク＋システム音 同時取得(Windows) /
/// それ以外=マイク入力。device は入力=デバイス名 / ループバック=レンダーデバイスID。
pub fn start(device: Option<String>, kind: Option<String>) -> Result<Recording, String> {
    match kind.as_deref() {
        Some("loopback") => {
            #[cfg(windows)]
            {
                Ok(Recording {
                    captures: vec![start_loopback(device)?],
                })
            }
            #[cfg(not(windows))]
            {
                let _ = device;
                Err(crate::errcode::E_LOOPBACK_UNSUPPORTED.into())
            }
        }
        Some("mix") => {
            #[cfg(windows)]
            {
                // 既定マイク＋指定/既定の出力デバイスを同時取得（S1.3 Phase1）。
                let mic = start_input(None)?;
                let sys = start_loopback(device)?;
                Ok(Recording {
                    captures: vec![mic, sys],
                })
            }
            #[cfg(not(windows))]
            {
                let _ = device;
                Err(crate::errcode::E_MIXED_UNSUPPORTED.into())
            }
        }
        _ => Ok(Recording {
            captures: vec![start_input(device)?],
        }),
    }
}

/// マイク入力のキャプチャを開始する。device_name 指定時はその入力デバイス(無ければ既定にフォールバック)。
fn start_input(device_name: Option<String>) -> Result<Capture, String> {
    let stop = Arc::new(AtomicBool::new(false));
    let samples = Arc::new(Mutex::new(Vec::<f32>::new()));

    // 入力仕様(sample_rate/channels)はキャプチャスレッドからチャネルで親へ返す。
    let (cfg_tx, cfg_rx) = mpsc::channel::<Result<(u32, u16), String>>();
    let stop_t = stop.clone();
    let samples_t = samples.clone();

    let join = std::thread::spawn(move || {
        let host = cpal::default_host();
        // 指定デバイスを名前で探す。未指定/見つからない場合は既定にフォールバック(実行時切替)。
        let picked = device_name
            .filter(|n| !n.trim().is_empty())
            .and_then(|name| {
                host.input_devices()
                    .ok()
                    .and_then(|mut it| it.find(|d| d.to_string() == name))
            })
            .or_else(|| host.default_input_device());
        let device = match picked {
            Some(d) => d,
            None => {
                let _ = cfg_tx.send(Err(crate::errcode::E_NO_INPUT_DEVICE.into()));
                return;
            }
        };
        let config = match device.default_input_config() {
            Ok(c) => c,
            Err(e) => {
                let _ = cfg_tx.send(Err(crate::errcode::ec(crate::errcode::E_INPUT_CONFIG, e)));
                return;
            }
        };
        // cpal0.18: SampleRate は u32 のエイリアスになり .0 は不要。
        let sample_rate = config.sample_rate();
        let channels = config.channels();
        let sample_format = config.sample_format();
        let stream_config: cpal::StreamConfig = config.into();

        let buf = samples_t.clone();
        let err_fn = |e| eprintln!("録音ストリームエラー: {e}");

        // 入力フォーマット別に [-1.0, 1.0] の f32 へ正規化して収集する。
        let stream_res = match sample_format {
            cpal::SampleFormat::F32 => device.build_input_stream(
                stream_config,
                move |data: &[f32], _: &_| {
                    if let Ok(mut b) = buf.lock() {
                        b.extend_from_slice(data);
                    }
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::I16 => device.build_input_stream(
                stream_config,
                move |data: &[i16], _: &_| {
                    if let Ok(mut b) = buf.lock() {
                        b.extend(data.iter().map(|&s| s as f32 / i16::MAX as f32));
                    }
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::U16 => device.build_input_stream(
                stream_config,
                move |data: &[u16], _: &_| {
                    if let Ok(mut b) = buf.lock() {
                        b.extend(
                            data.iter()
                                .map(|&s| (s as f32 / u16::MAX as f32) * 2.0 - 1.0),
                        );
                    }
                },
                err_fn,
                None,
            ),
            fmt => {
                let _ = cfg_tx.send(Err(crate::errcode::ec(crate::errcode::E_UNSUPPORTED_FORMAT, format!("{fmt:?}"))));
                return;
            }
        };

        let stream = match stream_res {
            Ok(s) => s,
            Err(e) => {
                let _ = cfg_tx.send(Err(crate::errcode::ec(crate::errcode::E_STREAM_BUILD, e)));
                return;
            }
        };
        if let Err(e) = stream.play() {
            let _ = cfg_tx.send(Err(crate::errcode::ec(crate::errcode::E_REC_START, e)));
            return;
        }

        // 設定の通知に成功 → 停止フラグが立つまでストリームを生かす。
        let _ = cfg_tx.send(Ok((sample_rate, channels)));
        while !stop_t.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        drop(stream);
    });

    match cfg_rx.recv() {
        Ok(Ok((sample_rate, channels))) => Ok(Capture {
            stop,
            samples,
            sample_rate,
            channels,
            join,
        }),
        Ok(Err(e)) => {
            let _ = join.join();
            Err(e)
        }
        Err(_) => Err(crate::errcode::E_REC_THREAD_INIT.into()),
    }
}

/// Windows: 選択した出力デバイスをループバック録音する(S1.3 / ADR-0013)。
/// レンダーデバイス + Direction::Capture で wasapi が AUDCLNT_STREAMFLAGS_LOOPBACK を設定する。
/// 無音時はパケットが来ないため、wait_for_event をタイムアウト付きにして stop を監視する。
#[cfg(windows)]
fn start_loopback(device_id: Option<String>) -> Result<Capture, String> {
    use std::collections::VecDeque;
    use wasapi::{Direction, DeviceEnumerator, StreamMode};

    let stop = Arc::new(AtomicBool::new(false));
    let samples = Arc::new(Mutex::new(Vec::<f32>::new()));
    let (cfg_tx, cfg_rx) = mpsc::channel::<Result<(u32, u16), String>>();
    let stop_t = stop.clone();
    let samples_t = samples.clone();

    let join = std::thread::spawn(move || {
        if let Err(e) = wasapi::initialize_mta().ok() {
            let _ = cfg_tx.send(Err(crate::errcode::ec(crate::errcode::E_COM_INIT, e)));
            return;
        }
        let enumerator = match DeviceEnumerator::new() {
            Ok(e) => e,
            Err(e) => {
                let _ = cfg_tx.send(Err(crate::errcode::ec(crate::errcode::E_AUDIO_ENUM, e)));
                return;
            }
        };
        // 指定IDのレンダーデバイス、無ければ既定の出力にフォールバック。
        let device = match device_id.filter(|s| !s.trim().is_empty()) {
            Some(id) => enumerator
                .get_device(&id)
                .or_else(|_| enumerator.get_default_device(&Direction::Render)),
            None => enumerator.get_default_device(&Direction::Render),
        };
        let device = match device {
            Ok(d) => d,
            Err(e) => {
                let _ = cfg_tx.send(Err(crate::errcode::ec(crate::errcode::E_OUTPUT_DEVICE, e)));
                return;
            }
        };
        let mut audio_client = match device.get_iaudioclient() {
            Ok(c) => c,
            Err(e) => {
                let _ = cfg_tx.send(Err(crate::errcode::ec(crate::errcode::E_AUDIO_CLIENT, e)));
                return;
            }
        };
        let format = match audio_client.get_mixformat() {
            Ok(f) => f,
            Err(e) => {
                let _ = cfg_tx.send(Err(crate::errcode::ec(crate::errcode::E_MIX_FORMAT, e)));
                return;
            }
        };
        let sample_rate = format.get_samplespersec();
        let channels = format.get_nchannels();
        let bits = format.get_bitspersample();
        let (def_time, _min_time) = match audio_client.get_device_period() {
            Ok(t) => t,
            Err(e) => {
                let _ = cfg_tx.send(Err(crate::errcode::ec(crate::errcode::E_DEVICE_PERIOD, e)));
                return;
            }
        };
        // レンダーデバイスを Capture 方向で初期化 → ループバック。共有モード必須。
        let mode = StreamMode::EventsShared {
            autoconvert: true,
            buffer_duration_hns: def_time,
        };
        if let Err(e) = audio_client.initialize_client(&format, &Direction::Capture, &mode) {
            let _ = cfg_tx.send(Err(crate::errcode::ec(crate::errcode::E_LOOPBACK_INIT, e)));
            return;
        }
        let h_event = match audio_client.set_get_eventhandle() {
            Ok(h) => h,
            Err(e) => {
                let _ = cfg_tx.send(Err(crate::errcode::ec(crate::errcode::E_EVENT_HANDLE, e)));
                return;
            }
        };
        let capture_client = match audio_client.get_audiocaptureclient() {
            Ok(c) => c,
            Err(e) => {
                let _ = cfg_tx.send(Err(crate::errcode::ec(crate::errcode::E_CAPTURE_CLIENT, e)));
                return;
            }
        };
        if let Err(e) = audio_client.start_stream() {
            let _ = cfg_tx.send(Err(crate::errcode::ec(crate::errcode::E_LOOPBACK_START, e)));
            return;
        }

        // 設定通知に成功 → 停止フラグが立つまでキャプチャ。
        let _ = cfg_tx.send(Ok((sample_rate, channels)));
        let mut queue: VecDeque<u8> = VecDeque::new();
        while !stop_t.load(Ordering::Relaxed) {
            // 無音時はイベントが来ないので200msでタイムアウトして stop を再評価。
            if h_event.wait_for_event(200).is_err() {
                continue;
            }
            if capture_client.read_from_device_to_deque(&mut queue).is_err() {
                continue;
            }
            if let Ok(mut buf) = samples_t.lock() {
                push_pcm_bytes_as_f32(&mut queue, bits, &mut buf);
            }
        }
        let _ = audio_client.stop_stream();
    });

    match cfg_rx.recv() {
        Ok(Ok((sample_rate, channels))) => Ok(Capture {
            stop,
            samples,
            sample_rate,
            channels,
            join,
        }),
        Ok(Err(e)) => {
            let _ = join.join();
            Err(e)
        }
        Err(_) => Err(crate::errcode::E_LOOPBACK_THREAD_INIT.into()),
    }
}

/// WASAPIキャプチャの生バイト列を f32 サンプルへ変換して追記する。
/// 共有モードのミックスフォーマットは通常 32bit float。16bit PCM もフォールバック対応。
#[cfg(windows)]
fn push_pcm_bytes_as_f32(queue: &mut std::collections::VecDeque<u8>, bits: u16, out: &mut Vec<f32>) {
    if bits == 16 {
        while queue.len() >= 2 {
            let lo = queue.pop_front().unwrap();
            let hi = queue.pop_front().unwrap();
            let s = i16::from_le_bytes([lo, hi]);
            out.push(s as f32 / i16::MAX as f32);
        }
    } else {
        // 既定: 32bit float little-endian。
        while queue.len() >= 4 {
            let b0 = queue.pop_front().unwrap();
            let b1 = queue.pop_front().unwrap();
            let b2 = queue.pop_front().unwrap();
            let b3 = queue.pop_front().unwrap();
            out.push(f32::from_le_bytes([b0, b1, b2, b3]));
        }
    }
}

/// インターリーブ済み f32 をモノラル化し 16kHz へリサンプルする（純粋関数・テスト対象 S7.1）。
pub fn to_mono_16k(interleaved: &[f32], sample_rate: u32, channels: u16) -> Vec<f32> {
    let ch = channels.max(1) as usize;
    let mono: Vec<f32> = if ch <= 1 {
        interleaved.to_vec()
    } else {
        interleaved
            .chunks(ch)
            .map(|frame| frame.iter().copied().sum::<f32>() / ch as f32)
            .collect()
    };
    resample_linear(&mono, sample_rate, WHISPER_SR)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mono_16k_passthrough_when_already_16k_mono() {
        let v = vec![0.1, 0.2, 0.3];
        assert_eq!(to_mono_16k(&v, WHISPER_SR, 1), v);
    }

    #[test]
    fn stereo_is_averaged_to_mono() {
        // L=0.0,R=1.0 が交互 → モノは 0.5 が並ぶ（16kならリサンプル無し）。
        let interleaved = vec![0.0, 1.0, 0.0, 1.0];
        let out = to_mono_16k(&interleaved, WHISPER_SR, 2);
        assert_eq!(out, vec![0.5, 0.5]);
    }

    #[test]
    fn downsample_halves_stereo_length() {
        // 32kHz ステレオ 100フレーム(200サンプル) → 16kモノは約50。
        let interleaved: Vec<f32> = (0..200).map(|i| i as f32).collect();
        let out = to_mono_16k(&interleaved, 32000, 2);
        assert!((out.len() as i32 - 50).abs() <= 1);
    }

    // 浮動小数点の近似一致を検証する（加算で末尾ビットがずれるため厳密等価は使わない）。
    fn assert_close(actual: &[f32], expected: &[f32]) {
        assert_eq!(actual.len(), expected.len(), "長さ不一致");
        for (i, (a, e)) in actual.iter().zip(expected).enumerate() {
            assert!((a - e).abs() < 1e-6, "index {i}: {a} != {e}");
        }
    }

    #[test]
    fn mix_sums_same_length_streams() {
        let a = vec![0.1, 0.2, -0.3];
        let b = vec![0.2, 0.2, 0.1];
        assert_close(&mix_16k(&[a, b]), &[0.3, 0.4, -0.2]);
    }

    #[test]
    fn mix_pads_shorter_stream_with_silence() {
        // 長さ違いは最長に合わせ、短い側は無音(0)として扱う（三角測量・R3）。
        let a = vec![0.5, 0.5, 0.5];
        let b = vec![0.1];
        assert_close(&mix_16k(&[a, b]), &[0.6, 0.5, 0.5]);
    }

    #[test]
    fn mix_clips_to_unit_range() {
        let a = vec![0.8, -0.8];
        let b = vec![0.5, -0.5];
        assert_close(&mix_16k(&[a, b]), &[1.0, -1.0]);
    }

    #[test]
    fn mix_single_stream_is_identity() {
        let a = vec![0.1, -0.2, 0.3];
        assert_close(&mix_16k(&[a.clone()]), &a);
    }

    #[test]
    fn finish_single_capture_keeps_raw_and_converts_mono16k() {
        // 32kHz ステレオの原音を保持しつつ、16k mono へ変換する（保存用と文字起こし用の分離）。
        let raw: Vec<f32> = (0..200).map(|i| (i % 7) as f32 / 10.0).collect();
        let rec = test_recording(vec![(raw.clone(), 32000, 2)]);
        let out = rec.finish().unwrap();
        assert_eq!(out.raw, raw, "原音は無変換で保持");
        assert_eq!(out.sample_rate, 32000);
        assert_eq!(out.channels, 2);
        assert!((out.mono16k.len() as i32 - 50).abs() <= 1, "16k mono へ変換");
    }

    #[test]
    fn finish_multi_capture_mixes_to_16k_mono() {
        // ミックス（マイク＋システム音）は各々16k monoへ変換後に加算合成し、保存用も合成音。
        let a = vec![0.25; 160]; // 16kHz mono
        let b = vec![0.25; 320]; // 32kHz mono → 16k で 160
        let rec = test_recording(vec![(a, WHISPER_SR, 1), (b, 32000, 1)]);
        let out = rec.finish().unwrap();
        assert_eq!(out.sample_rate, WHISPER_SR);
        assert_eq!(out.channels, 1);
        assert_eq!(out.raw, out.mono16k, "ミックス時は保存用も合成後16k mono");
        assert!((out.mono16k.len() as i32 - 160).abs() <= 1);
        assert!((out.mono16k[10] - 0.5).abs() < 1e-3, "加算合成されている");
    }

    #[test]
    fn list_audio_sources_does_not_panic() {
        // デバイス構成に依らず総当たりで列挙できる（CIのヘッドレス環境では空でもよい）。
        let sources = list_audio_sources().unwrap();
        for s in &sources {
            assert!(!s.label.is_empty());
            assert!(s.kind == "input" || s.kind == "loopback");
        }
    }

    #[cfg(not(windows))]
    #[test]
    fn loopback_and_mix_are_unsupported_off_windows() {
        assert_eq!(
            start(None, Some("loopback".into())).err().unwrap(),
            crate::errcode::E_LOOPBACK_UNSUPPORTED
        );
        assert_eq!(
            start(None, Some("mix".into())).err().unwrap(),
            crate::errcode::E_MIXED_UNSUPPORTED
        );
    }

    #[test]
    fn start_input_falls_back_or_fails_cleanly() {
        // 存在しないデバイス名は既定へフォールバック。デバイスが無い環境（CI）は
        // 安定コードのエラーになる。いずれもパニックせず、成功時は即停止できる。
        match start(Some("qs-no-such-device".into()), None) {
            Ok(rec) => {
                let _ = rec.finish();
            }
            Err(e) => assert!(!e.is_empty(), "エラーは安定コードを持つ: {e}"),
        }
    }
}
