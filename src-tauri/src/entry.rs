// エントリ(保管庫ドキュメント)ドメイン(#392 / DDD: lib.rs から抽出)。
// 文字起こし/整形結果を保存する際の「本文の組み立て」と「命名規則」を純粋関数として持つ。
// 出力形式(txt/md)・メタデータ(種別/スタイル/タグ)から決定的に本文を生成する(S4.2/S4.3/ADR-0017)。

/// エントリのスキーマ版(ADR-0017 / S4.4)。md フロントマターに刻む。
const CURRENT_ENTRY_SCHEMA: u32 = 1;

/// エントリのメタデータ(S4.2/S4.3 / Markdownフロントマター用)。
pub struct DocMeta<'a> {
    /// 種別: "transcript"(文字起こし) / "refined"(整形) / "note"(任意保存)。
    pub kind: &'a str,
    /// 整形スタイル(refined のときのみ Some)。
    pub style: Option<&'a str>,
    /// 内省タグ(S4.3)。空なら付与しない。
    pub tags: &'a [String],
}

/// 出力形式から拡張子を返す(純粋)。"md" 以外は "txt"。
pub fn doc_extension(format: &str) -> &'static str {
    if format.trim().eq_ignore_ascii_case("md") {
        "md"
    } else {
        "txt"
    }
}

/// YAMLフロントマターの値を1行・安全に整える(改行を空白化、前後空白除去)。
/// コロン等を含んでもダブルクォートで囲むため曖昧にならない。
fn yaml_scalar(v: &str) -> String {
    let one_line = v.replace(['\n', '\r'], " ");
    let escaped = one_line.replace('"', "'");
    format!("\"{}\"", escaped.trim())
}

/// 出力形式に応じてエントリ本文を組み立てる(純粋・テスト対象)。
/// md は schema/created/type(/style/tags) のYAMLフロントマターを本文の前に付す。
/// txt はタグがあれば末尾に `Tags: a, b` 行を付す(形式に依らずタグを残す / S4.3)。
pub fn build_document(content: &str, format: &str, created_iso: &str, meta: &DocMeta) -> String {
    if doc_extension(format) != "md" {
        // プレーンテキスト: 本文＋(タグがあれば)末尾にタグ行。
        if meta.tags.is_empty() {
            return content.to_string();
        }
        return format!("{}\n\nTags: {}", content, meta.tags.join(", "));
    }
    let mut fm = String::from("---\n");
    // エントリスキーマ版(S4.4 / ADR-0017)。将来の非破壊移行のための版マーカー。
    fm.push_str(&format!("schema: {CURRENT_ENTRY_SCHEMA}\n"));
    fm.push_str(&format!("created: {}\n", yaml_scalar(created_iso)));
    fm.push_str(&format!("type: {}\n", yaml_scalar(meta.kind)));
    if let Some(style) = meta.style {
        fm.push_str(&format!("style: {}\n", yaml_scalar(style)));
    }
    if !meta.tags.is_empty() {
        let items: Vec<String> = meta.tags.iter().map(|t| yaml_scalar(t)).collect();
        fm.push_str(&format!("tags: [{}]\n", items.join(", ")));
    }
    fm.push_str("---\n\n");
    fm.push_str(content);
    fm
}

/// 種別ごとのファイル名プレフィックス(純粋)。生の文字起こしと整形済みを名前で見分けられるようにする。
/// transcript=生の文字起こし / refined=整形済み / note=その他。
pub fn filename_prefix(kind: &str) -> &'static str {
    match kind {
        "transcript" => "transcript",
        "refined" => "refined",
        _ => "note",
    }
}

/// ファイル名ラベルの最大文字数。時刻(-hhMMss)を省いた分を内容由来ラベルに充て、
/// 一覧で内容を見分けられるようにする(ADR-0032)。
pub const LABEL_MAX_CHARS: usize = 24;

/// 本文冒頭やAIタイトルをファイル名に安全な1行ラベルへ整える(純粋)。
/// Windows禁止文字(\\/:*?"<>|)と制御文字を除去し、空白列(改行含む)は単一スペースへ畳み、
/// max_chars 文字で切る。両端のドット/空白は除く(Windowsの末尾制約＋隠しファイル風の見た目回避)。
pub fn sanitize_label(s: &str, max_chars: usize) -> String {
    let mut collapsed = String::new();
    let mut prev_space = true; // 先頭の空白は捨てる。
    for c in s.chars() {
        let c = if c.is_whitespace() { ' ' } else { c };
        if c.is_control() || matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
            continue;
        }
        if c == ' ' {
            if prev_space {
                continue;
            }
            prev_space = true;
        } else {
            prev_space = false;
        }
        collapsed.push(c);
    }
    let truncated: String = collapsed.chars().take(max_chars).collect();
    truncated.trim_matches([' ', '.']).to_string()
}

