// whisper モデルの保管・選択・初回自動取得（S2.2）。
// 標準 whisper.cpp ggml（速度/精度のトレードオフ）に加え、日本語特化 kotoba-whisper を選べる。
// 既定は ggml-base（日本語と速度のバランス）。OSのデータディレクトリ配下に保存する。

use std::io::{Read, Write};
use std::path::PathBuf;

use serde::Serialize;

/// 選択可能な whisper モデルのカタログ項目。
pub struct WhisperModel {
    /// 設定で渡される識別子（空文字 "" は既定=base）。
    pub id: &'static str,
    /// UI 表示名。
    pub label: &'static str,
    /// 保存ファイル名。
    pub filename: &'static str,
    /// ダウンロードURL（HuggingFace resolve）。
    pub url: &'static str,
}

/// モデルカタログ（既定 base を先頭に）。日本語特化 kotoba-whisper を含む。
pub const MODELS: &[WhisperModel] = &[
    WhisperModel {
        id: "base",
        label: "標準 base（日本語と速度のバランス・約142MB / 既定）",
        filename: "ggml-base.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin",
    },
    WhisperModel {
        id: "tiny",
        label: "最速 tiny（低精度・約75MB）",
        filename: "ggml-tiny.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin",
    },
    WhisperModel {
        id: "small",
        label: "高精度寄り small（約466MB）",
        filename: "ggml-small.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin",
    },
    WhisperModel {
        id: "medium",
        label: "高精度 medium（約1.5GB・低速）",
        filename: "ggml-medium.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin",
    },
    WhisperModel {
        id: "kotoba-q5",
        label: "日本語特化 kotoba-whisper 量子化（約538MB・推奨）",
        filename: "ggml-kotoba-whisper-v2.0-q5_0.bin",
        url: "https://huggingface.co/kotoba-tech/kotoba-whisper-v2.0-ggml/resolve/main/ggml-kotoba-whisper-v2.0-q5_0.bin",
    },
    WhisperModel {
        id: "kotoba",
        label: "日本語特化 kotoba-whisper 高精度（約1.5GB）",
        filename: "ggml-kotoba-whisper-v2.0.bin",
        url: "https://huggingface.co/kotoba-tech/kotoba-whisper-v2.0-ggml/resolve/main/ggml-kotoba-whisper-v2.0.bin",
    },
];

/// 一覧表示用の最小情報（フロントへ渡す）。
#[derive(Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub label: String,
}

/// カタログをフロント表示用に返す。
pub fn list_models() -> Vec<ModelInfo> {
    MODELS
        .iter()
        .map(|m| ModelInfo {
            id: m.id.to_string(),
            label: m.label.to_string(),
        })
        .collect()
}

/// id からモデルを解決する（空/未知は既定 base）。
fn model_for(id: &str) -> &'static WhisperModel {
    let id = id.trim();
    MODELS
        .iter()
        .find(|m| m.id == id)
        .unwrap_or(&MODELS[0])
}

/// モデル保管ディレクトリ（例: ~/.local/share/QuickScribe/models）。
pub fn model_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_default()
        .join("QuickScribe")
        .join("models")
}

/// 既定モデル（base）のパス。
pub fn model_path() -> PathBuf {
    model_dir().join(MODELS[0].filename)
}

/// 指定モデルが無ければダウンロードしてパスを返す（あればそのまま）。
/// 進捗を on_progress(downloaded, total) で通知する。空/未知idは既定 base。
pub fn ensure_model_id<F: FnMut(u64, Option<u64>)>(
    id: &str,
    on_progress: F,
) -> Result<PathBuf, String> {
    let m = model_for(id);
    let path = model_dir().join(m.filename);
    if path.exists() {
        return Ok(path);
    }
    std::fs::create_dir_all(model_dir()).map_err(|e| e.to_string())?;
    download_to(m.url, &path, on_progress)?;
    Ok(path)
}

/// 後方互換: 既定モデル(base)を用意する。
pub fn ensure_model<F: FnMut(u64, Option<u64>)>(on_progress: F) -> Result<PathBuf, String> {
    ensure_model_id("", on_progress)
}

/// URL から path へダウンロードする（.part に書いてから rename＝壊れたモデルを残さない）。
fn download_to<F: FnMut(u64, Option<u64>)>(
    url: &str,
    path: &PathBuf,
    mut on_progress: F,
) -> Result<(), String> {
    let resp = ureq::get(url)
        .call()
        .map_err(|e| format!("モデルのダウンロードに失敗: {e}"))?;
    let total: Option<u64> = resp.header("Content-Length").and_then(|s| s.parse().ok());

    let tmp = path.with_extension("part");
    let mut reader = resp.into_reader();
    let mut file = std::fs::File::create(&tmp).map_err(|e| e.to_string())?;
    let mut buf = [0u8; 65536];
    let mut downloaded: u64 = 0;
    loop {
        let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n]).map_err(|e| e.to_string())?;
        downloaded += n as u64;
        on_progress(downloaded, total);
    }
    file.sync_all().ok();
    drop(file);
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_for_defaults_to_base() {
        assert_eq!(model_for("").id, "base");
        assert_eq!(model_for("unknown").id, "base");
        assert_eq!(model_for("kotoba-q5").id, "kotoba-q5");
    }

    #[test]
    fn catalog_has_kotoba_and_unique_ids() {
        assert!(MODELS.iter().any(|m| m.id == "kotoba-q5"));
        let mut ids: Vec<&str> = MODELS.iter().map(|m| m.id).collect();
        let n = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), n, "id は一意");
    }

    #[test]
    fn list_models_includes_default_first() {
        let l = list_models();
        assert_eq!(l[0].id, "base");
        assert!(l.iter().any(|m| m.id == "kotoba"));
    }
}
