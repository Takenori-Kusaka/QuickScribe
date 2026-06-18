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

/// 既定の入力デバイスから録音を開始する。
/// デバイス/設定の取得・ストリーム生成に失敗した場合はエラーを返す。
pub fn start() -> Result<Recording, String> {
    let stop = Arc::new(AtomicBool::new(false));
    let samples = Arc::new(Mutex::new(Vec::<f32>::new()));

    // 入力仕様(sample_rate/channels)はキャプチャスレッドからチャネルで親へ返す。
    let (cfg_tx, cfg_rx) = mpsc::channel::<Result<(u32, u16), String>>();
    let stop_t = stop.clone();
    let samples_t = samples.clone();

    let join = std::thread::spawn(move || {
        let host = cpal::default_host();
        let device = match host.default_input_device() {
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
