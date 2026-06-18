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
        hound::WavWriter::create(&path, spec).map_err(|e| format!("WAV作成に失敗: {e}"))?;
    for &s in interleaved {
        writer
            .write_sample(f32_to_i16(s))
            .map_err(|e| e.to_string())?;
    }
    writer
        .finalize()
        .map_err(|e| format!("WAV書き出しに失敗: {e}"))?;
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
        return Err("音声データが空です".into());
    }

    let mut enc = opus::Encoder::new(48_000, opus::Channels::Mono, opus::Application::Voip)
        .map_err(|e| format!("Opusエンコーダ生成に失敗: {e}"))?;
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
        .map_err(|e| format!("OpusHead書き込みに失敗: {e}"))?;

    // OpusTags。
    let vendor = b"QuickScribe";
    let mut tags = Vec::new();
    tags.extend_from_slice(b"OpusTags");
    tags.extend_from_slice(&(vendor.len() as u32).to_le_bytes());
    tags.extend_from_slice(vendor);
    tags.extend_from_slice(&0u32.to_le_bytes()); // user comment count
    writer
        .write_packet(tags, SERIAL, ogg::PacketWriteEndInfo::EndPage, 0)
        .map_err(|e| format!("OpusTags書き込みに失敗: {e}"))?;

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
            .map_err(|e| format!("Opusエンコードに失敗: {e}"))?;
        let granule = (i as u64 + 1) * FRAME as u64 + PRE_SKIP as u64;
        let info = if i + 1 == total_frames {
            ogg::PacketWriteEndInfo::EndStream
        } else {
            ogg::PacketWriteEndInfo::NormalPacket
        };
        writer
            .write_packet(out[..n].to_vec(), SERIAL, info, granule)
            .map_err(|e| format!("Opusパケット書き込みに失敗: {e}"))?;
    }

    writer
        .into_inner()
        .flush()
        .map_err(|e| format!("Opus書き出しに失敗: {e}"))?;
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
}
