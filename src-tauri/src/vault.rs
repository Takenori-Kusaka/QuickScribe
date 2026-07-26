// 保管庫エントリの一覧・解析（S4.3 Phase1 / ADR-0015）。
// 過去エントリをアプリ内で横断（タグ/全文絞り込み）するための読み取り側。
// 保存形式(S4.2/S4.3): md=YAMLフロントマター(created/type/style/tags) / txt=末尾 Tags: 行。
// 外部編集(Obsidian等)も想定し、タグは [a,b] / a,b / 箇条書き(- a) を緩く受ける。

use std::path::Path;

use serde::Serialize;

/// 一覧表示用のエントリ要約。
#[derive(Serialize, Clone, PartialEq, Debug)]
pub struct EntrySummary {
    pub path: String,
    pub name: String,
    /// 作成日時(ISO8601)。フロントマターの created、無ければファイル更新時刻。
    pub created: String,
    /// 種別(transcript/refined/note)。不明は空。
    pub kind: String,
    pub tags: Vec<String>,
    /// 本文の冒頭プレビュー(1行)。
    pub preview: String,
}

/// 解析結果（メタ＋本文）。
pub struct Parsed {
    pub created: Option<String>,
    pub kind: Option<String>,
    pub tags: Vec<String>,
    pub body: String,
}

/// 先頭/末尾のクォートと空白を除く。
fn unquote(s: &str) -> String {
    let t = s.trim();
    let t = t.strip_prefix('"').unwrap_or(t);
    let t = t.strip_suffix('"').unwrap_or(t);
    t.trim().to_string()
}

/// タグ表記をパースする：`[a, b]` / `a, b` のいずれも受ける（各要素はunquote・空/先頭#除去）。
fn parse_tag_inline(v: &str) -> Vec<String> {
    let inner = v.trim();
    let inner = inner.strip_prefix('[').unwrap_or(inner);
    let inner = inner.strip_suffix(']').unwrap_or(inner);
    inner
        .split([',', '、'])
        .map(|t| unquote(t).trim_start_matches('#').trim().to_string())
        .filter(|t| !t.is_empty())
        .collect()
}

/// エントリ内容（md frontmatter / txt）からメタと本文を解析する（純粋・テスト対象）。
pub fn parse_entry(content: &str) -> Parsed {
    let text = content.trim_start_matches(['\u{feff}']); // BOM除去
    // md: 先頭 --- ... --- のフロントマター。
    if let Some(rest) = text.strip_prefix("---\n").or_else(|| text.strip_prefix("---\r\n")) {
        if let Some(end) = rest.find("\n---") {
            let fm = &rest[..end];
            // 終了 --- 行の次から本文。
            let after = &rest[end..];
            let body = after
                .trim_start_matches(['\n', '\r'])
                .trim_start_matches("---")
                .trim_start_matches(['\n', '\r'])
                .to_string();
            let (created, kind, tags) = parse_frontmatter(fm);
            return Parsed {
                created,
                kind,
                tags,
                body,
            };
        }
    }
    // txt: 末尾の `Tags: a, b` 行があれば抽出し、本文から外す。
    let mut tags = Vec::new();
    let mut body_lines: Vec<&str> = text.lines().collect();
    while matches!(body_lines.last(), Some(l) if l.trim().is_empty()) {
        body_lines.pop();
    }
    if let Some(last) = body_lines.last() {
        if let Some(v) = last.trim().strip_prefix("Tags:") {
            tags = parse_tag_inline(v);
            body_lines.pop();
        }
    }
    Parsed {
        created: None,
        kind: None,
        tags,
        body: body_lines.join("\n").trim().to_string(),
    }
}

