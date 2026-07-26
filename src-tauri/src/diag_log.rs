// 内部診断ログの追記（#667）。常駐アプリがディスクを無制限に消費しないよう、
// サイズ上限＋1世代ローテーション（`x.log` → `x.log.1`、2世代で打ち止め）で書く。
// 既定では無効。環境変数で明示的に有効化したときだけ書き出す（診断用途のため）。
//
// 個人情報は載せない。ここへ渡してよいのは状態フラグや API 失敗理由などの
// 内部診断メッセージのみで、録音内容・文字起こし結果・保管庫本文は渡さないこと。

use std::path::{Path, PathBuf};

/// 1ファイルあたりの上限。超えたらローテーションする。
pub const MAX_BYTES: u64 = 1024 * 1024;

/// 環境変数の値から診断ログの有効/無効を判定する（未設定＝無効）。
pub fn enabled_from(value: Option<&str>) -> bool {
    matches!(
        value.map(|v| v.trim().to_ascii_lowercase()).as_deref(),
        Some("1") | Some("true") | Some("on") | Some("yes")
    )
}

/// ローテーション後のファイル名（`taskbar-diag.log` → `taskbar-diag.log.1`）。
fn rotated_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".1");
    PathBuf::from(name)
}

/// 1行を追記する。追記後に上限を超えるなら、先に1世代だけ退避してから書く。
///
/// 退避は `path` → `path.1` のリネーム1回のみ（既存の `.1` は上書き）。
/// 世代は2つで打ち止めなので、ディスク使用量は `MAX_BYTES * 2` 程度で頭打ちになる。
pub fn append_line(path: &Path, line: &str, max_bytes: u64) -> std::io::Result<()> {
    use std::io::Write;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let current = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    // +1 は改行分。空ファイルへの1行が上限を超える場合でもローテーションはしない
    // （直前の中身が無く、退避しても減らないため）。
    if current > 0 && current + line.len() as u64 + 1 > max_bytes {
        std::fs::rename(path, rotated_path(path))?;
    }
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(f, "{line}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("qs_diag_{tag}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn enabled_only_for_explicit_opt_in() {
        // 未設定・空・0 は無効。診断用途なので既定OFF。
        assert!(!enabled_from(None));
        assert!(!enabled_from(Some("")));
        assert!(!enabled_from(Some("0")));
        assert!(!enabled_from(Some("false")));
        assert!(enabled_from(Some("1")));
        assert!(enabled_from(Some("true")));
        assert!(enabled_from(Some(" ON ")), "前後空白と大文字も受ける");
        assert!(enabled_from(Some("yes")));
    }

    #[test]
    fn append_line_creates_missing_directory() {
        let dir = tmp_dir("mkdir");
        let path = dir.join("logs").join("taskbar-diag.log");
        append_line(&path, "hello", MAX_BYTES).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello\n");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rotates_before_exceeding_limit() {
        // 上限を超える書き込みの直前で1世代退避し、本体は上限内に収まる。
        let dir = tmp_dir("rotate");
        let path = dir.join("taskbar-diag.log");
        let limit = 100u64;
        let line = "x".repeat(20); // 1行 21バイト
        for _ in 0..4 {
            append_line(&path, &line, limit).unwrap();
        }
        assert_eq!(std::fs::metadata(&path).unwrap().len(), 84, "4行=84B は上限内");
        assert!(!rotated_path(&path).exists(), "まだ退避しない");

        append_line(&path, &line, limit).unwrap(); // 5行目で 105B > 100B

        assert_eq!(std::fs::metadata(&path).unwrap().len(), 21, "新しい本体は1行だけ");
        assert_eq!(
            std::fs::metadata(rotated_path(&path)).unwrap().len(),
            84,
            "直前の内容が .1 へ退避される"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn keeps_at_most_two_generations() {
        // 何度ローテーションしても世代は2つ。合計サイズが上限×2程度で頭打ちになる。
        let dir = tmp_dir("gens");
        let path = dir.join("taskbar-diag.log");
        let limit = 100u64;
        for i in 0..200 {
            append_line(&path, &format!("{i}:{}", "x".repeat(20)), limit).unwrap();
        }
        let files: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        assert_eq!(files.len(), 2, "本体と .1 の2つだけ: {files:?}");
        assert!(files.contains(&"taskbar-diag.log".to_string()));
        assert!(files.contains(&"taskbar-diag.log.1".to_string()));
        let total: u64 = files
            .iter()
            .map(|f| std::fs::metadata(dir.join(f)).unwrap().len())
            .sum();
        assert!(total <= limit * 2, "合計 {total}B が上限×2 に収まる");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn single_oversized_line_does_not_rotate_empty_file() {
        // 空ファイルへの1行が上限を超える場合、退避しても意味がないので書くだけ。
        let dir = tmp_dir("oversize");
        let path = dir.join("taskbar-diag.log");
        append_line(&path, &"x".repeat(50), 10).unwrap();
        assert!(!rotated_path(&path).exists());
        assert_eq!(std::fs::metadata(&path).unwrap().len(), 51);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
