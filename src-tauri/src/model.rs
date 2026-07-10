// whisper モデルの保管・選択・初回自動取得（S2.2）。
// 標準 whisper.cpp ggml（速度/精度のトレードオフ）から用途に応じて選ぶ。
// 既定は ggml-base（日本語と速度のバランス）。OSのデータディレクトリ配下に保存する。

use std::io::{Read, Write};
use std::path::PathBuf;

use serde::Serialize;
use sha2::{Digest, Sha256};

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
    /// 期待される SHA256（HuggingFace LFS の oid＝ファイル内容の sha256）。
    /// ダウンロード後に照合し、改ざん/破損モデルを排除する(#391)。
    pub sha256: &'static str,
    /// 期待されるバイトサイズ（破損・途中切れの早期検出）。
    pub size: u64,
    /// 相対的な処理速度クラス（#598）: "fastest" | "fast" | "medium" | "slow"。
    /// RTF は端末性能に依存するため絶対値ではなく相対クラスで示す（大きいモデルほど遅い、は不変）。
    /// フロントは `settings.model_speed_<speed>` で「最速/速い/普通/低速」を表示する。
    pub speed: &'static str,
}

/// モデルカタログ（カタログ順の先頭 base。日本語の既定は large-v3-turbo / ADR-0025）。
/// 実録音での実測(ADR-0025)で、**kotoba-whisper は長尺の末尾欠落＋自発発話(会話)で崩壊**が確認され、
/// 日本語既定を kotoba → **large-v3-turbo** に変更した。turbo は会話に強く長尺の末尾も確実に取れる
/// （ただし低速 RTF~1.15）。**kotoba は本プロダクトのコア用途(自発発話)と正面衝突し開発も停止のため
/// カタログから撤去した(ADR-0029)**。base は速く頑健なフォールバック（末尾欠落なし）。
/// tiny は日本語で非推奨（実測で base に劣位）。ADR-0022(削らず用途で導く)の例外を ADR-0029 で承認。
pub const MODELS: &[WhisperModel] = &[
    WhisperModel {
        id: "base",
        label: "標準 base（日本語と速度のバランス・約142MB / 既定・頑健）",
        filename: "ggml-base.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin",
        sha256: "60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe",
        size: 147951465,
        speed: "fast",
    },
    WhisperModel {
        id: "large-v3-turbo",
        label: "高精度 large-v3-turbo 量子化（日本語・会話に強い／長尺の末尾も確実・約547MB・低速）",
        filename: "ggml-large-v3-turbo-q5_0.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo-q5_0.bin",
        sha256: "394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2",
        size: 574041195,
        speed: "slow",
    },
    WhisperModel {
        id: "small",
        label: "多言語 small（英語向き・約466MB）",
        filename: "ggml-small.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin",
        sha256: "1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b",
        size: 487601967,
        speed: "medium",
    },
    WhisperModel {
        id: "medium",
        label: "多言語 medium（英語向き・高精度・約1.5GB・低速）",
        filename: "ggml-medium.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin",
        sha256: "6c14d5adee5f86394037b4e4e8b59f1673b6cee10e3cf0b11bbdbee79c156208",
        size: 1533763059,
        speed: "slow",
    },
    WhisperModel {
        id: "tiny",
        label: "最速 tiny（英語・下書き向き / 日本語は低精度で非推奨・約75MB）",
        filename: "ggml-tiny.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin",
        sha256: "be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21",
        size: 77691713,
        speed: "fastest",
    },
];

/// 一覧表示用の最小情報（フロントへ渡す）。
/// label は日本語既定のフォールバック。フロントは id を安定識別子として
/// `catalog.whisper_models.<id>`（ja/en/zh/es）で表示名を解決し、未収載 id のみ label を使う(#462)。
#[derive(Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub label: String,
    /// 相対処理速度クラス（#598）。フロントは settings.model_speed_<speed> で表示する。
    pub speed: String,
}

