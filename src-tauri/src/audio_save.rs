// 録音音声の保存。まずは純Rustの hound で確実な WAV(i16) 保存を提供する。
// モダン圧縮(Opus/.opus = Ogg Opus, 24kbps mono)は後続PRで format="opus" として追加する
// （audio-encode-rust 調査）。保存しない場合は本モジュールを呼ばず完全メモリ完結とする。

use std::path::{Path, PathBuf};

/// f32サンプル[-1,1]を i16 へ量子化する（純粋関数・テスト対象）。
pub fn f32_to_i16(s: f32) -> i16 {
    (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16
}

/// インターリーブ f32 PCM を WAV(i16) として dir 配下に保存し、保存パスを返す。
pub fn save_wav(
    interleaved: &[f32],
    sample_rate: u32,
    channels: u16,
    dir: &Path,
    file_stem: &str,
) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{file_stem}.wav"));
    let spec = hound::WavSpec {
        channels: channels.max(1),
        sample_rate: sample_rate.max(8000),
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer =
        hound::WavWriter::create(&path, spec).map_err(|e| crate::errcode::ec(crate::errcode::E_WAV_CREATE, e))?;
    for &s in interleaved {
        writer
            .write_sample(f32_to_i16(s))
            .map_err(|e| e.to_string())?;
    }
    writer
        .finalize()
        .map_err(|e| crate::errcode::ec(crate::errcode::E_WAV_EXPORT, e))?;
    Ok(path)
}

/// インターリーブ f32 を 48kHz mono へ整える（Opusは48kHz基準 / audio-encode-rust 調査）。
fn to_mono_48k(interleaved: &[f32], sample_rate: u32, channels: u16) -> Vec<f32> {
    let ch = channels.max(1) as usize;
    let mono: Vec<f32> = if ch <= 1 {
        interleaved.to_vec()
    } else {
        interleaved
            .chunks(ch)
            .map(|f| f.iter().copied().sum::<f32>() / ch as f32)
            .collect()
    };
    crate::stt::resample_linear(&mono, sample_rate, 48_000)
}

/// 録音音声を Ogg Opus(.opus) として保存する（mono / 24kbps VBR / VOIP）。
/// libopus は vendored static でビルドされ、実行時の外部依存は無い。
pub fn save_opus(
    interleaved: &[f32],
    sample_rate: u32,
    channels: u16,
    dir: &Path,
    file_stem: &str,
) -> Result<PathBuf, String> {
    use std::io::Write;

    const FRAME: usize = 960; // 20ms @ 48kHz mono
    const PRE_SKIP: u16 = 312; // libopusの代表的な先読み(48kHz)。多少ずれても再生は可能。
    const SERIAL: u32 = 0x5175_6953; // 任意のストリームシリアル

    let pcm = to_mono_48k(interleaved, sample_rate, channels);
    if pcm.is_empty() {
        return Err(crate::errcode::E_EMPTY_AUDIO.into());
    }

    let mut enc = opus::Encoder::new(48_000, opus::Channels::Mono, opus::Application::Voip)
        .map_err(|e| crate::errcode::ec(crate::errcode::E_OPUS_ENCODER, e))?;
    enc.set_bitrate(opus::Bitrate::Bits(24_000))
        .map_err(|e| e.to_string())?;

    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{file_stem}.opus"));
    let file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
    let mut writer = ogg::PacketWriter::new(std::io::BufWriter::new(file));

    // OpusHead (RFC 7845)。
    let mut head = Vec::with_capacity(19);
    head.extend_from_slice(b"OpusHead");
    head.push(1); // version
    head.push(1); // channel count (mono)
    head.extend_from_slice(&PRE_SKIP.to_le_bytes());
    head.extend_from_slice(&sample_rate.to_le_bytes()); // 入力サンプルレート(情報)
    head.extend_from_slice(&0i16.to_le_bytes()); // output gain
    head.push(0); // mapping family 0
    writer
        .write_packet(head, SERIAL, ogg::PacketWriteEndInfo::EndPage, 0)
        .map_err(|e| crate::errcode::ec(crate::errcode::E_OPUS_HEAD, e))?;

    // OpusTags。
    let vendor = b"QuickScribe";
    let mut tags = Vec::new();
    tags.extend_from_slice(b"OpusTags");
    tags.extend_from_slice(&(vendor.len() as u32).to_le_bytes());
    tags.extend_from_slice(vendor);
    tags.extend_from_slice(&0u32.to_le_bytes()); // user comment count
    writer
        .write_packet(tags, SERIAL, ogg::PacketWriteEndInfo::EndPage, 0)
        .map_err(|e| crate::errcode::ec(crate::errcode::E_OPUS_TAGS, e))?;

    // 音声パケット（20msフレーム）。最後のフレームはゼロ詰め。
    let total_frames = pcm.len().div_ceil(FRAME);
    let mut out = vec![0u8; 4000];
    for i in 0..total_frames {
        let start = i * FRAME;
        let end = (start + FRAME).min(pcm.len());
        let mut frame = [0f32; FRAME];
        frame[..end - start].copy_from_slice(&pcm[start..end]);
        let n = enc
            .encode_float(&frame, &mut out)
            .map_err(|e| crate::errcode::ec(crate::errcode::E_OPUS_ENCODE, e))?;
        let granule = (i as u64 + 1) * FRAME as u64 + PRE_SKIP as u64;
        let info = if i + 1 == total_frames {
            ogg::PacketWriteEndInfo::EndStream
        } else {
            ogg::PacketWriteEndInfo::NormalPacket
        };
        writer
            .write_packet(out[..n].to_vec(), SERIAL, info, granule)
            .map_err(|e| crate::errcode::ec(crate::errcode::E_OPUS_PACKET, e))?;
    }

    writer
        .into_inner()
        .flush()
        .map_err(|e| crate::errcode::ec(crate::errcode::E_OPUS_EXPORT, e))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f32_to_i16_maps_range() {
        assert_eq!(f32_to_i16(0.0), 0);
        assert_eq!(f32_to_i16(1.0), i16::MAX);
        assert_eq!(f32_to_i16(-1.0), -i16::MAX);
        // クランプ
        assert_eq!(f32_to_i16(2.0), i16::MAX);
        assert_eq!(f32_to_i16(-2.0), -i16::MAX);
    }

    fn tmp_dir(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("qs_audio_save_{}_{}", std::process::id(), name))
    }

    #[test]
    fn save_wav_writes_readable_pcm16() {
        let dir = tmp_dir("wav");
        let _ = std::fs::remove_dir_all(&dir);
        let samples = vec![0.0, 0.5, -0.5, 1.0, -1.0, 0.25];
        let path = save_wav(&samples, 44_100, 2, &dir, "take1").unwrap();
        assert!(path.ends_with("take1.wav"));
        let mut reader = hound::WavReader::open(&path).unwrap();
        let spec = reader.spec();
        assert_eq!(spec.channels, 2);
        assert_eq!(spec.sample_rate, 44_100);
        assert_eq!(spec.bits_per_sample, 16);
        let read: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
        assert_eq!(read.len(), samples.len());
        assert_eq!(read[3], i16::MAX, "1.0 はフルスケールへ量子化");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_wav_clamps_degenerate_spec() {
        // channels=0 / 極端に低いレートは安全側（1ch / 8kHz 下限）に矯正して保存する。
        let dir = tmp_dir("wav0");
        let _ = std::fs::remove_dir_all(&dir);
        let path = save_wav(&[0.1, 0.2], 4_000, 0, &dir, "weird").unwrap();
        let spec = hound::WavReader::open(&path).unwrap().spec();
        assert_eq!(spec.channels, 1);
        assert_eq!(spec.sample_rate, 8_000);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_opus_rejects_empty_audio() {
        let dir = tmp_dir("opus-empty");
        let err = save_opus(&[], 48_000, 1, &dir, "empty").unwrap_err();
        assert_eq!(err, crate::errcode::E_EMPTY_AUDIO);
    }

    #[test]
    fn save_opus_writes_ogg_stream_with_headers() {
        let dir = tmp_dir("opus");
        let _ = std::fs::remove_dir_all(&dir);
        // 0.2秒 44.1kHz ステレオ → 48kHz mono へ整えて 20ms フレームでエンコード。
        let interleaved: Vec<f32> = (0..(8_820 * 2))
            .map(|i| (i as f32 * 0.01).sin() * 0.4)
            .collect();
        let path = save_opus(&interleaved, 44_100, 2, &dir, "rec1").unwrap();
        assert!(path.ends_with("rec1.opus"));
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(&bytes[0..4], b"OggS", "Oggコンテナで始まる");
        let hay = |needle: &[u8]| bytes.windows(needle.len()).any(|w| w == needle);
        assert!(hay(b"OpusHead"), "識別ヘッダを含む");
        assert!(hay(b"OpusTags") && hay(b"QuickScribe"), "コメントヘッダとvendorを含む");
        assert!(bytes.len() > 200, "音声パケットが書かれている: {}", bytes.len());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn to_mono_48k_downmixes_and_resamples() {
        // 24kHz ステレオ L/R 平均 → 48kHz で約2倍長。
        let interleaved = vec![0.0, 1.0, 0.0, 1.0, 0.0, 1.0];
        let out = to_mono_48k(&interleaved, 24_000, 2);
        assert!((out.len() as i32 - 6).abs() <= 1);
        assert!(out.iter().all(|&s| (s - 0.5).abs() < 1e-6));
        // mono はダウンミックス無しでリサンプルのみ。
        let out = to_mono_48k(&[0.2; 48], 48_000, 1);
        assert_eq!(out, vec![0.2; 48]);
    }
}
