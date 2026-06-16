// whisper モデルの保管と初回自動取得（S2.2）。
// 既定は ggml-base（日本語と速度のバランス）。OSのデータディレクトリ配下に保存する。

use std::path::PathBuf;

const MODEL_URL: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin";

/// モデル保管ディレクトリ（例: ~/.local/share/QuickScribe/models）。
pub fn model_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_default()
        .join("QuickScribe")
        .join("models")
}

/// 既定モデルのパス。
pub fn model_path() -> PathBuf {
    model_dir().join("ggml-base.bin")
}

/// モデルが無ければダウンロードしてパスを返す（あればそのまま）。
pub fn ensure_model() -> Result<PathBuf, String> {
    let path = model_path();
    if path.exists() {
        return Ok(path);
    }
    std::fs::create_dir_all(model_dir()).map_err(|e| e.to_string())?;

    let resp = ureq::get(MODEL_URL)
        .call()
        .map_err(|e| format!("モデルのダウンロードに失敗: {e}"))?;

    // 途中失敗で壊れたモデルを残さないよう .part に書いてから rename する。
    let tmp = path.with_extension("part");
    let mut reader = resp.into_reader();
    let mut file = std::fs::File::create(&tmp).map_err(|e| e.to_string())?;
    std::io::copy(&mut reader, &mut file).map_err(|e| e.to_string())?;
    file.sync_all().ok();
    drop(file);
    std::fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
    Ok(path)
}