/// カタログをフロント表示用に返す。
pub fn list_models() -> Vec<ModelInfo> {
    MODELS
        .iter()
        .map(|m| ModelInfo {
            id: m.id.to_string(),
            label: m.label.to_string(),
            speed: m.speed.to_string(),
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
    download_to(m.url, &path, m.sha256, m.size, on_progress)?;
    Ok(path)
}

/// 後方互換: 既定モデル(base)を用意する。
pub fn ensure_model<F: FnMut(u64, Option<u64>)>(on_progress: F) -> Result<PathBuf, String> {
    ensure_model_id("", on_progress)
}

/// 話者特定(S2.5)のONNXモデル2種（segmentation / embedding）の解決済みパス。
#[cfg(feature = "diarization")]
pub struct DiarizationAssets {
    pub segmentation: PathBuf,
    pub embedding: PathBuf,
}

/// 話者特定用ONNXモデル(pyannote segmentation + 話者埋め込み)をオンデマンドで用意する
/// （無ければDL）。有効時のみ呼ばれる＝非利用者は増量ゼロ・単一バイナリ維持（ADR-0012 / 選択A）。
/// 保存先は model_dir()/diarization/。
///
/// TODO(Phase2・リリース前必須): (1) 配布URLを確定し SHA256/サイズをピン留めして整合性検証を有効化
/// する（現状は sha=""・size=0 で未検証DL＝R4 未達）。(2) ネイティブランタイム(onnxruntime 等)の
/// オンデマンド取得＋DLL検索パス設定(SetDllDirectory)＋遅延ロード(build.rs /DELAYLOAD)を実装し、
/// 未取得時にabortせずフォールバックする(R5)ことを実機で検証する。
#[cfg(feature = "diarization")]
pub fn ensure_diarization_assets<F: FnMut(u64, Option<u64>)>(
    mut on_progress: F,
) -> Result<DiarizationAssets, String> {
    let dir = model_dir().join("diarization");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    // segmentation=pyannote 3.0、embedding=3D-Speaker ERes2Net(zh-cn・話者埋め込みは概ね言語非依存)。
    let seg = dir.join("sherpa-onnx-pyannote-segmentation-3-0.onnx");
    let emb = dir.join("3dspeaker_eres2net_base_sv_zh-cn_16k.onnx");
    const SEG_URL: &str = "https://huggingface.co/csukuangfj/sherpa-onnx-pyannote-segmentation-3-0/resolve/main/model.onnx";
    const EMB_URL: &str = "https://huggingface.co/csukuangfj/speaker-embedding-models/resolve/main/3dspeaker_speech_eres2net_base_sv_zh-cn_3dspeaker_16k.onnx";
    if !seg.exists() {
        download_to(SEG_URL, &seg, "", 0, &mut on_progress)?;
    }
    if !emb.exists() {
        download_to(EMB_URL, &emb, "", 0, &mut on_progress)?;
    }
    Ok(DiarizationAssets { segmentation: seg, embedding: emb })
}

/// URL から path へダウンロードする（.part に書いてから rename＝壊れたモデルを残さない）。
/// ダウンロード中に SHA256 を逐次計算し、完了後に期待値・サイズと照合する(#391)。
/// 不一致なら .part を削除してエラーにし、改ざん/破損モデルを残さない。
/// expected_sha256 が空文字のときはハッシュ照合をスキップ（expected_size==0 も同様）。
fn download_to<F: FnMut(u64, Option<u64>)>(
    url: &str,
    path: &PathBuf,
    expected_sha256: &str,
    expected_size: u64,
    mut on_progress: F,
) -> Result<(), String> {
    let resp = ureq::get(url)
        .call()
        .map_err(|e| crate::errcode::ec(crate::errcode::E_MODEL_DOWNLOAD, e))?;
    let total: Option<u64> = resp
        .headers()
        .get("Content-Length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok());

    let tmp = path.with_extension("part");
    // ureq3 の読取は既定で上限があるため、大容量モデル(最大~1.5GB)が切り詰められないよう
    // 上限を実質無制限にしてストリーム読みする(整合性は後段の SHA256/サイズ照合で担保)。
    let mut body = resp.into_body();
    let mut reader = body.with_config().limit(u64::MAX).reader();
    let mut file = std::fs::File::create(&tmp).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    let mut downloaded: u64 = 0;
    loop {
        let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n]).map_err(|e| e.to_string())?;
        hasher.update(&buf[..n]);
        downloaded += n as u64;
        on_progress(downloaded, total);
    }
    file.sync_all().ok();
    drop(file);

    // 整合性検証(#391): サイズ + SHA256 を照合。破損・途中切れ・改ざんを検出する。
    let actual = hex::encode(hasher.finalize());
    if let Err(e) = verify_integrity(downloaded, &actual, expected_size, expected_sha256) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())?;
    Ok(())
}

