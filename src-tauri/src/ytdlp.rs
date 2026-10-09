// yt-dlp による YouTube / Twitch 等の動画リンクからの音声ダウンロードモジュール。
//
// 1. システム PATH または `%APPDATA%/QuickScribe/tools/` の yt-dlp バイナリを解決。
// 2. 見つからない場合は GitHub releases からオンデマンド取得（プロキシ対応）。
// 3. YouTube / Twitch のストリーム差異（Audio_Only / bestaudio / クリップフォールバック）を吸収。
// 4. 引数インジェクション対策（-- による境界分離、安全な URL 検証）。
// 5. ダウンロード進捗のパースと通知、処理後の一時ディレクトリ自動破棄。

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};

/// アプリ専用の外部ツール格納ディレクトリ（`%APPDATA%/QuickScribe/tools`）。
pub fn tools_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_default()
        .join("QuickScribe")
        .join("tools")
}

/// 実行可能ファイル名（Windows は .exe 付き）。
#[cfg(windows)]
const YTDLP_BIN_NAME: &str = "yt-dlp.exe";
#[cfg(not(windows))]
const YTDLP_BIN_NAME: &str = "yt-dlp";

/// yt-dlp のダウンロード元 URL（公式 GitHub Releases のスタンドアロンバイナリ）。
#[cfg(windows)]
pub const YTDLP_DOWNLOAD_URL: &str =
    "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe";
#[cfg(not(windows))]
pub const YTDLP_DOWNLOAD_URL: &str =
    "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp";

/// システム PATH または tools_dir 内で yt-dlp 実行ファイルを検索する。
pub fn find_ytdlp_bin() -> Option<PathBuf> {
    // 1. PATH 環境変数を走査
    if let Ok(path_var) = std::env::var("PATH") {
        let sep = if cfg!(windows) { ';' } else { ':' };
        for dir in path_var.split(sep) {
            let candidate = Path::new(dir).join(YTDLP_BIN_NAME);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    // 2. tools ディレクトリ内を確認
    let app_tool = tools_dir().join(YTDLP_BIN_NAME);
    if app_tool.is_file() {
        return Some(app_tool);
    }

    None
}

/// yt-dlp バイナリの存在を確認し、なければダウンロードして配置する。
pub fn ensure_ytdlp_bin<F: FnMut(u64, Option<u64>)>(mut on_progress: F) -> Result<PathBuf, String> {
    if let Some(bin) = find_ytdlp_bin() {
        return Ok(bin);
    }

    let dir = tools_dir();
    std::fs::create_dir_all(&dir)
        .map_err(|e| crate::errcode::ec(crate::errcode::E_YTDLP_DOWNLOAD, format!("mkdir failed: {e}")))?;

    let target = dir.join(YTDLP_BIN_NAME);
    let tmp = dir.join(format!("{}.part", YTDLP_BIN_NAME));

    let agent = crate::proxy::build_agent_for_url(YTDLP_DOWNLOAD_URL);
    let resp = agent
        .get(YTDLP_DOWNLOAD_URL)
        .call()
        .map_err(|e| crate::errcode::ec(crate::errcode::E_YTDLP_DOWNLOAD, format!("HTTP request failed: {e}")))?;

    let total = resp
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());

    let mut reader = resp.into_body().into_reader();
    let mut file = std::fs::File::create(&tmp)
        .map_err(|e| crate::errcode::ec(crate::errcode::E_YTDLP_DOWNLOAD, format!("file create failed: {e}")))?;

    let mut written: u64 = 0;
    let mut buf = [0u8; 64 * 1024];

    loop {
        use std::io::Read;
        let n = reader
            .read(&mut buf)
            .map_err(|e| crate::errcode::ec(crate::errcode::E_YTDLP_DOWNLOAD, format!("stream read failed: {e}")))?;
        if n == 0 {
            break;
        }
        use std::io::Write;
        file.write_all(&buf[..n])
            .map_err(|e| crate::errcode::ec(crate::errcode::E_YTDLP_DOWNLOAD, format!("write failed: {e}")))?;
        written += n as u64;
        on_progress(written, total);
    }

    drop(file);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        let _ = std::fs::set_permissions(&tmp, perms);
    }

    std::fs::rename(&tmp, &target)
        .map_err(|e| crate::errcode::ec(crate::errcode::E_YTDLP_DOWNLOAD, format!("rename failed: {e}")))?;

    crate::diag_log::log("ytdlp", &format!("yt-dlp binary installed to {}", target.display()));
    Ok(target)
}