/// フロントマター本文から created/type/tags を抜く（自前形式＋Obsidian的な箇条書きtagsを許容）。
fn parse_frontmatter(fm: &str) -> (Option<String>, Option<String>, Vec<String>) {
    let mut created = None;
    let mut kind = None;
    let mut tags = Vec::new();
    let lines: Vec<&str> = fm.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if let Some((key, val)) = line.split_once(':') {
            let key = key.trim();
            let val = val.trim();
            match key {
                "created" => created = Some(unquote(val)),
                "type" => kind = Some(unquote(val)),
                "tags" => {
                    if val.is_empty() {
                        // 箇条書き形式: 後続の "- x" 行を集める。
                        let mut j = i + 1;
                        while j < lines.len() {
                            let t = lines[j].trim();
                            if let Some(item) = t.strip_prefix('-') {
                                let v = unquote(item).trim_start_matches('#').trim().to_string();
                                if !v.is_empty() {
                                    tags.push(v);
                                }
                                j += 1;
                            } else {
                                break;
                            }
                        }
                        i = j;
                        continue;
                    } else {
                        tags = parse_tag_inline(val);
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }
    (created, kind, tags)
}

/// ファイル名から種別を推定する（フロントマター欠落時の補完・主にtxt用）。
/// 新形式 `{yyyymmdd}-{種別}-…`(ADR-0032) は先頭の日付を読み飛ばして判定し、
/// 旧形式 `{種別}-…`（既存ファイル）も引き続き見分ける。該当なしは空。
pub fn kind_from_filename(name: &str) -> &'static str {
    // 先頭が数字列+ハイフン(日付)なら読み飛ばす（新形式の後方互換判定）。
    let rest = match name.split_once('-') {
        Some((head, rest)) if !head.is_empty() && head.chars().all(|c| c.is_ascii_digit()) => rest,
        _ => name,
    };
    for kind in ["transcript", "refined", "note"] {
        // "{kind}-…" または ラベル無しの "{kind}.ext"。
        if rest.starts_with(&format!("{kind}-")) || rest.split('.').next() == Some(kind) {
            return kind;
        }
    }
    ""
}

/// 本文の冒頭を1行・最大 n 文字でプレビューする（純粋）。
pub fn preview_of(body: &str, n: usize) -> String {
    let one_line = body.split_whitespace().collect::<Vec<_>>().join(" ");
    let chars: Vec<char> = one_line.chars().collect();
    if chars.len() <= n {
        one_line
    } else {
        let head: String = chars.into_iter().take(n).collect();
        format!("{head}…")
    }
}

/// 一覧プレビューの最大文字数。
const PREVIEW_CHARS: usize = 140;
/// 要約のために読む先頭バイト数。フロントマターとプレビュー分の本文を賄う。
const SUMMARY_HEAD_BYTES: u64 = 8 * 1024;
/// 要約のために読む末尾バイト数。txt の末尾 `Tags:` 行を拾う分。
const SUMMARY_TAIL_BYTES: u64 = 4 * 1024;

/// バイト列を UTF-8 として解釈する。末尾が文字の途中で切れていれば切り捨てる。
fn decode_trim_end(buf: Vec<u8>) -> String {
    match String::from_utf8(buf) {
        Ok(s) => s,
        Err(e) => {
            let valid = e.utf8_error().valid_up_to();
            let mut b = e.into_bytes();
            b.truncate(valid);
            String::from_utf8(b).unwrap_or_default()
        }
    }
}

/// 末尾側チャンクを UTF-8 として解釈する。先頭が文字の途中なら継続バイトを読み飛ばす。
fn decode_trim_both(buf: Vec<u8>) -> String {
    let start = buf
        .iter()
        .position(|b| (b & 0xC0) != 0x80)
        .unwrap_or(buf.len());
    decode_trim_end(buf[start..].to_vec())
}

/// ファイルの指定範囲を読む。
fn read_range(path: &Path, offset: u64, len: u64) -> std::io::Result<Vec<u8>> {
    use std::io::{Read, Seek, SeekFrom};
    let mut f = std::fs::File::open(path)?;
    f.seek(SeekFrom::Start(offset))?;
    let mut buf = vec![0u8; len as usize];
    let mut filled = 0usize;
    while filled < buf.len() {
        match f.read(&mut buf[filled..])? {
            0 => break,
            n => filled += n,
        }
    }
    buf.truncate(filled);
    Ok(buf)
}

/// 一覧要約に必要な範囲だけを読んで解析する。返り値は (解析結果, 実際に読んだバイト数)。
///
/// 小さいファイルは従来どおり全文を読む。大きいファイルは先頭 [`SUMMARY_HEAD_BYTES`]
/// （＋txt形式なら末尾 [`SUMMARY_TAIL_BYTES`]）だけを読み、プレビューとメタを組み立てる。
/// 先頭窓だけでは従来と同じ結果を保証できない場合（フロントマターが窓をまたぐ・
/// プレビュー分の文字が取れない）は全文読み込みへフォールバックする。
fn read_summary_source(path: &Path) -> std::io::Result<(Parsed, u64)> {
    let size = std::fs::metadata(path)?.len();
    let read_all = |size: u64| -> std::io::Result<(Parsed, u64)> {
        let content = std::fs::read_to_string(path)?;
        Ok((parse_entry(&content), size))
    };
    if size <= SUMMARY_HEAD_BYTES + SUMMARY_TAIL_BYTES {
        return read_all(size);
    }

    let head_bytes = read_range(path, 0, SUMMARY_HEAD_BYTES)?;
    let head_len = head_bytes.len() as u64;
    let head = decode_trim_end(head_bytes);
    let text = head.trim_start_matches(['\u{feff}']);

    // md フロントマター形式: 終端 `---` が先頭窓に収まっていれば先頭だけで足りる
    // （この形式では末尾 Tags: 行を見ないため、末尾は不要）。
    if let Some(rest) = text
        .strip_prefix("---\n")
        .or_else(|| text.strip_prefix("---\r\n"))
    {
        // 終端が窓の外にあると txt 扱いへ落ちてメタを失うため、そのときは全文を読む。
        if rest.contains("\n---") {
            let parsed = parse_entry(&head);
            if preview_of(&parsed.body, PREVIEW_CHARS).ends_with('…') {
                return Ok((parsed, head_len));
            }
        }
        return read_all(size);
    }

    // txt 形式: プレビューは先頭から、タグは末尾から。
    let mut parsed = parse_entry(&head);
    if !preview_of(&parsed.body, PREVIEW_CHARS).ends_with('…') {
        // 先頭が空白ばかり等でプレビューを賄えないときだけ全文へ。
        return read_all(size);
    }
    let tail_bytes = read_range(path, size - SUMMARY_TAIL_BYTES, SUMMARY_TAIL_BYTES)?;
    let tail_len = tail_bytes.len() as u64;
    parsed.tags = parse_entry(&decode_trim_both(tail_bytes)).tags;
    Ok((parsed, head_len + tail_len))
}

/// 保管庫ディレクトリのエントリ(.txt/.md)を一覧する。created 降順。
pub fn list_entries(dir: &Path) -> Result<Vec<EntrySummary>, String> {
    let mut out: Vec<EntrySummary> = Vec::new();
    let rd = match std::fs::read_dir(dir) {
        Ok(rd) => rd,
        // 保管庫未作成（まだ何も保存していない）は空一覧。
        Err(_) => return Ok(out),
    };
    for ent in rd.flatten() {
        let path = ent.path();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if ext != "txt" && ext != "md" {
            continue;
        }
        let parsed = match read_summary_source(&path) {
            Ok((p, _)) => p,
            Err(_) => continue,
        };
        let created = parsed.created.unwrap_or_else(|| file_mtime_iso(&path));
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        // フロントマターに type が無ければ(主にtxt)ファイル名から種別を推定。
        let kind = parsed
            .kind
            .unwrap_or_else(|| kind_from_filename(&name).to_string());
        out.push(EntrySummary {
            path: path.to_string_lossy().to_string(),
            name,
            created,
            kind,
            tags: parsed.tags,
            preview: preview_of(&parsed.body, PREVIEW_CHARS),
        });
    }
    // created(ISO文字列)で降順。新しいものが上。
    out.sort_by(|a, b| b.created.cmp(&a.created));
    Ok(out)
}

/// ファイル更新時刻をISO8601(ローカル)で返す。取得不可は空。
fn file_mtime_iso(path: &Path) -> String {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .map(|t| {
            chrono::DateTime::<chrono::Local>::from(t)
                .format("%Y-%m-%dT%H:%M:%S")
                .to_string()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_md_frontmatter_extracts_meta_and_body() {
        let c = "---\ncreated: \"2026-06-27T12:00:00\"\ntype: \"refined\"\nstyle: \"構造化\"\ntags: [\"仕事\", \"不安\"]\n---\n\n本文ここ";
        let p = parse_entry(c);
        assert_eq!(p.created.as_deref(), Some("2026-06-27T12:00:00"));
        assert_eq!(p.kind.as_deref(), Some("refined"));
        assert_eq!(p.tags, vec!["仕事", "不安"]);
        assert_eq!(p.body, "本文ここ");
    }

    #[test]
    fn parse_md_tags_bullet_list_form() {
        // Obsidian的な箇条書きtagsも受ける。
        let c = "---\ntype: note\ntags:\n  - a\n  - b\n---\nbody";
        let p = parse_entry(c);
        assert_eq!(p.tags, vec!["a", "b"]);
        assert_eq!(p.kind.as_deref(), Some("note"));
    }

    #[test]
    fn parse_txt_trailing_tags_line() {
        let c = "本文だけ\n\nTags: アイデア, 仕事";
        let p = parse_entry(c);
        assert_eq!(p.tags, vec!["アイデア", "仕事"]);
        assert_eq!(p.body, "本文だけ");
        assert!(p.created.is_none());
    }

    #[test]
    fn parse_plain_txt_without_tags() {
        let p = parse_entry("ただの本文");
        assert!(p.tags.is_empty());
        assert_eq!(p.body, "ただの本文");
    }

    #[test]
    fn kind_from_filename_classifies() {
        // 旧形式（既存ファイルの後方互換）。
        assert_eq!(kind_from_filename("transcript-20260627-120000.txt"), "transcript");
        assert_eq!(kind_from_filename("refined-20260627-120000.md"), "refined");
        assert_eq!(kind_from_filename("note-x.txt"), "note");
        // 新形式 {yyyymmdd}-{種別}-{ラベル}（ADR-0032・日付先頭）。
        assert_eq!(kind_from_filename("20260724-transcript-今日の振り返り.txt"), "transcript");
        assert_eq!(kind_from_filename("20260724-refined-不安の整理.md"), "refined");
        assert_eq!(kind_from_filename("20260724-note.txt"), "note", "ラベル無しでも種別を読む");
        assert_eq!(kind_from_filename("20260724-note-2.txt"), "note", "衝突index付きも判定");
        assert_eq!(kind_from_filename("foo.txt"), "");
        assert_eq!(kind_from_filename("20260724-foo.txt"), "");
    }

    #[test]
    fn preview_truncates_and_singlelines() {
        assert_eq!(preview_of("a\nb  c", 10), "a b c");
        let long = "あ".repeat(200);
        let pv = preview_of(&long, 5);
        assert!(pv.ends_with('…'));
        assert_eq!(pv.chars().count(), 6); // 5文字＋…
    }

    #[test]
    fn parse_md_frontmatter_with_crlf_and_bom() {
        // 外部エディタ由来の CRLF / BOM 付きでもフロントマターを解釈する。
        let c = "\u{feff}---\r\ncreated: 2026-01-02T03:04:05\r\ntype: note\r\n---\r\n本文";
        let p = parse_entry(c);
        assert_eq!(p.created.as_deref(), Some("2026-01-02T03:04:05"));
        assert_eq!(p.kind.as_deref(), Some("note"));
        assert_eq!(p.body, "本文");
    }

    #[test]
    fn parse_unterminated_frontmatter_falls_back_to_txt() {
        // 終了 --- が無い場合はプレーンtxt扱い（本文を失わない）。
        let p = parse_entry("---\ntype: note\n本文つづき");
        assert!(p.kind.is_none());
        assert!(p.body.contains("本文つづき"));
    }

    #[test]
    fn parse_tag_inline_variants() {
        assert_eq!(parse_tag_inline("[\"a\", \"b\"]"), vec!["a", "b"]);
        assert_eq!(parse_tag_inline("#x、 y"), vec!["x", "y"]);
        assert!(parse_tag_inline("[ , ]").is_empty());
        assert_eq!(unquote("  \"q\"  "), "q");
        assert_eq!(unquote("noquote"), "noquote");
    }

    #[test]
    fn list_entries_reads_sorts_and_skips_non_entries() {
        let dir = std::env::temp_dir().join(format!("qs_vault_list_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("refined-a.md"),
            "---\ncreated: \"2026-07-02T10:00:00\"\ntype: \"refined\"\ntags: [\"内省\"]\n---\n新しい方の本文",
        )
        .unwrap();
        std::fs::write(dir.join("transcript-b.txt"), "古い方の本文\n\nTags: 仕事").unwrap();
        std::fs::write(dir.join("rec-c.wav"), b"RIFF....").unwrap(); // 対象外拡張子
        let entries = list_entries(&dir).unwrap();
        assert_eq!(entries.len(), 2, "音声ファイルは一覧に含めない");
        // frontmatter の created は過去日付、txt はファイル更新時刻(今) → txt が先頭（降順）。
        assert_eq!(entries[0].kind, "transcript", "txt はファイル名から種別推定");
        assert_eq!(entries[0].tags, vec!["仕事"]);
        assert!(!entries[0].created.is_empty(), "mtime を ISO で補完");
        assert_eq!(entries[1].kind, "refined");
        assert_eq!(entries[1].created, "2026-07-02T10:00:00");
        assert_eq!(entries[1].preview, "新しい方の本文");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// テスト用の空ディレクトリを作る。
    fn tmp_dir(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("qs_vault_{tag}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn summary_source_avoids_reading_whole_file() {
        // 巨大エントリでも要約に必要な先頭/末尾だけを読む（全文読み込みをしない）。
        let dir = tmp_dir("partial");
        let path = dir.join("20260726-note-big.txt");
        let filler = "あ".repeat(200_000); // 本文だけで約600KB(UTF-8)
        std::fs::write(&path, format!("先頭の本文\n{filler}\n\nTags: 仕事, 内省")).unwrap();
        let size = std::fs::metadata(&path).unwrap().len();

        let (parsed, read) = read_summary_source(&path).unwrap();

        assert!(read < size / 4, "全文({size}B)ではなく一部({read}B)だけを読む");
        assert_eq!(parsed.tags, vec!["仕事", "内省"], "末尾のタグ行は取りこぼさない");
        let pv = preview_of(&parsed.body, PREVIEW_CHARS);
        assert!(pv.starts_with("先頭の本文"), "プレビューは先頭から: {pv}");
        assert!(pv.ends_with('…'), "切り詰め済みの印が付く");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn summary_source_parses_frontmatter_that_exceeds_head_window() {
        // フロントマターが先頭読み込み窓をまたぐ場合は全文へフォールバックし、メタを取りこぼさない。
        let dir = tmp_dir("bigfm");
        let path = dir.join("20260726-refined-bigfm.md");
        let filler: String = (0..600)
            .map(|i| format!("note{i}: \"{}\"\n", "x".repeat(40)))
            .collect();
        std::fs::write(
            &path,
            format!(
                "---\ncreated: \"2026-07-26T10:00:00\"\n{filler}type: \"refined\"\ntags: [\"仕事\"]\n---\n本文ここ"
            ),
        )
        .unwrap();

        let (parsed, _) = read_summary_source(&path).unwrap();

        assert_eq!(parsed.created.as_deref(), Some("2026-07-26T10:00:00"));
        assert_eq!(parsed.kind.as_deref(), Some("refined"), "窓の外の type も読む");
        assert_eq!(parsed.tags, vec!["仕事"]);
        assert_eq!(parsed.body, "本文ここ");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn summary_source_falls_back_when_head_yields_too_little_preview() {
        // 先頭が空白ばかりでプレビュー分の文字が取れないときは全文を読む（プレビューを空にしない）。
        let dir = tmp_dir("latebody");
        let path = dir.join("20260726-note-late.txt");
        let pad = " ".repeat(20_000);
        std::fs::write(&path, format!("{pad}遅れて始まる本文\n\nTags: x")).unwrap();

        let (parsed, _) = read_summary_source(&path).unwrap();

        assert_eq!(preview_of(&parsed.body, PREVIEW_CHARS), "遅れて始まる本文");
        assert_eq!(parsed.tags, vec!["x"]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn summary_source_handles_multibyte_char_on_window_boundary() {
        // 読み込み境界がマルチバイト文字の途中でも壊れた文字を混ぜない。
        let dir = tmp_dir("boundary");
        // 3バイト文字を敷き詰めると 8KB 境界は必ず文字の途中に落ちる。
        for pad in 0..3usize {
            let path = dir.join(format!("20260726-note-b{pad}.txt"));
            let filler = format!("{}{}", "a".repeat(pad), "あ".repeat(100_000));
            std::fs::write(&path, format!("{filler}\n\nTags: t")).unwrap();
            let (parsed, _) = read_summary_source(&path).unwrap();
            assert!(!parsed.body.contains('\u{fffd}'), "置換文字を混ぜない(pad={pad})");
            assert_eq!(parsed.tags, vec!["t"]);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn list_entries_summarizes_large_entry_without_full_read() {
        // 一覧の外形（プレビュー・タグ・種別）は巨大エントリでも従来どおり。
        let dir = tmp_dir("largelist");
        let filler = "あ".repeat(200_000);
        std::fs::write(
            dir.join("20260726-transcript-big.txt"),
            format!("先頭の本文\n{filler}\n\nTags: 仕事"),
        )
        .unwrap();

        let entries = list_entries(&dir).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].kind, "transcript");
        assert_eq!(entries[0].tags, vec!["仕事"]);
        assert!(entries[0].preview.starts_with("先頭の本文"));
        assert!(entries[0].preview.ends_with('…'));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 1000件規模での所要時間を見るための計測用（CI では時間依存を避けるため既定で除外）。
    /// `cargo test -p quickscribe --lib vault::tests::bench_list_entries -- --ignored --nocapture`
    #[test]
    #[ignore = "ベンチ用: 時間依存のため既定では実行しない"]
    fn bench_list_entries_1000_entries() {
        let dir = tmp_dir("bench");
        let body = "あ".repeat(20_000); // 1件あたり約60KB
        for i in 0..1000 {
            std::fs::write(
                dir.join(format!("2026072{}-note-{i}.md", i % 10)),
                format!("---\ncreated: \"2026-07-2{}T10:00:00\"\ntype: \"note\"\ntags: [\"t\"]\n---\n{body}", i % 10),
            )
            .unwrap();
        }
        let t0 = std::time::Instant::now();
        let entries = list_entries(&dir).unwrap();
        let elapsed = t0.elapsed();
        assert_eq!(entries.len(), 1000);
        println!("list_entries(1000 entries, ~60KB each): {elapsed:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn list_entries_missing_dir_is_empty() {
        let dir = std::env::temp_dir().join("qs_vault_definitely_missing_xyz");
        assert!(list_entries(&dir).unwrap().is_empty());
    }
}
