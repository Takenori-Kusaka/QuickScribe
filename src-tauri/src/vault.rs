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

/// ファイル名のプレフィックスから種別を推定する（フロントマター欠落時の補完・主にtxt用）。
/// transcript-/refined-/note- を見分ける。該当なしは空。
pub fn kind_from_filename(name: &str) -> &'static str {
    if name.starts_with("transcript-") {
        "transcript"
    } else if name.starts_with("refined-") {
        "refined"
    } else if name.starts_with("note-") {
        "note"
    } else {
        ""
    }
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
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let parsed = parse_entry(&content);
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
            preview: preview_of(&parsed.body, 140),
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
        assert_eq!(kind_from_filename("transcript-20260627-120000.txt"), "transcript");
        assert_eq!(kind_from_filename("refined-20260627-120000.md"), "refined");
        assert_eq!(kind_from_filename("note-x.txt"), "note");
        assert_eq!(kind_from_filename("foo.txt"), "");
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

    #[test]
    fn list_entries_missing_dir_is_empty() {
        let dir = std::env::temp_dir().join("qs_vault_definitely_missing_xyz");
        assert!(list_entries(&dir).unwrap().is_empty());
    }
}