/// エントリのファイル名語幹を組み立てる(純粋)。`{種別}-{yyyymmdd}-{ラベル}`。
/// ラベルは本文冒頭(transcript/note)またはAIタイトル(refined)で、内容を名前で見分ける(ADR-0032)。
/// 時刻は含めない(タイムスタンプがファイル名を占有しないため)。ラベルが空(記号のみ等)なら
/// 日付までとし、同名衝突は保存側の一意化(-2, -3…)で吸収する。
pub fn entry_stem(kind: &str, date: &str, label: &str) -> String {
    let prefix = filename_prefix(kind);
    let label = sanitize_label(label, LABEL_MAX_CHARS);
    if label.is_empty() {
        format!("{prefix}-{date}")
    } else {
        format!("{prefix}-{date}-{label}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filename_prefix_distinguishes_kinds() {
        // 生の文字起こしと整形済みをファイル名で見分けられる。
        assert_eq!(filename_prefix("transcript"), "transcript");
        assert_eq!(filename_prefix("refined"), "refined");
        assert_eq!(filename_prefix("note"), "note");
        assert_eq!(filename_prefix("unknown"), "note");
    }

    #[test]
    fn doc_extension_maps_md_else_txt() {
        assert_eq!(doc_extension("md"), "md");
        assert_eq!(doc_extension("MD"), "md");
        assert_eq!(doc_extension("txt"), "txt");
        assert_eq!(doc_extension(""), "txt");
        assert_eq!(doc_extension("zzz"), "txt");
    }

    #[test]
    fn build_document_txt_is_content_only() {
        let meta = DocMeta {
            kind: "refined",
            style: Some("構造化"),
            tags: &[],
        };
        // txt は本文そのまま(フロントマター無し・後方互換)。
        assert_eq!(
            build_document("本文", "txt", "2026-06-27T12:00:00", &meta),
            "本文"
        );
    }

    #[test]
    fn build_document_md_refined_has_frontmatter_with_style() {
        let meta = DocMeta {
            kind: "refined",
            style: Some("構造化"),
            tags: &[],
        };
        let out = build_document("本文ABC", "md", "2026-06-27T12:00:00", &meta);
        assert!(out.starts_with("---\n"), "先頭はYAMLフロントマター");
        assert!(
            out.contains("schema: 1"),
            "スキーマ版マーカーを含む(ADR-0017)"
        );
        assert!(out.contains("created: \"2026-06-27T12:00:00\""));
        assert!(out.contains("type: \"refined\""));
        assert!(out.contains("style: \"構造化\""));
        assert!(out.contains("\n---\n\n本文ABC"), "本文が後続する");
    }

    #[test]
    fn build_document_md_transcript_omits_style() {
        // style 無し(transcript)では style 行を出さない(三角測量)。
        let meta = DocMeta {
            kind: "transcript",
            style: None,
            tags: &[],
        };
        let out = build_document("x", "md", "2026-06-27T12:00:00", &meta);
        assert!(out.contains("type: \"transcript\""));
        assert!(!out.contains("style:"), "style 行は無い");
        assert!(!out.contains("tags:"), "tags 行は無い");
    }

    #[test]
    fn build_document_md_includes_tags_when_present() {
        let tags = vec!["仕事".to_string(), "不安".to_string()];
        let meta = DocMeta {
            kind: "refined",
            style: Some("構造化"),
            tags: &tags,
        };
        let out = build_document("本文", "md", "2026-06-27T12:00:00", &meta);
        assert!(
            out.contains("tags: [\"仕事\", \"不安\"]"),
            "frontmatterにtags配列: {out}"
        );
    }

    #[test]
    fn build_document_txt_appends_tags_line() {
        // txt でもタグがあれば末尾に Tags: 行を付す(形式に依らずタグを残す)。
        let tags = vec!["アイデア".to_string()];
        let meta = DocMeta {
            kind: "note",
            style: None,
            tags: &tags,
        };
        let out = build_document("本文", "txt", "2026-06-27T12:00:00", &meta);
        assert_eq!(out, "本文\n\nTags: アイデア");
    }

    #[test]
    fn sanitize_label_strips_invalid_chars_and_collapses_whitespace() {
        // Windows禁止文字は除去、改行/連続空白は単一スペースへ。
        assert_eq!(sanitize_label("a/b\\c:d*e?f\"g<h>i|j", 24), "abcdefghij");
        assert_eq!(sanitize_label("  今日は\n晴れ  です  ", 24), "今日は 晴れ です");
    }

    #[test]
    fn sanitize_label_truncates_by_chars_and_trims_edge_dots() {
        // 文字数(バイトでなく)で切る。両端のドット/空白は落とす(Windows末尾制約＋隠しファイル風回避)。
        assert_eq!(sanitize_label(&"あ".repeat(30), 24), "あ".repeat(24));
        assert_eq!(sanitize_label("title...", 8), "title");
        assert_eq!(sanitize_label("...これは冒頭", 24), "これは冒頭", "先頭ドットも除く");
        assert_eq!(sanitize_label("???", 24), "", "記号のみは空になる");
    }

    #[test]
    fn entry_stem_appends_label_after_date() {
        // 種別-日付-ラベル。時刻は含めない(ADR-0032)。
        assert_eq!(
            entry_stem("transcript", "20260724", "今日の振り返り"),
            "transcript-20260724-今日の振り返り"
        );
        assert_eq!(
            entry_stem("refined", "20260724", "仕事の不安の整理"),
            "refined-20260724-仕事の不安の整理"
        );
    }

    #[test]
    fn entry_stem_without_label_is_date_only() {
        // ラベルが空(記号のみ等)なら従来同様プレフィックス+日付。
        assert_eq!(entry_stem("note", "20260724", ""), "note-20260724");
        assert_eq!(entry_stem("note", "20260724", "***"), "note-20260724");
    }

    #[test]
    fn yaml_scalar_is_single_line_and_quoted() {
        // 改行はスペース化、二重引用符は単引用符化し、ダブルクォートで囲む(YAML安全)。
        let s = yaml_scalar("行1\n行2: \"値\"");
        assert!(!s.contains('\n'));
        assert!(s.starts_with('"') && s.ends_with('"'));
        assert!(s.contains("行1 行2"));
    }
}