/// ダウンロード結果のサイズ・SHA256 を期待値と照合する（純関数＝テスト可能）。
/// expected_size==0 はサイズ照合を、expected_sha256 が空はハッシュ照合をスキップ。
fn verify_integrity(
    actual_size: u64,
    actual_sha256: &str,
    expected_size: u64,
    expected_sha256: &str,
) -> Result<(), String> {
    if expected_size != 0 && actual_size != expected_size {
        return Err(crate::errcode::ec(
            crate::errcode::E_MODEL_SIZE_MISMATCH,
            format!("expected {expected_size} bytes / actual {actual_size} bytes"),
        ));
    }
    if !expected_sha256.is_empty() && !actual_sha256.eq_ignore_ascii_case(expected_sha256) {
        return Err(crate::errcode::ec(
            crate::errcode::E_MODEL_SHA256_MISMATCH,
            format!("expected {expected_sha256} / actual {actual_sha256}"),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_for_defaults_to_base() {
        assert_eq!(model_for("").id, "base");
        assert_eq!(model_for("unknown").id, "base");
        // 撤去済み kotoba(ADR-0029)を選択していた既存ユーザーは既定 base へ安全にフォールバックする。
        assert_eq!(model_for("kotoba-q5").id, "base");
        assert_eq!(model_for("kotoba").id, "base");
    }

    #[test]
    fn catalog_has_unique_ids_and_no_kotoba() {
        // kotoba はコア用途(自発発話)で崩壊＋開発停止のため撤去済み(ADR-0029)。
        assert!(!MODELS.iter().any(|m| m.id.starts_with("kotoba")));
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
        assert!(l.iter().any(|m| m.id == "large-v3-turbo"));
    }

    #[test]
    fn catalog_curation_orders_ja_first_and_guides_tiny() {
        // ADR-0022: 日本語向きを上位に、tiny は末尾に降格する。
        let ids: Vec<&str> = MODELS.iter().map(|m| m.id).collect();
        let pos = |id: &str| ids.iter().position(|x| *x == id).unwrap();
        assert_eq!(ids[0], "base", "既定 base は先頭");
        assert!(
            pos("large-v3-turbo") < pos("small"),
            "日本語推奨(turbo)は英語向きより上位"
        );
        assert_eq!(*ids.last().unwrap(), "tiny", "tiny は末尾へ降格");
        // ラベルで日本語の適否を明示（実測に基づくガイド）。
        let tiny = MODELS.iter().find(|m| m.id == "tiny").unwrap();
        assert!(tiny.label.contains("非推奨"), "tiny は日本語非推奨を明示");
    }

    #[test]
    fn catalog_entries_have_valid_sha256_and_size() {
        for m in MODELS {
            assert_eq!(m.sha256.len(), 64, "{} の sha256 は64桁hex", m.id);
            assert!(
                m.sha256.chars().all(|c| c.is_ascii_hexdigit()),
                "{} の sha256 はhexのみ",
                m.id
            );
            assert!(m.size > 0, "{} の size は正", m.id);
        }
    }

    #[test]
    fn sha256_matches_known_vector() {
        // NIST 既知ベクトル: SHA256("abc")。
        let actual = hex::encode(Sha256::digest(b"abc"));
        assert_eq!(
            actual,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn verify_integrity_accepts_match_and_skips() {
        // 完全一致。
        assert!(verify_integrity(10, "ABCD", 10, "abcd").is_ok());
        // 期待値が空/0なら該当照合をスキップ。
        assert!(verify_integrity(10, "abcd", 0, "").is_ok());
        assert!(verify_integrity(10, "abcd", 0, "abcd").is_ok());
    }

    #[test]
    fn verify_integrity_rejects_size_and_hash_mismatch() {
        assert!(verify_integrity(9, "abcd", 10, "abcd").is_err()); // サイズ不一致
        assert!(verify_integrity(10, "dead", 10, "beef").is_err()); // ハッシュ不一致
    }

    #[test]
    fn model_dir_and_default_path_layout() {
        assert!(model_dir().ends_with(std::path::Path::new("QuickScribe/models")));
        assert!(model_path().ends_with(std::path::Path::new("QuickScribe/models/ggml-base.bin")));
    }

    // ─── download_to のローカルサーバ検証（#391 整合性検証を含む / 監査項目12） ───
    use crate::testhttp::{serve, Route};

    fn tmp_target(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("qs_model_dl_{}_{}", std::process::id(), name))
    }

    #[test]
    fn download_to_writes_file_and_reports_progress() {
        let body = b"hello-model-bytes".to_vec();
        let sha = hex::encode(Sha256::digest(&body));
        let (base, _) = serve(vec![Route {
            path_contains: "/m.bin",
            status: 200,
            body: body.clone(),
        }]);
        let path = tmp_target("ok.bin");
        let _ = std::fs::remove_file(&path);
        let mut seen: Vec<(u64, Option<u64>)> = Vec::new();
        download_to(
            &format!("{base}/m.bin"),
            &path,
            &sha,
            body.len() as u64,
            |d, t| seen.push((d, t)),
        )
        .unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), body, "本文が完全に書けている");
        assert!(!path.with_extension("part").exists(), ".part は残さない");
        // Content-Length があるので total は Some(len)。
        assert!(seen.iter().all(|(_, t)| *t == Some(body.len() as u64)));
        assert_eq!(seen.last().unwrap().0, body.len() as u64);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn download_to_rejects_sha_mismatch_and_cleans_up() {
        let body = b"corrupted-bytes".to_vec();
        let (base, _) = serve(vec![Route {
            path_contains: "/bad.bin",
            status: 200,
            body: body.clone(),
        }]);
        let path = tmp_target("bad.bin");
        let err = download_to(
            &format!("{base}/bad.bin"),
            &path,
            "00000000000000000000000000000000ffffffffffffffffffffffffffffffff",
            body.len() as u64,
            |_, _| {},
        )
        .unwrap_err();
        assert!(err.starts_with(crate::errcode::E_MODEL_SHA256_MISMATCH), "{err}");
        assert!(!path.exists(), "破損モデルを残さない");
        assert!(!path.with_extension("part").exists(), ".part も削除");
    }

    #[test]
    fn download_to_rejects_size_mismatch() {
        let body = b"short".to_vec();
        let (base, _) = serve(vec![Route {
            path_contains: "/s.bin",
            status: 200,
            body: body.clone(),
        }]);
        let path = tmp_target("size.bin");
        let err = download_to(&format!("{base}/s.bin"), &path, "", 999, |_, _| {}).unwrap_err();
        assert!(err.starts_with(crate::errcode::E_MODEL_SIZE_MISMATCH), "{err}");
        assert!(!path.exists());
    }

    #[test]
    fn download_to_maps_http_error_to_stable_code() {
        let (base, _) = serve(vec![]); // ルート無し → 404
        let path = tmp_target("404.bin");
        let err = download_to(&format!("{base}/none.bin"), &path, "", 0, |_, _| {}).unwrap_err();
        assert!(err.starts_with(crate::errcode::E_MODEL_DOWNLOAD), "{err}");
    }
}