/// 動画 URL の基本的な正当性と安全性を検証する（純関数・テスト可能）。
/// コマンドライン引数インジェクションや不正なスキームを排除する。
pub fn validate_video_url(url: &str) -> Result<String, String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err(crate::errcode::ec(crate::errcode::E_YTDLP_INVALID_URL, "URL is empty"));
    }

    // 引数インジェクション防止（先頭がハイフンで始まるオプション形式を遮断）
    if trimmed.starts_with('-') {
        return Err(crate::errcode::ec(
            crate::errcode::E_YTDLP_INVALID_URL,
            "URL cannot start with a hyphen",
        ));
    }

    // NULL バイトや改行の混入を遮断
    if trimmed.contains('\0') || trimmed.contains('\n') || trimmed.contains('\r') {
        return Err(crate::errcode::ec(
            crate::errcode::E_YTDLP_INVALID_URL,
            "URL contains invalid control characters",
        ));
    }

    // HTTP / HTTPS スキームの確認
    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        return Err(crate::errcode::ec(
            crate::errcode::E_YTDLP_INVALID_URL,
            "URL must start with http:// or https://",
        ));
    }

    Ok(trimmed.to_string())
}

/// yt-dlp の進捗行からダウンロード進捗率（0.0 〜 100.0）を抽出する。
/// 例: `[download]  45.2% of 12.34MiB at 2.50MiB/s ETA 00:02`
pub fn parse_progress_percent(line: &str) -> Option<f32> {
    if !line.contains("[download]") {
        return None;
    }
    let parts: Vec<&str> = line.split_whitespace().collect();
    // [download] の直後または近傍にある `%` で終わる要素を探す
    for part in parts {
        if part.ends_with('%') {
            if let Ok(pct) = part.trim_end_matches('%').parse::<f32>() {
                return Some(pct);
            }
        }
    }
    None
}

