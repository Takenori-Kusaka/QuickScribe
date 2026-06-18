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
