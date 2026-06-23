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

/// 録音中のハンドル（停止フラグ・収集バッファ・キャプチャスレッド・入力仕様）。
pub struct Recording {
    stop: Arc<AtomicBool>,
    samples: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
    channels: u16,
    join: JoinHandle<()>,
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
    pub fn finish(self) -> Result<Recorded, String> {
        self.stop.store(true, Ordering::Relaxed);
        let _ = self.join.join();
        let raw = self
            .samples
            .lock()
            .map_err(|_| "録音バッファの取得に失敗".to_string())?
            .clone();
        let mono16k = to_mono_16k(&raw, self.sample_rate, self.channels);
        Ok(Recorded {
            mono16k,
            raw,
            sample_rate: self.sample_rate,
            channels: self.channels,
        })
    }
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
            if let Ok(name) = d.name() {
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

/// 録音を開始する。kind="loopback" なら出力デバイスのループバック(Windowsのみ)、
/// それ以外はマイク入力。device は入力=デバイス名 / ループバック=レンダーデバイスID。
pub fn start(device: Option<String>, kind: Option<String>) -> Result<Recording, String> {
    if kind.as_deref() == Some("loopback") {
        #[cfg(windows)]
        {
            return start_loopback(device);
        }
        #[cfg(not(windows))]
        {
            let _ = device;
            return Err("システム音(ループバック)はこのプラットフォームでは未対応です".into());
        }
    }
    start_input(device)
}

/// マイク入力から録音を開始する。device_name 指定時はその入力デバイス(無ければ既定にフォールバック)。
fn start_input(device_name: Option<String>) -> Result<Recording, String> {
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
                    .and_then(|mut it| it.find(|d| d.name().map(|n| n == name).unwrap_or(false)))
            })
            .or_else(|| host.default_input_device());
        let device = match picked {
            Some(d) => d,
            None => {
                let _ = cfg_tx.send(Err("マイク(入力デバイス)が見つかりません".into()));
                return;
            }
        };
        let config = match device.default_input_config() {
            Ok(c) => c,
            Err(e) => {
                let _ = cfg_tx.send(Err(format!("入力デバイス設定の取得に失敗: {e}")));
                return;
            }
        };
        let sample_rate = config.sample_rate().0;
        let channels = config.channels();
        let sample_format = config.sample_format();
        let stream_config: cpal::StreamConfig = config.into();

        let buf = samples_t.clone();
        let err_fn = |e| eprintln!("録音ストリームエラー: {e}");

        // 入力フォーマット別に [-1.0, 1.0] の f32 へ正規化して収集する。
        let stream_res = match sample_format {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &stream_config,
                move |data: &[f32], _: &_| {
                    if let Ok(mut b) = buf.lock() {
                        b.extend_from_slice(data);
                    }
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::I16 => device.build_input_stream(
                &stream_config,
                move |data: &[i16], _: &_| {
                    if let Ok(mut b) = buf.lock() {
                        b.extend(data.iter().map(|&s| s as f32 / i16::MAX as f32));
                    }
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::U16 => device.build_input_stream(
                &stream_config,
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
                let _ = cfg_tx.send(Err(format!("未対応の入力音声フォーマット: {fmt:?}")));
                return;
            }
        };

        let stream = match stream_res {
            Ok(s) => s,
            Err(e) => {
                let _ = cfg_tx.send(Err(format!("録音ストリーム生成に失敗: {e}")));
                return;
            }
        };
        if let Err(e) = stream.play() {
            let _ = cfg_tx.send(Err(format!("録音開始に失敗: {e}")));
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
        Ok(Ok((sample_rate, channels))) => Ok(Recording {
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
        Err(_) => Err("録音スレッドの初期化に失敗しました".into()),
    }
}

/// Windows: 選択した出力デバイスをループバック録音する(S1.3 / ADR-0013)。
/// レンダーデバイス + Direction::Capture で wasapi が AUDCLNT_STREAMFLAGS_LOOPBACK を設定する。
/// 無音時はパケットが来ないため、wait_for_event をタイムアウト付きにして stop を監視する。
#[cfg(windows)]
fn start_loopback(device_id: Option<String>) -> Result<Recording, String> {
    use std::collections::VecDeque;
    use wasapi::{Direction, DeviceEnumerator, StreamMode};

    let stop = Arc::new(AtomicBool::new(false));
    let samples = Arc::new(Mutex::new(Vec::<f32>::new()));
    let (cfg_tx, cfg_rx) = mpsc::channel::<Result<(u32, u16), String>>();
    let stop_t = stop.clone();
    let samples_t = samples.clone();

    let join = std::thread::spawn(move || {
        if let Err(e) = wasapi::initialize_mta().ok() {
            let _ = cfg_tx.send(Err(format!("COM初期化に失敗: {e}")));
            return;
        }
        let enumerator = match DeviceEnumerator::new() {
            Ok(e) => e,
            Err(e) => {
                let _ = cfg_tx.send(Err(format!("オーディオ列挙の初期化に失敗: {e}")));
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
                let _ = cfg_tx.send(Err(format!("出力デバイスの取得に失敗: {e}")));
                return;
            }
        };
        let mut audio_client = match device.get_iaudioclient() {
            Ok(c) => c,
            Err(e) => {
                let _ = cfg_tx.send(Err(format!("オーディオクライアント取得に失敗: {e}")));
                return;
            }
        };
        let format = match audio_client.get_mixformat() {
            Ok(f) => f,
            Err(e) => {
                let _ = cfg_tx.send(Err(format!("ミックスフォーマット取得に失敗: {e}")));
                return;
            }
        };
        let sample_rate = format.get_samplespersec();
        let channels = format.get_nchannels();
        let bits = format.get_bitspersample();
        let (def_time, _min_time) = match audio_client.get_device_period() {
            Ok(t) => t,
            Err(e) => {
                let _ = cfg_tx.send(Err(format!("デバイス周期の取得に失敗: {e}")));
                return;
            }
        };
        // レンダーデバイスを Capture 方向で初期化 → ループバック。共有モード必須。
        let mode = StreamMode::EventsShared {
            autoconvert: true,
            buffer_duration_hns: def_time,
        };
        if let Err(e) = audio_client.initialize_client(&format, &Direction::Capture, &mode) {
            let _ = cfg_tx.send(Err(format!("ループバック初期化に失敗: {e}")));
            return;
        }
        let h_event = match audio_client.set_get_eventhandle() {
            Ok(h) => h,
            Err(e) => {
                let _ = cfg_tx.send(Err(format!("イベントハンドル取得に失敗: {e}")));
                return;
            }
        };
        let capture_client = match audio_client.get_audiocaptureclient() {
            Ok(c) => c,
            Err(e) => {
                let _ = cfg_tx.send(Err(format!("キャプチャクライアント取得に失敗: {e}")));
                return;
            }
        };
        if let Err(e) = audio_client.start_stream() {
            let _ = cfg_tx.send(Err(format!("ループバック録音開始に失敗: {e}")));
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
        Ok(Ok((sample_rate, channels))) => Ok(Recording {
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
        Err(_) => Err("ループバック録音スレッドの初期化に失敗しました".into()),
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
}