/// yt-dlp を実行して動画 URL から音声ストリームをダウンロードする。
/// 戻り値: (ダウンロードされた音声ファイルの絶対パス, 一時親ディレクトリ)
/// 呼び出し側は処理完了後に一時親ディレクトリを削除すること。
pub fn download_audio_from_url<F: FnMut(f32)>(
    ytdlp_bin: &Path,
    url: &str,
    mut on_progress: F,
) -> Result<(PathBuf, PathBuf), String> {
    let safe_url = validate_video_url(url)?;

    // 一意な一時作業ディレクトリを作成
    let unique_id = format!(
        "qs_ytdlp_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    let work_dir = std::env::temp_dir().join(unique_id);
    std::fs::create_dir_all(&work_dir)
        .map_err(|e| crate::errcode::ec(crate::errcode::E_YTDLP_EXEC, format!("failed to create temp dir: {e}")))?;

    let out_template = work_dir.join("%(id)s.%(ext)s");
    let filepath_txt = work_dir.join("filepath.txt");

    let mut cmd = Command::new(ytdlp_bin);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    cmd.arg("--no-playlist")
        // Twitch (Audio_Only / audio_only), YouTube (ba[ext=m4a] / bestaudio), フォールバック (ba / best / b)
        .arg("-f")
        .arg("Audio_Only/audio_only/ba[ext=m4a]/bestaudio/ba/best/b")
        .arg("--newline")
        .arg("--no-colors")
        .arg("--no-warnings")
        .arg("-o")
        .arg(&out_template)
        .arg("--print-to-file")
        .arg("after_move:filepath")
        .arg(&filepath_txt)
        .arg("--")
        .arg(&safe_url)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| crate::errcode::ec(crate::errcode::E_YTDLP_EXEC, format!("failed to spawn yt-dlp: {e}")))?;

    // stdout をリアルタイムで読み取って進捗通知
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            if let Some(pct) = parse_progress_percent(&line) {
                on_progress(pct);
            }
        }
    }

    let status = child
        .wait()
        .map_err(|e| crate::errcode::ec(crate::errcode::E_YTDLP_EXEC, format!("wait on yt-dlp failed: {e}")))?;

    if !status.success() {
        let mut err_msg = String::new();
        if let Some(mut stderr) = child.stderr.take() {
            use std::io::Read;
            let _ = stderr.read_to_string(&mut err_msg);
        }
        let _ = std::fs::remove_dir_all(&work_dir);
        return Err(crate::errcode::ec(
            crate::errcode::E_YTDLP_EXEC,
            format!("yt-dlp exited with status {status}: {err_msg}"),
        ));
    }

    // filepath.txt から最終ダウンロードファイルのパスを取得
    let target_file = if filepath_txt.is_file() {
        let content = std::fs::read_to_string(&filepath_txt)
            .map_err(|e| crate::errcode::ec(crate::errcode::E_YTDLP_EXEC, format!("read filepath.txt failed: {e}")))?;
        let first_line = content.lines().next().unwrap_or("").trim();
        PathBuf::from(first_line)
    } else {
        // フォールバック: work_dir 直下のファイルを走査
        let mut found = None;
        if let Ok(entries) = std::fs::read_dir(&work_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && p != filepath_txt {
                    found = Some(p);
                    break;
                }
            }
        }
        found.ok_or_else(|| {
            let _ = std::fs::remove_dir_all(&work_dir);
            crate::errcode::ec(crate::errcode::E_YTDLP_EXEC, "no downloaded file found")
        })?
    };

    if !target_file.is_file() {
        let _ = std::fs::remove_dir_all(&work_dir);
        return Err(crate::errcode::ec(
            crate::errcode::E_YTDLP_EXEC,
            format!("target file does not exist: {}", target_file.display()),
        ));
    }

    Ok((target_file, work_dir))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_video_url_accepts_valid_urls() {
        assert!(validate_video_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ").is_ok());
        assert!(validate_video_url("https://youtu.be/dQw4w9WgXcQ").is_ok());
        assert!(validate_video_url("https://www.twitch.tv/videos/12345678").is_ok());
        assert!(validate_video_url("https://clips.twitch.tv/SampleClipName").is_ok());
        assert!(validate_video_url("http://example.com/audio.mp3").is_ok());
    }

    #[test]
    fn validate_video_url_rejects_flag_injection() {
        let err = validate_video_url("--output /etc/passwd").unwrap_err();
        assert!(err.contains(crate::errcode::E_YTDLP_INVALID_URL));
        let err2 = validate_video_url("-f bestaudio").unwrap_err();
        assert!(err2.contains(crate::errcode::E_YTDLP_INVALID_URL));
    }

    #[test]
    fn validate_video_url_rejects_invalid_schemes_and_control_chars() {
        assert!(validate_video_url("").is_err());
        assert!(validate_video_url("file:///etc/passwd").is_err());
        assert!(validate_video_url("javascript:alert(1)").is_err());
        assert!(validate_video_url("https://youtube.com/watch?v=1\n--evil").is_err());
        assert!(validate_video_url("https://youtube.com/watch?v=1\0evil").is_err());
    }

    #[test]
    fn parse_progress_percent_extracts_correct_values() {
        assert_eq!(
            parse_progress_percent("[download]  45.2% of 12.34MiB at 2.50MiB/s ETA 00:02"),
            Some(45.2)
        );
        assert_eq!(
            parse_progress_percent("[download] 100.0% of 5.00MiB"),
            Some(100.0)
        );
        assert_eq!(
            parse_progress_percent("[download]   0.0% of 1.00MiB"),
            Some(0.0)
        );
        assert_eq!(parse_progress_percent("Some other log line"), None);
        assert_eq!(parse_progress_percent("[download] Destination: test.m4a"), None);
    }
}
